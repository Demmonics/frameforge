use serde::{Deserialize, Serialize};

/// Milestone G non-destructive operation recipe foundation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditOperationRecipe {
    pub id: String,
    pub operation: String,
    pub requires_reencode: bool,
    pub parameters: serde_json::Value,
}
