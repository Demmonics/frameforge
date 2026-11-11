use chrono::Utc;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::conversion::command_builder::build_execution_args;
use crate::core::errors::FrameForgeError;
use crate::core::events::CoreEvent;
use crate::core::models::{
    CapabilityRegistry, ConversionJob, ConversionSettings, JobState, MediaFile,
};
use crate::hardware::fallback::resolve_cpu_fallback_settings;
use crate::jobs::cancellation::JobCancelHandle;
use crate::jobs::progress::ProgressFrameAccumulator;
use crate::jobs::queue::JobQueueStore;
use crate::platform::configure_background_command;
use crate::storage::output_manager::{check_disk_space_for_output, resolve_output_path};
use crate::storage::temporary_files::StagedOutput;

pub struct ExecutionRequest {
    pub initial_job: ConversionJob,
    pub media: MediaFile,
    pub settings: ConversionSettings,
    pub registry: CapabilityRegistry,
    pub cancel: JobCancelHandle,
}

const MAX_LOG_LINES: usize = 400;

pub async fn execute_conversion_job(
    req: ExecutionRequest,
    store: Arc<JobQueueStore>,
    event_tx: Option<mpsc::UnboundedSender<CoreEvent>>,
) {
    let tx_ref = event_tx.as_ref();
    let mut job = req.initial_job;
    if req.cancel.is_cancelled() {
        finalize_cancelled(&mut job, &store, tx_ref);
        return;
    }

    job.state = JobState::Validating;
    job.started_at_utc = Some(Utc::now().to_rfc3339());
    emit_job(&store, tx_ref, &job);

    let outcome = run_attempt(
        &mut job,
        &req.media,
        &req.settings,
        &req.registry,
        &req.cancel,
        &store,
        tx_ref,
    )
    .await;

    match outcome {
        Ok(final_path) => finalize_completed(&mut job, &final_path, &store, tx_ref).await,
        Err(FrameForgeError::Cancelled) => finalize_cancelled(&mut job, &store, tx_ref),
        Err(err) => {
            let failed_enc = job.actual_encoder.clone().unwrap_or_default();
            if let Some(fallback_settings) =
                resolve_cpu_fallback_settings(&req.settings, &failed_enc, &req.registry)
                && !req.cancel.is_cancelled()
            {
                push_log(
                    &mut job,
                    tx_ref,
                    format!(
                        "Hardware encoder '{failed_enc}' failed ({err}). Falling back to CPU encoder as configured."
                    ),
                );
                emit_job(&store, tx_ref, &job);

                match run_attempt(
                    &mut job,
                    &req.media,
                    &fallback_settings,
                    &req.registry,
                    &req.cancel,
                    &store,
                    tx_ref,
                )
                .await
                {
                    Ok(final_path) => {
                        finalize_completed(&mut job, &final_path, &store, tx_ref).await;
                    }
                    Err(FrameForgeError::Cancelled) => finalize_cancelled(&mut job, &store, tx_ref),
                    Err(fallback_err) => finalize_failed(&mut job, &fallback_err, &store, tx_ref),
                }
            } else {
                finalize_failed(&mut job, &err, &store, tx_ref);
            }
        }
    }
}

async fn finalize_completed(
    job: &mut ConversionJob,
    final_path: &Path,
    store: &Arc<JobQueueStore>,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
) {
    job.output_path = final_path.display().to_string();
    job.output_size_bytes = tokio::fs::metadata(final_path).await.ok().map(|m| m.len());
    job.progress.percent = 100.0;
    job.progress.eta_secs = Some(0.0);
    job.state = JobState::Completed;
    job.finished_at_utc = Some(Utc::now().to_rfc3339());
    emit_job(store, event_tx, job);
}

fn finalize_cancelled(
    job: &mut ConversionJob,
    store: &Arc<JobQueueStore>,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
) {
    job.state = JobState::Cancelled;
    job.error = None;
    job.finished_at_utc = Some(Utc::now().to_rfc3339());
    emit_job(store, event_tx, job);
}

fn finalize_failed(
    job: &mut ConversionJob,
    err: &FrameForgeError,
    store: &Arc<JobQueueStore>,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
) {
    job.state = JobState::Failed;
    job.error = Some(err.to_serializable());
    job.finished_at_utc = Some(Utc::now().to_rfc3339());
    emit_job(store, event_tx, job);
}

async fn run_attempt(
    job: &mut ConversionJob,
    media: &MediaFile,
    settings: &ConversionSettings,
    registry: &CapabilityRegistry,
    cancel: &JobCancelHandle,
    store: &Arc<JobQueueStore>,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
) -> crate::core::errors::Result<std::path::PathBuf> {
    let (mut staged, ffmpeg_program, os_args) =
        prepare_staged_execution(job, media, settings, registry, store, event_tx).await?;

    let mut cmd = Command::new(&ffmpeg_program);
    cmd.args(&os_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_background_command(&mut cmd);

    let mut child = cmd.spawn().map_err(|source| FrameForgeError::Io {
        context: format!("spawning ffmpeg at {ffmpeg_program}"),
        source,
    })?;

    let (mut child_stdin, mut line_rx) = attach_process_pipes(&mut child);
    let (was_cancelled, exit_status, stderr_tail) = drive_ffmpeg_loop(
        &mut child,
        &mut child_stdin,
        &mut line_rx,
        job,
        media.duration_secs,
        cancel,
        store,
        event_tx,
    )
    .await;

    if was_cancelled || cancel.is_cancelled() {
        staged.cleanup_async().await;
        return Err(FrameForgeError::Cancelled);
    }

    let status = exit_status.ok_or_else(|| FrameForgeError::ConversionFailed {
        exit_code: None,
        summary: "FFmpeg process terminated unexpectedly".to_string(),
        raw_stderr: stderr_tail.join("\n"),
    })?;

    if !status.success() {
        staged.cleanup_async().await;
        let summary = stderr_tail
            .iter()
            .rev()
            .find(|l| !l.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| format!("FFmpeg exited with code {:?}", status.code()));
        return Err(FrameForgeError::ConversionFailed {
            exit_code: status.code(),
            summary,
            raw_stderr: stderr_tail.join("\n"),
        });
    }

    match staged.finalize(settings.overwrite_policy).await {
        Ok(p) => Ok(p),
        Err(e) => {
            staged.cleanup_async().await;
            Err(e)
        }
    }
}

async fn prepare_staged_execution(
    job: &mut ConversionJob,
    media: &MediaFile,
    settings: &ConversionSettings,
    registry: &CapabilityRegistry,
    store: &Arc<JobQueueStore>,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
) -> crate::core::errors::Result<(StagedOutput, String, Vec<std::ffi::OsString>)> {
    let temp_probe_plan =
        crate::conversion::command_builder::build_conversion_plan(media, settings, registry)?;
    let ext = Path::new(&temp_probe_plan.resolved_output_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or(&settings.container)
        .to_string();

    let media_clone = media.clone();
    let settings_clone = settings.clone();
    let ext_clone = ext.clone();
    let min_required = (media.size_bytes / 10).max(1024 * 1024);

    let resolved_final_path = tokio::task::spawn_blocking(move || {
        let final_path = resolve_output_path(&media_clone, &settings_clone, &ext_clone)?;
        check_disk_space_for_output(&final_path, min_required)?;
        Ok::<_, FrameForgeError>(final_path)
    })
    .await
    .map_err(|e| FrameForgeError::Internal(format!("Output staging task failed: {e}")))??;

    let mut staged = StagedOutput::create_async(resolved_final_path, &ext).await?;
    let (plan, os_args) = build_execution_args(media, settings, registry, &staged.temp_path)?;

    if plan.decision == crate::core::models::DecisionKind::Fail {
        staged.cleanup_async().await;
        return Err(FrameForgeError::IncompatibleConversion {
            summary: plan.summary,
            reasons: plan.errors,
        });
    }

    job.actual_encoder = plan
        .resolved_video_encoder
        .clone()
        .or_else(|| plan.resolved_audio_encoder.clone());
    job.output_path = staged.final_path.display().to_string();
    job.state = JobState::Running;
    push_log(job, event_tx, format!("Running: {}", plan.command_preview));
    emit_job(store, event_tx, job);

    Ok((staged, plan.ffmpeg_program, os_args))
}

fn attach_process_pipes(
    child: &mut tokio::process::Child,
) -> (
    Option<tokio::process::ChildStdin>,
    mpsc::UnboundedReceiver<StreamMsg>,
) {
    let child_stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (line_tx, line_rx) = mpsc::unbounded_channel::<StreamMsg>();

    if let Some(out) = stdout {
        let tx = line_tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx.send(StreamMsg::Stdout(line));
            }
        });
    }
    if let Some(err) = stderr {
        let tx = line_tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx.send(StreamMsg::Stderr(line));
            }
        });
    }
    drop(line_tx);
    (child_stdin, line_rx)
}

#[allow(clippy::too_many_arguments)]
async fn drive_ffmpeg_loop(
    child: &mut tokio::process::Child,
    child_stdin: &mut Option<tokio::process::ChildStdin>,
    line_rx: &mut mpsc::UnboundedReceiver<StreamMsg>,
    job: &mut ConversionJob,
    duration_secs: Option<f64>,
    cancel: &JobCancelHandle,
    store: &Arc<JobQueueStore>,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
) -> (bool, Option<std::process::ExitStatus>, Vec<String>) {
    let start_instant = Instant::now();
    let mut acc = ProgressFrameAccumulator::default();
    let mut stderr_tail: Vec<String> = Vec::new();
    let mut was_cancelled = false;
    let mut last_emit = Instant::now();
    let token = cancel.token();
    let exit_status: Option<std::process::ExitStatus>;

    loop {
        tokio::select! {
            () = token.cancelled(), if !was_cancelled => {
                was_cancelled = true;
                job.state = JobState::Cancelling;
                push_log(job, event_tx, "Cancellation requested; stopping FFmpeg...".to_string());
                emit_job(store, event_tx, job);

                if let Some(mut stdin) = child_stdin.take() {
                    let _ = stdin.write_all(b"q\n").await;
                    let _ = stdin.flush().await;
                }

                exit_status = if let Ok(Ok(status)) =
                    tokio::time::timeout(std::time::Duration::from_secs(2), child.wait()).await
                {
                    Some(status)
                } else {
                    let _ = child.kill().await;
                    child.wait().await.ok()
                };
                break;
            }
            maybe_msg = line_rx.recv() => {
                match maybe_msg {
                    Some(StreamMsg::Stdout(line)) => {
                        let elapsed_ms = u64::try_from(start_instant.elapsed().as_millis()).unwrap_or(u64::MAX);
                        if let Some(prog) = acc.push_line(&line, duration_secs, elapsed_ms) {
                            job.progress = prog;
                            if last_emit.elapsed() >= std::time::Duration::from_millis(120) || line.trim() == "progress=end" {
                                emit_job(store, event_tx, job);
                                last_emit = Instant::now();
                            }
                        }
                    }
                    Some(StreamMsg::Stderr(line)) => {
                        record_stderr_line(&line, &mut stderr_tail, job, event_tx);
                    }
                    None => {
                        exit_status = child.wait().await.ok();
                        break;
                    }
                }
            }
        }
    }

    while let Ok(msg) = line_rx.try_recv() {
        if let StreamMsg::Stderr(line) = msg {
            record_stderr_line(&line, &mut stderr_tail, job, event_tx);
        }
    }

    (was_cancelled, exit_status, stderr_tail)
}

fn record_stderr_line(
    line: &str,
    stderr_tail: &mut Vec<String>,
    job: &mut ConversionJob,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
) {
    let trimmed = line.trim().to_string();
    if !trimmed.is_empty() {
        if stderr_tail.len() >= 40 {
            stderr_tail.remove(0);
        }
        stderr_tail.push(trimmed.clone());
        push_log(job, event_tx, trimmed);
    }
}

enum StreamMsg {
    Stdout(String),
    Stderr(String),
}

fn push_log(
    job: &mut ConversionJob,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
    line: String,
) {
    if job.logs.len() >= MAX_LOG_LINES {
        job.logs.remove(0);
    }
    job.logs.push(line.clone());
    if let Some(tx) = event_tx {
        let _ = tx.send(CoreEvent::JobLog {
            job_id: job.id.clone(),
            line,
        });
    }
}

fn emit_job(
    store: &Arc<JobQueueStore>,
    event_tx: Option<&mpsc::UnboundedSender<CoreEvent>>,
    job: &ConversionJob,
) {
    store.update(job);
    if let Some(tx) = event_tx {
        let _ = tx.send(CoreEvent::JobUpdated(Box::new(job.clone())));
    }
}
