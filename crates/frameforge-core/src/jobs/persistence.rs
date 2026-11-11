use crate::core::errors::Result;
use crate::core::models::ConversionJob;

/// Trait boundary for SQLite job persistence (implemented in Milestone D).
pub trait JobPersistence: Send + Sync {
    /// Persists or updates a conversion job record.
    ///
    /// # Errors
    /// Returns a [`FrameForgeError`](crate::core::errors::FrameForgeError) if persistence fails.
    fn save_job(&self, job: &ConversionJob) -> Result<()>;

    /// Loads previously persisted conversion job records.
    ///
    /// # Errors
    /// Returns a [`FrameForgeError`](crate::core::errors::FrameForgeError) if loading fails.
    fn load_jobs(&self) -> Result<Vec<ConversionJob>>;
}

#[derive(Debug, Default)]
pub struct InMemoryPersistence;

impl JobPersistence for InMemoryPersistence {
    fn save_job(&self, _job: &ConversionJob) -> Result<()> {
        Ok(())
    }

    fn load_jobs(&self) -> Result<Vec<ConversionJob>> {
        Ok(Vec::new())
    }
}
