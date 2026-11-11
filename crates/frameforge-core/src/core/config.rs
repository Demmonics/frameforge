use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::models::OverwritePolicy;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BinaryPathsConfig {
    pub ffmpeg_override: Option<PathBuf>,
    pub ffprobe_override: Option<PathBuf>,
    pub mpv_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConcurrencyConfig {
    pub max_cpu_jobs: usize,
    pub max_gpu_jobs: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_cpu_jobs: 2,
            max_gpu_jobs: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutputDefaults {
    pub default_output_dir: Option<PathBuf>,
    pub filename_pattern: String,
    pub overwrite_policy: OverwritePolicy,
}

impl Default for OutputDefaults {
    fn default() -> Self {
        Self {
            default_output_dir: None,
            filename_pattern: "{name}_converted".to_string(),
            overwrite_policy: OverwritePolicy::RenameIfExists,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub binaries: BinaryPathsConfig,
    pub concurrency: ConcurrencyConfig,
    pub output: OutputDefaults,
    pub cache_dir: PathBuf,
    pub max_cache_bytes: u64,
    pub probe_hardware_at_startup: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let cache_dir = std::env::temp_dir().join("frameforge-cache");
        Self {
            binaries: BinaryPathsConfig::default(),
            concurrency: ConcurrencyConfig::default(),
            output: OutputDefaults::default(),
            cache_dir,
            max_cache_bytes: 512 * 1024 * 1024,
            probe_hardware_at_startup: true,
        }
    }
}
