use std::path::{Path, PathBuf};

use crate::core::errors::{FrameForgeError, Result};

/// Resolves a tool executable path on macOS (`override` -> app dirs -> `PATH`).
///
/// # Errors
/// Returns [`FrameForgeError::BinaryNotFound`] if the executable does not exist in any search root.
pub fn discover_binary(tool: &str, override_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(custom) = override_path
        && custom.is_file()
    {
        return Ok(custom.to_path_buf());
    }
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        for rel in [exe_dir.join("bin").join(tool), exe_dir.join(tool)] {
            if rel.is_file() {
                return Ok(rel);
            }
        }
    }
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(tool);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(FrameForgeError::BinaryNotFound {
        tool: tool.to_string(),
        searched: "override, bundled bin, PATH".to_string(),
    })
}

#[must_use]
pub fn available_disk_space(_path: &Path) -> Option<u64> {
    None
}
