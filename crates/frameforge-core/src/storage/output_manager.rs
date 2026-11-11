use std::path::{Path, PathBuf};

use crate::core::errors::{FrameForgeError, Result};
use crate::core::models::{ConversionSettings, MediaFile, OverwritePolicy};
use crate::platform;
use crate::storage::temporary_files::deduplicate_path;

/// Sanitizes an arbitrary filename stem by stripping path separators, Windows-reserved characters,
/// and DOS device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`).
#[must_use]
pub fn sanitize_filename_stem(raw: &str) -> String {
    let invalid: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if invalid.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        return "output".to_string();
    }

    let first_segment = trimmed.split('.').next().unwrap_or(trimmed);
    if is_windows_reserved_device_name(first_segment) {
        return format!("_{trimmed}");
    }

    trimmed.to_string()
}

fn is_windows_reserved_device_name(name: &str) -> bool {
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    RESERVED
        .iter()
        .any(|reserved| name.eq_ignore_ascii_case(reserved))
}

/// Resolves the output path from template variables, collision avoidance, and overwrite policy.
///
/// # Errors
/// Returns [`FrameForgeError::InvalidInputPath`] if `media.path` lacks a parent directory, or
/// [`FrameForgeError::OutputConflict`] when `OverwritePolicy::FailIfExists` encounters a collision.
pub fn resolve_output_path(
    media: &MediaFile,
    settings: &ConversionSettings,
    target_extension: &str,
) -> Result<PathBuf> {
    let input_path = Path::new(&media.path);
    let default_dir = input_path
        .parent()
        .ok_or_else(|| FrameForgeError::InvalidInputPath {
            path: media.path.clone(),
            reason: "Input path has no parent directory".to_string(),
        })?;

    let output_dir = match settings.output_dir.as_deref().map(str::trim) {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => default_dir.to_path_buf(),
    };

    let input_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("media");

    let pattern = if settings.filename_pattern.trim().is_empty() {
        "{name}_converted"
    } else {
        settings.filename_pattern.trim()
    };

    let codec_token = settings
        .video_encoder
        .as_deref()
        .unwrap_or(settings.video_codec.as_str());
    let preset_token = settings
        .preset_name
        .as_deref()
        .unwrap_or(settings.container.as_str());

    let expanded = pattern
        .replace("{name}", input_stem)
        .replace("{codec}", codec_token)
        .replace("{container}", &settings.container)
        .replace("{preset}", preset_token);

    let safe_stem = sanitize_filename_stem(&expanded);
    let ext = target_extension.trim_start_matches('.');
    let mut candidate = if ext.is_empty() {
        output_dir.join(safe_stem)
    } else {
        output_dir.join(format!("{safe_stem}.{ext}"))
    };

    // Never collide directly with the active input file!
    if paths_match_case_aware(&candidate, input_path) {
        let stem = candidate
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        candidate = if ext.is_empty() {
            output_dir.join(format!("{stem}_out"))
        } else {
            output_dir.join(format!("{stem}_out.{ext}"))
        };
    }

    match settings.overwrite_policy {
        OverwritePolicy::RenameIfExists => Ok(deduplicate_path(&candidate)),
        OverwritePolicy::FailIfExists => {
            if candidate.exists() {
                Err(FrameForgeError::OutputConflict {
                    path: candidate.display().to_string(),
                })
            } else {
                Ok(candidate)
            }
        }
        OverwritePolicy::Overwrite => Ok(candidate),
    }
}

/// Verifies available disk space against `estimated_min_bytes` before starting conversion.
///
/// # Errors
/// Returns [`FrameForgeError::InsufficientDiskSpace`] if available space is below the requirement.
pub fn check_disk_space_for_output(output_path: &Path, estimated_min_bytes: u64) -> Result<()> {
    let check_dir = output_path.parent().unwrap_or(output_path);
    let existing_ancestor = find_existing_ancestor(check_dir);
    if let Some(dir) = existing_ancestor
        && let Some(free) = platform::available_disk_space(&dir)
        && free < estimated_min_bytes
    {
        return Err(FrameForgeError::InsufficientDiskSpace {
            required_bytes: estimated_min_bytes,
            available_bytes: free,
            path: dir.display().to_string(),
        });
    }
    Ok(())
}

fn find_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut cur = Some(path);
    while let Some(p) = cur {
        if p.exists() {
            return Some(p.to_path_buf());
        }
        cur = p.parent();
    }
    None
}

fn paths_match_case_aware(a: &Path, b: &Path) -> bool {
    if cfg!(target_os = "windows") {
        normalize_windows_components(a) == normalize_windows_components(b)
    } else {
        normalize_lexical_components(a) == normalize_lexical_components(b)
    }
}

fn normalize_windows_components(path: &Path) -> Vec<String> {
    normalize_lexical_components(path)
        .into_iter()
        .map(|seg| seg.to_ascii_lowercase())
        .collect()
}

fn normalize_lexical_components(path: &Path) -> Vec<String> {
    let mut parts = Vec::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if matches!(parts.last(), Some(last) if last != "..") {
                    parts.pop();
                } else {
                    parts.push("..".to_string());
                }
            }
            _ => parts.push(comp.as_os_str().to_string_lossy().into_owned()),
        }
    }
    parts
}
