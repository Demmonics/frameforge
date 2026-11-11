#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use std::path::{Path, PathBuf};

use crate::core::errors::Result;

/// Applies platform background subprocess flags and ensures child processes are terminated
/// automatically if their owning future or `Child` handle is dropped (e.g., on timeout/cancellation).
pub fn configure_background_command(cmd: &mut tokio::process::Command) {
    cmd.kill_on_drop(true);
    #[cfg(target_os = "windows")]
    windows::configure_no_window(cmd);
}

/// Discovers an external media CLI binary from explicit overrides, bundled app paths, or `PATH`.
///
/// # Errors
/// Returns [`FrameForgeError::BinaryNotFound`](crate::core::errors::FrameForgeError::BinaryNotFound)
/// when the executable cannot be located in any candidate location.
pub fn discover_binary(tool: &str, override_path: Option<&Path>) -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    return windows::discover_binary(tool, override_path);
    #[cfg(target_os = "linux")]
    return linux::discover_binary(tool, override_path);
    #[cfg(target_os = "macos")]
    return macos::discover_binary(tool, override_path);
}

/// Queries available free disk space in bytes at `path` (or its existing parent directory).
#[must_use]
pub fn available_disk_space(path: &Path) -> Option<u64> {
    #[cfg(target_os = "windows")]
    return windows::available_disk_space(path);
    #[cfg(target_os = "linux")]
    return linux::available_disk_space(path);
    #[cfg(target_os = "macos")]
    return macos::available_disk_space(path);
}

/// Returns the platform executable suffix (`".exe"` on Windows, `""` elsewhere).
#[must_use]
pub fn executable_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    }
}
