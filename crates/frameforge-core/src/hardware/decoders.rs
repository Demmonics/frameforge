use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareDecoderInfo {
    pub name: String,
    pub hwaccel: String,
}
