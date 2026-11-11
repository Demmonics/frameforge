use std::path::{Path, PathBuf};

use crate::core::errors::{FrameForgeError, Result};

#[derive(Debug, Clone)]
pub struct CacheManager {
    root: PathBuf,
    max_bytes: u64,
}

impl CacheManager {
    /// Initializes a cache manager rooted at `root`, creating the directory if needed.
    ///
    /// # Errors
    /// Returns [`FrameForgeError::Io`] if `root` cannot be created.
    pub fn new(root: PathBuf, max_bytes: u64) -> Result<Self> {
        std::fs::create_dir_all(&root).map_err(|source| FrameForgeError::Io {
            context: format!("creating cache directory {}", root.display()),
            source,
        })?;
        Ok(Self { root, max_bytes })
    }

    #[must_use]
    pub fn thumbnails_dir(&self) -> PathBuf {
        self.root.join("thumbnails")
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn enforce_budget(&self) {
        let Ok(entries) = std::fs::read_dir(self.thumbnails_dir()) else {
            return;
        };
        let mut files: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
        let mut total_size: u64 = 0;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = entry.metadata()
                && meta.is_file()
            {
                let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                total_size = total_size.saturating_add(meta.len());
                files.push((path, meta.len(), modified));
            }
        }

        if total_size <= self.max_bytes {
            return;
        }

        files.sort_by_key(|(_, _, time)| *time);
        for (path, size, _) in files {
            if total_size <= self.max_bytes {
                break;
            }
            if std::fs::remove_file(&path).is_ok() {
                total_size = total_size.saturating_sub(size);
            }
        }
    }
}
