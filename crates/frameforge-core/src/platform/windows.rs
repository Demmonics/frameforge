use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::core::errors::{FrameForgeError, Result};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available_to_caller: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
}

pub fn configure_no_window(cmd: &mut tokio::process::Command) {
    cmd.creation_flags(CREATE_NO_WINDOW);
}

/// Resolves a tool executable path on Windows (`override` -> app dirs -> `PATH`).
///
/// # Errors
/// Returns [`FrameForgeError::BinaryNotFound`] if the executable does not exist in any search root.
pub fn discover_binary(tool: &str, override_path: Option<&Path>) -> Result<PathBuf> {
    let exe_name = if Path::new(tool)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        tool.to_string()
    } else {
        format!("{tool}.exe")
    };

    let mut searched = Vec::new();

    if let Some(custom) = override_path {
        searched.push(format!("override={}", custom.display()));
        if custom.is_file() {
            return Ok(custom.to_path_buf());
        }
    }

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        for rel in [exe_dir.join("bin").join(&exe_name), exe_dir.join(&exe_name)] {
            searched.push(rel.display().to_string());
            if rel.is_file() {
                return Ok(rel);
            }
        }
    }

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(&exe_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        searched.push("PATH".to_string());
    }

    Err(FrameForgeError::BinaryNotFound {
        tool: tool.to_string(),
        searched: searched.join(", "),
    })
}

/// Queries caller-available free bytes on the volume backing `path`.
#[must_use]
pub fn available_disk_space(path: &Path) -> Option<u64> {
    let target = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };
    let mut wide: Vec<u16> = target.as_os_str().encode_wide().collect();
    wide.push(0);

    let mut free_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;

    // SAFETY: `wide` is null-terminated UTF-16 valid for the call duration, and the three output
    // `u64` pointers point to distinct aligned local stack variables.
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &raw mut free_available,
            &raw mut total_bytes,
            &raw mut total_free,
        )
    };
    if ok == 0 { None } else { Some(free_available) }
}
