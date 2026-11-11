use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::models::{ConversionJob, JobState};
use crate::jobs::cancellation::JobCancelHandle;

#[derive(Debug)]
pub struct JobQueueStore {
    jobs: RwLock<Vec<ConversionJob>>,
    cancel_handles: RwLock<HashMap<String, JobCancelHandle>>,
}

impl JobQueueStore {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            jobs: RwLock::new(Vec::new()),
            cancel_handles: RwLock::new(HashMap::new()),
        })
    }

    pub fn insert(&self, job: &ConversionJob, cancel: JobCancelHandle) {
        if let Ok(mut list) = self.jobs.write() {
            list.insert(0, job.clone());
        }
        if let Ok(mut map) = self.cancel_handles.write() {
            map.insert(job.id.clone(), cancel);
        }
    }

    pub fn update(&self, updated: &ConversionJob) {
        if let Ok(mut list) = self.jobs.write()
            && let Some(slot) = list.iter_mut().find(|j| j.id == updated.id)
        {
            *slot = updated.clone();
        }
    }

    pub fn list(&self) -> Vec<ConversionJob> {
        self.jobs.read().map(|l| l.clone()).unwrap_or_default()
    }

    pub fn get(&self, job_id: &str) -> Option<ConversionJob> {
        self.jobs
            .read()
            .ok()
            .and_then(|l| l.iter().find(|j| j.id == job_id).cloned())
    }

    pub fn request_cancel(&self, job_id: &str) -> bool {
        let handle = self
            .cancel_handles
            .read()
            .ok()
            .and_then(|m| m.get(job_id).cloned());
        if let Some(h) = handle {
            h.cancel();
            if let Ok(mut list) = self.jobs.write()
                && let Some(job) = list.iter_mut().find(|j| j.id == job_id)
                && matches!(
                    job.state,
                    JobState::Queued | JobState::Validating | JobState::Running
                )
            {
                job.state = JobState::Cancelling;
            }
            true
        } else {
            false
        }
    }

    pub fn remove_finished(&self) -> usize {
        if let Ok(mut list) = self.jobs.write() {
            let before = list.len();
            list.retain(|j| {
                matches!(
                    j.state,
                    JobState::Queued
                        | JobState::Validating
                        | JobState::Running
                        | JobState::Paused
                        | JobState::Cancelling
                )
            });
            before.saturating_sub(list.len())
        } else {
            0
        }
    }
}
