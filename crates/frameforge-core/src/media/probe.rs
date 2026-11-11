use chrono::Utc;
use std::path::Path;
use tokio::process::Command;
use uuid::Uuid;

use crate::core::errors::{FrameForgeError, Result};
use crate::core::models::MediaFile;
use crate::media::metadata::{RawFfprobeOutput, normalize_probe_output};
use crate::platform::configure_background_command;

/// Asynchronously probes a local media file via `ffprobe` without shell interpolation.
///
/// # Errors
/// Returns [`FrameForgeError::InvalidInputPath`] if the file does not exist or is not regular,
/// [`FrameForgeError::EncryptedOrCorruptMedia`] for empty/corrupt/DRM inputs,
/// [`FrameForgeError::UnsupportedMedia`] when no recognizable streams exist, or
/// [`FrameForgeError::ProbeFailed`] / [`FrameForgeError::Io`] on process failures.
pub async fn probe_media_file(ffprobe_path: &Path, input_path: &Path) -> Result<MediaFile> {
    if !tokio::fs::try_exists(input_path).await.unwrap_or(false) {
        return Err(FrameForgeError::InvalidInputPath {
            path: input_path.display().to_string(),
            reason: "File does not exist".to_string(),
        });
    }
    let meta = tokio::fs::metadata(input_path)
        .await
        .map_err(|source| FrameForgeError::Io {
            context: format!("reading file metadata for {}", input_path.display()),
            source,
        })?;
    if !meta.is_file() {
        return Err(FrameForgeError::InvalidInputPath {
            path: input_path.display().to_string(),
            reason: "Path is not a regular file".to_string(),
        });
    }
    if meta.len() == 0 {
        return Err(FrameForgeError::EncryptedOrCorruptMedia {
            path: input_path.display().to_string(),
            reason: "File is empty (0 bytes)".to_string(),
        });
    }

    let abs_path = tokio::fs::canonicalize(input_path).await.map_or_else(
        |_| input_path.to_path_buf(),
        |p| normalize_windows_verbatim_path(&p),
    );

    let mut cmd = Command::new(ffprobe_path);
    cmd.args([
        "-v",
        "error",
        "-protocol_whitelist",
        "file,pipe",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
        "-show_chapters",
        "-i",
    ])
    .arg(abs_path.as_os_str());
    configure_background_command(&mut cmd);

    let output = cmd.output().await.map_err(|source| FrameForgeError::Io {
        context: format!("spawning ffprobe for {}", abs_path.display()),
        source,
    })?;

    let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout_str = String::from_utf8_lossy(&output.stdout).into_owned();

    if !output.status.success() {
        return Err(classify_probe_error(
            &abs_path,
            output.status.code(),
            stderr_str,
        ));
    }

    let raw: RawFfprobeOutput =
        serde_json::from_str(&stdout_str).map_err(|err| FrameForgeError::ProbeFailed {
            path: abs_path.display().to_string(),
            exit_code: output.status.code(),
            summary: format!("Failed to parse FFprobe JSON output: {err}"),
            raw_stderr: stderr_str.clone(),
        })?;

    if raw.streams.is_empty() {
        return Err(FrameForgeError::UnsupportedMedia {
            path: abs_path.display().to_string(),
            reason: "Container contains no recognizable audio, video, or subtitle streams"
                .to_string(),
        });
    }

    let file_name = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(normalize_probe_output(
        Uuid::new_v4().to_string(),
        file_name,
        abs_path.display().to_string(),
        meta.len(),
        raw,
        stdout_str,
        Utc::now().to_rfc3339(),
    ))
}

fn classify_probe_error(path: &Path, exit_code: Option<i32>, stderr: String) -> FrameForgeError {
    let lower = stderr.to_ascii_lowercase();
    let path_str = path.display().to_string();

    if lower.contains("encrypted") || lower.contains("drm") || lower.contains("cenc") {
        return FrameForgeError::EncryptedOrCorruptMedia {
            path: path_str,
            reason: "Media appears to be encrypted or DRM-protected".to_string(),
        };
    }

    if lower.contains("invalid data")
        || lower.contains("moov atom not found")
        || lower.contains("ebml header")
        || lower.contains("end of file")
    {
        return FrameForgeError::EncryptedOrCorruptMedia {
            path: path_str,
            reason: if stderr.is_empty() {
                "Media header is truncated or corrupt".to_string()
            } else {
                stderr
            },
        };
    }

    FrameForgeError::ProbeFailed {
        path: path_str,
        exit_code,
        summary: if stderr.is_empty() {
            "FFprobe rejected input file".to_string()
        } else {
            stderr
                .lines()
                .next()
                .unwrap_or("FFprobe failed")
                .to_string()
        },
        raw_stderr: stderr,
    }
}

#[must_use]
pub fn normalize_windows_verbatim_path(path: &Path) -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        let s = path.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
            return std::path::PathBuf::from(format!(r"\\{stripped}"));
        }
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            return std::path::PathBuf::from(stripped);
        }
    }
    path.to_path_buf()
}
