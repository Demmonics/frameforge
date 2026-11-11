use serde::{Deserialize, Serialize};

use crate::core::models::JobState;

/// Foundation record type for Milestone D SQLite persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobHistoryEntry {
    pub job_id: String,
    pub input_path: String,
    pub output_path: String,
    pub state: JobState,
    pub created_at_utc: String,
    pub finished_at_utc: Option<String>,
}
