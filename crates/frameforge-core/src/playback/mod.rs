use serde::{Deserialize, Serialize};

/// Milestone E playback domain model foundation (mpv/libmpv IPC controller).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackReadiness {
    pub mpv_available: bool,
    pub mpv_path: Option<String>,
    pub mpv_version: Option<String>,
    pub target_milestone: &'static str,
    pub notes: String,
}
