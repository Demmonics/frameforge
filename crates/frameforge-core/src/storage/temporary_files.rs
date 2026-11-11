use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::core::errors::{FrameForgeError, Result};
use crate::core::models::OverwritePolicy;

#[derive(Debug)]
pub struct StagedOutput {
    pub temp_path: PathBuf,
    pub final_path: PathBuf,
    finalized: bool,
}

impl StagedOutput {
    /// Creates a staged temporary output descriptor and ensures its parent directory exists synchronously.
    ///
    /// # Errors
    /// Returns [`FrameForgeError::ValidationError`] if `final_path` lacks a parent directory,
    /// or [`FrameForgeError::Io`] if directory creation fails.
    pub fn create(final_path: PathBuf, extension: &str) -> Result<Self> {
        let (parent, temp_path) = prepare_staged_paths(&final_path, extension)?;
        std::fs::create_dir_all(&parent).map_err(|source| FrameForgeError::Io {
            context: format!("creating output directory {}", parent.display()),
            source,
        })?;

        Ok(Self {
            temp_path,
            final_path,
            finalized: false,
        })
    }

    /// Asynchronous equivalent of [`StagedOutput::create`] safe for Tokio worker tasks.
    ///
    /// # Errors
    /// Returns [`FrameForgeError::ValidationError`] if `final_path` lacks a parent directory,
    /// or [`FrameForgeError::Io`] if directory creation fails.
    pub async fn create_async(final_path: PathBuf, extension: &str) -> Result<Self> {
        let (parent, temp_path) = prepare_staged_paths(&final_path, extension)?;
        tokio::fs::create_dir_all(&parent)
            .await
            .map_err(|source| FrameForgeError::Io {
                context: format!("creating output directory {}", parent.display()),
                source,
            })?;

        Ok(Self {
            temp_path,
            final_path,
            finalized: false,
        })
    }

    /// Atomically moves the completed temporary output to its final destination path according
    /// to `overwrite_policy`.
    ///
    /// # Errors
    /// Returns [`FrameForgeError::ConversionFailed`] if no staged file was written,
    /// [`FrameForgeError::OutputConflict`] when `FailIfExists` encounters an existing file,
    /// or [`FrameForgeError::Io`] if filesystem operations fail.
    pub async fn finalize(&mut self, overwrite_policy: OverwritePolicy) -> Result<PathBuf> {
        if !tokio::fs::try_exists(&self.temp_path)
            .await
            .unwrap_or(false)
        {
            return Err(FrameForgeError::ConversionFailed {
                exit_code: None,
                summary: "FFmpeg exited without producing an output file".to_string(),
                raw_stderr: format!("Missing temporary file {}", self.temp_path.display()),
            });
        }

        let target = match overwrite_policy {
            // `tokio::fs::rename` atomically replaces `final_path` when both paths live in the
            // same parent directory, preserving the existing file intact if the rename fails.
            OverwritePolicy::Overwrite => self.final_path.clone(),
            OverwritePolicy::FailIfExists => {
                if tokio::fs::try_exists(&self.final_path)
                    .await
                    .unwrap_or(false)
                {
                    return Err(FrameForgeError::OutputConflict {
                        path: self.final_path.display().to_string(),
                    });
                }
                self.final_path.clone()
            }
            OverwritePolicy::RenameIfExists => {
                let candidate = self.final_path.clone();
                tokio::task::spawn_blocking(move || deduplicate_path(&candidate))
                    .await
                    .unwrap_or_else(|_| self.final_path.clone())
            }
        };

        tokio::fs::rename(&self.temp_path, &target)
            .await
            .map_err(|source| FrameForgeError::Io {
                context: format!(
                    "finalizing output {} -> {}",
                    self.temp_path.display(),
                    target.display()
                ),
                source,
            })?;

        self.finalized = true;
        Ok(target)
    }

    pub async fn cleanup_async(&mut self) {
        if !self.finalized
            && tokio::fs::try_exists(&self.temp_path)
                .await
                .unwrap_or(false)
        {
            let _ = tokio::fs::remove_file(&self.temp_path).await;
        }
    }
}

fn prepare_staged_paths(final_path: &Path, extension: &str) -> Result<(PathBuf, PathBuf)> {
    let parent = final_path
        .parent()
        .ok_or_else(|| FrameForgeError::ValidationError {
            field: "outputPath".to_string(),
            message: "Output path must include a parent directory".to_string(),
        })?
        .to_path_buf();

    let raw_stem = final_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let stem: String = raw_stem.chars().take(64).collect();
    let short_id = &Uuid::new_v4().simple().to_string()[..8];
    let safe_ext = extension.trim_start_matches('.');
    let temp_name = if safe_ext.is_empty() {
        format!(".{stem}.frameforge-{short_id}.tmp")
    } else {
        format!(".{stem}.frameforge-{short_id}.tmp.{safe_ext}")
    };
    let temp_path = parent.join(temp_name);
    Ok((parent, temp_path))
}

impl Drop for StagedOutput {
    fn drop(&mut self) {
        if !self.finalized && self.temp_path.exists() {
            let _ = std::fs::remove_file(&self.temp_path);
        }
    }
}

/// Generates a non-colliding file path by appending `_1`, `_2`, ... before the extension.
#[must_use]
pub fn deduplicate_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = path.extension().and_then(|s| s.to_str());

    for idx in 1..10_000 {
        let file_name = match ext {
            Some(e) if !e.is_empty() => format!("{stem}_{idx}.{e}"),
            _ => format!("{stem}_{idx}"),
        };
        let candidate = parent.join(file_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("{stem}_{}", Uuid::new_v4().simple()))
}
