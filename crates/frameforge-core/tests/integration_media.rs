#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

use frameforge_core::conversion::command_builder::build_conversion_plan;
use frameforge_core::conversion::profiles::builtin_presets;
use frameforge_core::conversion::validation::is_safe_token;
use frameforge_core::core::config::AppConfig;
use frameforge_core::core::errors::FrameForgeError;
use frameforge_core::core::models::{
    AudioMode, ConversionSettings, DecisionKind, HardwareMode, JobState, OverwritePolicy,
    RateControlMode, ResolutionPreset, SubtitleMode,
};
use frameforge_core::jobs::progress::ProgressFrameAccumulator;
use frameforge_core::jobs::scheduler::JobScheduler;
use frameforge_core::media::capabilities::build_capability_registry;
use frameforge_core::media::metadata::{
    detect_vfr, extract_hdr_info, extract_rotation_degrees, parse_rational_fps,
};
use frameforge_core::media::probe::probe_media_file;
use frameforge_core::media::thumbnails::generate_thumbnail;
use frameforge_core::storage::output_manager::resolve_output_path;

fn generate_test_mp4(path: &Path, duration_secs: u32, width: u32, height: u32, audio_first: bool) {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);
    if audio_first {
        cmd.args([
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency=880:duration={duration_secs}"),
            "-f",
            "lavfi",
            "-i",
            &format!("testsrc2=duration={duration_secs}:size={width}x{height}:rate=30"),
            "-map",
            "0:a:0",
            "-map",
            "1:v:0",
        ]);
    } else {
        cmd.args([
            "-f",
            "lavfi",
            "-i",
            &format!("testsrc=duration={duration_secs}:size={width}x{height}:rate=30"),
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency=440:duration={duration_secs}"),
        ]);
    }
    cmd.args([
        "-c:v",
        "libx264",
        "-preset",
        "ultrafast",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-b:a",
        "96k",
    ])
    .arg(path);

    let status = cmd
        .status()
        .expect("ffmpeg must be available to run integration tests");
    assert!(
        status.success(),
        "synthetic fixture generation failed for {}",
        path.display()
    );
}

async fn wait_for_terminal_state(
    scheduler: &JobScheduler,
    job_id: &str,
    timeout: Duration,
) -> frameforge_core::core::models::ConversionJob {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Some(job) = scheduler.store.get(job_id)
            && matches!(
                job.state,
                JobState::Completed | JobState::Failed | JobState::Cancelled
            )
        {
            return job;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for job {job_id}"
        );
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
}

#[tokio::test]
async fn capabilities_probe_and_presets_validate() {
    let cfg = AppConfig::default();
    let caps = build_capability_registry(&cfg).await;
    assert!(caps.ffmpeg.available, "FFmpeg should be available");
    assert!(caps.ffprobe.available, "FFprobe should be available");
    assert!(caps.muxers.iter().any(|m| m.name == "mp4"));
    assert!(caps.muxers.iter().any(|m| m.name == "matroska"));
    assert!(caps.encoders.iter().any(|e| e.name == "libx264"));

    let mp4 = caps
        .supported_containers
        .iter()
        .find(|c| c.id == "mp4")
        .expect("mp4 container entry missing");
    assert!(mp4.available);

    let presets = builtin_presets(&caps);
    assert!(
        presets
            .iter()
            .any(|p| p.id == "general-h264-mp4" && p.available)
    );
    assert!(
        presets
            .iter()
            .any(|p| p.id == "remux-mkv-lossless" && p.available)
    );
}

#[tokio::test]
async fn probe_audio_first_stream_order_and_thumbnail() {
    let tmp = TempDir::new().unwrap();
    let sample = tmp.path().join("audio_first_sample.mp4");
    generate_test_mp4(&sample, 2, 320, 240, true);

    let cfg = AppConfig::default();
    let caps = build_capability_registry(&cfg).await;
    let ffprobe_path = PathBuf::from(caps.ffprobe.path.as_ref().unwrap());
    let ffmpeg_path = PathBuf::from(caps.ffmpeg.path.as_ref().unwrap());

    let media = probe_media_file(&ffprobe_path, &sample)
        .await
        .expect("ffprobe should succeed");

    // Stream 0 is audio, stream 1 is video — verify we never assume stream index 0 is video!
    let pv = media.primary_video.as_ref().expect("primary video missing");
    assert_eq!(pv.index, 1);
    assert_eq!(pv.codec_name, "h264");
    assert_eq!(pv.width, 320);
    assert_eq!(pv.height, 240);
    assert_eq!(media.primary_audio.as_ref().unwrap().index, 0);
    assert!(media.technical_report.contains("VIDEO #1 (Stream #1)"));

    let thumb = generate_thumbnail(&ffmpeg_path, &media, tmp.path()).await;
    assert!(thumb.is_some(), "thumbnail should be generated");
    let (data_url, cache_path) = thumb.unwrap();
    assert!(data_url.starts_with("data:image/jpeg;base64,"));
    assert!(Path::new(&cache_path).exists());
}

#[tokio::test]
async fn remux_mp4_to_mkv_stream_copy_and_verify_with_ffprobe() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("source.mp4");
    generate_test_mp4(&input, 2, 320, 240, false);

    let cfg = AppConfig::default();
    let caps = build_capability_registry(&cfg).await;
    let ffprobe_path = PathBuf::from(caps.ffprobe.path.as_ref().unwrap());
    let media = probe_media_file(&ffprobe_path, &input).await.unwrap();

    let settings = ConversionSettings {
        container: "mkv".into(),
        video_codec: "copy".into(),
        audio_mode: AudioMode::Copy,
        subtitle_mode: SubtitleMode::CopyCompatible,
        faststart: false,
        output_dir: Some(tmp.path().display().to_string()),
        filename_pattern: "{name}_remux".into(),
        ..Default::default()
    };

    let plan = build_conversion_plan(&media, &settings, &caps).unwrap();
    assert_eq!(plan.decision, DecisionKind::Remux);

    let scheduler = JobScheduler::new(&cfg.concurrency);
    let job = scheduler
        .enqueue_conversion(media, settings, caps, None)
        .unwrap();
    let finished = wait_for_terminal_state(&scheduler, &job.id, Duration::from_secs(20)).await;
    assert_eq!(
        finished.state,
        JobState::Completed,
        "error: {:?}",
        finished.error
    );

    let out_path = PathBuf::from(&finished.output_path);
    assert!(out_path.exists());
    let probed_out = probe_media_file(&ffprobe_path, &out_path).await.unwrap();
    assert!(probed_out.container.format_name.contains("matroska"));
    assert_eq!(probed_out.primary_video.unwrap().codec_name, "h264");
    assert_eq!(probed_out.primary_audio.unwrap().codec_name, "aac");
}

#[tokio::test]
async fn transcode_mp4_to_hevc_and_extract_audio_flac() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("clip.mp4");
    generate_test_mp4(&input, 1, 320, 240, false);

    let cfg = AppConfig::default();
    let caps = build_capability_registry(&cfg).await;
    let ffprobe_path = PathBuf::from(caps.ffprobe.path.as_ref().unwrap());
    let media = probe_media_file(&ffprobe_path, &input).await.unwrap();
    let scheduler = JobScheduler::new(&cfg.concurrency);

    // 1. Transcode to HEVC via CPU libx265
    let hevc_settings = ConversionSettings {
        container: "mp4".into(),
        video_codec: "hevc".into(),
        video_encoder: Some("libx265".into()),
        hardware_mode: HardwareMode::Cpu,
        rate_control: RateControlMode::Crf,
        quality_value: 28,
        encoder_preset: Some("ultrafast".into()),
        audio_mode: AudioMode::Transcode,
        audio_codec: Some("aac".into()),
        audio_bitrate_kbps: Some(128),
        output_dir: Some(tmp.path().display().to_string()),
        filename_pattern: "{name}_hevc".into(),
        ..Default::default()
    };

    let job1 = scheduler
        .enqueue_conversion(media.clone(), hevc_settings, caps.clone(), None)
        .unwrap();
    let done1 = wait_for_terminal_state(&scheduler, &job1.id, Duration::from_secs(30)).await;
    assert_eq!(
        done1.state,
        JobState::Completed,
        "HEVC failed: {:?}",
        done1.error
    );

    let probed_hevc = probe_media_file(&ffprobe_path, Path::new(&done1.output_path))
        .await
        .unwrap();
    assert_eq!(probed_hevc.primary_video.unwrap().codec_name, "hevc");

    // 2. Extract Audio to FLAC
    let flac_settings = ConversionSettings {
        container: "flac".into(),
        video_codec: "copy".into(),
        audio_mode: AudioMode::Transcode,
        audio_codec: Some("flac".into()),
        faststart: false,
        output_dir: Some(tmp.path().display().to_string()),
        filename_pattern: "{name}_audio".into(),
        ..Default::default()
    };

    let job2 = scheduler
        .enqueue_conversion(media, flac_settings, caps, None)
        .unwrap();
    let done2 = wait_for_terminal_state(&scheduler, &job2.id, Duration::from_secs(20)).await;
    assert_eq!(
        done2.state,
        JobState::Completed,
        "FLAC failed: {:?}",
        done2.error
    );

    let probed_flac = probe_media_file(&ffprobe_path, Path::new(&done2.output_path))
        .await
        .unwrap();
    assert_eq!(probed_flac.video_streams, [] as [_; 0]);
    assert_eq!(probed_flac.primary_audio.unwrap().codec_name, "flac");
}

#[tokio::test]
async fn batch_convert_and_cancel_active_job_cleans_temp_files() {
    let tmp = TempDir::new().unwrap();
    let f1 = tmp.path().join("batch_one.mp4");
    let f2 = tmp.path().join("batch_two.mp4");
    let slow_src = tmp.path().join("slow_cancel_src.mp4");

    generate_test_mp4(&f1, 1, 320, 240, false);
    generate_test_mp4(&f2, 1, 320, 240, false);
    // 18-second HD source encoded with veryslow guarantees enough runtime to cancel cleanly mid-flight
    generate_test_mp4(&slow_src, 18, 1280, 720, false);

    let cfg = AppConfig::default();
    let caps = build_capability_registry(&cfg).await;
    let ffprobe_path = PathBuf::from(caps.ffprobe.path.as_ref().unwrap());
    let scheduler = JobScheduler::new(&cfg.concurrency);

    // Batch remux 2 files
    let m1 = probe_media_file(&ffprobe_path, &f1).await.unwrap();
    let m2 = probe_media_file(&ffprobe_path, &f2).await.unwrap();
    let batch_settings = ConversionSettings {
        container: "mkv".into(),
        video_codec: "copy".into(),
        audio_mode: AudioMode::Copy,
        output_dir: Some(tmp.path().display().to_string()),
        filename_pattern: "{name}_batch".into(),
        ..Default::default()
    };
    let j1 = scheduler
        .enqueue_conversion(m1, batch_settings.clone(), caps.clone(), None)
        .unwrap();
    let j2 = scheduler
        .enqueue_conversion(m2, batch_settings, caps.clone(), None)
        .unwrap();
    let d1 = wait_for_terminal_state(&scheduler, &j1.id, Duration::from_secs(20)).await;
    let d2 = wait_for_terminal_state(&scheduler, &j2.id, Duration::from_secs(20)).await;
    assert_eq!(d1.state, JobState::Completed);
    assert_eq!(d2.state, JobState::Completed);
    assert_ne!(d1.output_path, d2.output_path);

    // Slow encode for mid-flight cancellation test
    let slow_media = probe_media_file(&ffprobe_path, &slow_src).await.unwrap();
    let cancel_out_dir = tmp.path().join("cancel_out");
    std::fs::create_dir_all(&cancel_out_dir).unwrap();

    let slow_settings = ConversionSettings {
        container: "mp4".into(),
        video_codec: "hevc".into(),
        video_encoder: Some("libx265".into()),
        hardware_mode: HardwareMode::Cpu,
        encoder_preset: Some("veryslow".into()),
        output_dir: Some(cancel_out_dir.display().to_string()),
        filename_pattern: "should_not_exist".into(),
        ..Default::default()
    };

    let cancel_job = scheduler
        .enqueue_conversion(slow_media, slow_settings, caps, None)
        .unwrap();

    // Wait until FFmpeg is actively running
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(j) = scheduler.store.get(&cancel_job.id)
            && j.state == JobState::Running
        {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "job never reached Running"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(scheduler.store.request_cancel(&cancel_job.id));
    let cancelled =
        wait_for_terminal_state(&scheduler, &cancel_job.id, Duration::from_secs(10)).await;
    assert_eq!(cancelled.state, JobState::Cancelled);

    // Ensure temp and final files do not remain after cancellation
    let remaining_files: Vec<_> = std::fs::read_dir(&cancel_out_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(
        remaining_files.is_empty(),
        "expected clean output dir after cancel, found: {:?}",
        remaining_files
            .iter()
            .map(std::fs::DirEntry::file_name)
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn rejects_incompatible_combinations_and_corrupt_inputs_and_handles_unicode() {
    let tmp = TempDir::new().unwrap();
    let unicode_input = tmp.path().join("тест 動画 café [4K].mp4");
    generate_test_mp4(&unicode_input, 1, 320, 240, false);

    let cfg = AppConfig::default();
    let caps = build_capability_registry(&cfg).await;
    let ffprobe_path = PathBuf::from(caps.ffprobe.path.as_ref().unwrap());

    // 1. Unicode filename probe & MKV remux
    let media = probe_media_file(&ffprobe_path, &unicode_input)
        .await
        .unwrap();
    let ok_settings = ConversionSettings {
        container: "mkv".into(),
        video_codec: "copy".into(),
        audio_mode: AudioMode::Copy,
        output_dir: Some(tmp.path().display().to_string()),
        filename_pattern: "{name}_unicode_out".into(),
        ..Default::default()
    };
    let scheduler = JobScheduler::new(&cfg.concurrency);
    let ujob = scheduler
        .enqueue_conversion(media.clone(), ok_settings, caps.clone(), None)
        .unwrap();
    let udone = wait_for_terminal_state(&scheduler, &ujob.id, Duration::from_secs(20)).await;
    assert_eq!(udone.state, JobState::Completed);
    assert!(Path::new(&udone.output_path).exists());

    // 2. Reject H.264 stream copy into WebM
    let bad_webm = ConversionSettings {
        container: "webm".into(),
        video_codec: "copy".into(),
        audio_mode: AudioMode::Copy,
        ..Default::default()
    };
    let bad_plan = build_conversion_plan(&media, &bad_webm, &caps).unwrap();
    assert_eq!(bad_plan.decision, DecisionKind::Fail);
    assert_ne!(bad_plan.errors, [] as [String; 0]);
    assert!(matches!(
        scheduler.enqueue_conversion(media.clone(), bad_webm, caps.clone(), None),
        Err(FrameForgeError::IncompatibleConversion { .. })
    ));

    // 3. Reject scale filter when video_codec == copy
    let bad_scale_copy = ConversionSettings {
        container: "mp4".into(),
        video_codec: "copy".into(),
        resolution: ResolutionPreset::Hd720p,
        ..Default::default()
    };
    let bad_scale_plan = build_conversion_plan(&media, &bad_scale_copy, &caps).unwrap();
    assert_eq!(bad_scale_plan.decision, DecisionKind::Fail);

    // 4. Corrupt and empty file errors
    let empty_file = tmp.path().join("empty.mp4");
    std::fs::write(&empty_file, b"").unwrap();
    assert!(matches!(
        probe_media_file(&ffprobe_path, &empty_file).await,
        Err(FrameForgeError::EncryptedOrCorruptMedia { .. })
    ));

    let corrupt_file = tmp.path().join("corrupt.mp4");
    std::fs::write(
        &corrupt_file,
        b"not a valid mp4 container header at all 1234567890",
    )
    .unwrap();
    let corrupt_res = probe_media_file(&ffprobe_path, &corrupt_file).await;
    assert!(corrupt_res.is_err());

    // 5. Output collision safe renaming
    let existing = tmp.path().join("collision_converted.mp4");
    std::fs::write(&existing, b"existing").unwrap();
    let col_settings = ConversionSettings {
        output_dir: Some(tmp.path().display().to_string()),
        filename_pattern: "collision_converted".into(),
        overwrite_policy: OverwritePolicy::RenameIfExists,
        ..Default::default()
    };
    let resolved = resolve_output_path(&media, &col_settings, "mp4").unwrap();
    assert!(
        resolved
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("_1.mp4")
    );
}

#[test]
fn unit_tests_progress_metadata_and_validation() {
    // Progress parser
    let mut acc = ProgressFrameAccumulator::default();
    assert!(acc.push_line("frame=120", Some(10.0), 1000).is_none());
    assert!(acc.push_line("fps=60.0", Some(10.0), 1000).is_none());
    assert!(
        acc.push_line("out_time_us=5000000", Some(10.0), 1000)
            .is_none()
    );
    assert!(acc.push_line("speed=2.0x", Some(10.0), 1000).is_none());
    let prog = acc
        .push_line("progress=continue", Some(10.0), 1000)
        .expect("should emit progress on continue");
    assert!((prog.percent - 50.0).abs() < 0.1);
    assert!((prog.eta_secs.unwrap() - 2.5).abs() < 0.1);

    // VFR, FPS, Rotation, HDR helpers
    let nom = parse_rational_fps(Some("30000/1001")).unwrap();
    assert!((nom - 29.97).abs() < 0.01);
    let (vfr, _) = detect_vfr(Some(30.0), Some(24.1));
    assert!(vfr);
    let (cfr, _) = detect_vfr(Some(29.97), Some(29.97));
    assert!(!cfr);

    let mut tags = BTreeMap::new();
    tags.insert("rotate".to_string(), "90".to_string());
    assert_eq!(extract_rotation_degrees(&tags, &[]), Some(90));

    let (hdr_fmt, _) = extract_hdr_info(Some("smpte2084"), Some("bt2020"), &[]);
    assert!(hdr_fmt.unwrap().contains("SMPTE ST 2084"));

    // Security token validation
    assert!(is_safe_token("high"));
    assert!(is_safe_token("4.1"));
    assert!(!is_safe_token("high; rm -rf /"));
    assert!(!is_safe_token("high && calc.exe"));
    assert!(!is_safe_token("bad space"));
}
