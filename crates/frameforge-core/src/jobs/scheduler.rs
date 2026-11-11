use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};

use crate::conversion::transcoder::{ExecutionRequest, execute_conversion_job};
use crate::core::config::ConcurrencyConfig;
use crate::core::events::CoreEvent;
use crate::core::models::{
    CapabilityRegistry, ConversionJob, ConversionSettings, DecisionKind, JobProgress, JobState,
    MediaFile,
};
use crate::jobs::cancellation::JobCancelHandle;
use crate::jobs::queue::JobQueueStore;

#[derive(Clone)]
pub struct JobScheduler {
    pub store: Arc<JobQueueStore>,
    cpu_semaphore: Arc<Semaphore>,
    gpu_semaphore: Arc<Semaphore>,
}

impl JobScheduler {
    #[must_use]
    pub fn new(concurrency: &ConcurrencyConfig) -> Self {
        Self {
            store: JobQueueStore::new(),
            cpu_semaphore: Arc::new(Semaphore::new(concurrency.max_cpu_jobs.max(1))),
            gpu_semaphore: Arc::new(Semaphore::new(concurrency.max_gpu_jobs.max(1))),
        }
    }

    /// Validates, enqueues, and spawns an asynchronous conversion job under bounded concurrency.
    ///
    /// # Errors
    /// Returns [`FrameForgeError::IncompatibleConversion`](crate::core::errors::FrameForgeError::IncompatibleConversion)
    /// when the validated conversion plan evaluates to [`DecisionKind::Fail`].
    pub fn enqueue_conversion(
        &self,
        media: MediaFile,
        settings: ConversionSettings,
        registry: CapabilityRegistry,
        event_tx: Option<mpsc::UnboundedSender<CoreEvent>>,
    ) -> crate::core::errors::Result<ConversionJob> {
        let plan = crate::conversion::command_builder::build_conversion_plan(
            &media, &settings, &registry,
        )?;
        if plan.decision == DecisionKind::Fail {
            return Err(
                crate::core::errors::FrameForgeError::IncompatibleConversion {
                    summary: plan.summary,
                    reasons: plan.errors,
                },
            );
        }

        let job_id = uuid::Uuid::new_v4().to_string();
        let initial_job = ConversionJob {
            id: job_id.clone(),
            input_path: media.path.clone(),
            output_path: plan.resolved_output_path.clone(),
            media_info: media.clone(),
            selected_preset: settings.preset_name.clone(),
            settings: settings.clone(),
            hardware_mode: settings.hardware_mode,
            decision: plan.decision,
            state: JobState::Queued,
            progress: JobProgress::default(),
            actual_encoder: plan.resolved_video_encoder.clone(),
            logs: Vec::new(),
            error: None,
            output_size_bytes: None,
            created_at_utc: chrono::Utc::now().to_rfc3339(),
            started_at_utc: None,
            finished_at_utc: None,
        };

        let cancel_handle = JobCancelHandle::new();
        self.store.insert(&initial_job, cancel_handle.clone());
        if let Some(tx) = &event_tx {
            let _ = tx.send(CoreEvent::JobUpdated(Box::new(initial_job.clone())));
        }

        let semaphore = if plan.using_hardware {
            self.gpu_semaphore.clone()
        } else {
            self.cpu_semaphore.clone()
        };

        let store = self.store.clone();
        let spawn_job = initial_job.clone();
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await;
            execute_conversion_job(
                ExecutionRequest {
                    initial_job: spawn_job,
                    media,
                    settings,
                    registry,
                    cancel: cancel_handle,
                },
                store,
                event_tx,
            )
            .await;
        });

        Ok(self.store.get(&job_id).unwrap_or(initial_job))
    }
}
