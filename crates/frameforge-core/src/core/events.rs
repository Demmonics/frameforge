use serde::{Deserialize, Serialize};

use super::models::{CapabilityRegistry, ConversionJob};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum CoreEvent {
    JobUpdated(Box<ConversionJob>),
    JobLog { job_id: String, line: String },
    CapabilitiesUpdated(Box<CapabilityRegistry>),
}
