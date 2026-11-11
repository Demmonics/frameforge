#![allow(clippy::needless_pass_by_value)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::{Semaphore, mpsc};

use frameforge_core::conversion::command_builder::build_conversion_plan;
use frameforge_core::conversion::profiles::builtin_presets;
use frameforge_core::core::config::AppConfig;
use frameforge_core::core::errors::{FrameForgeError, SerializableError};
use frameforge_core::core::events::CoreEvent;
use frameforge_core::core::models::{
    CapabilityRegistry, ConversionJob, ConversionPlan, ConversionSettings, ExportPreset, MediaFile,
};
use frameforge_core::jobs::scheduler::JobScheduler;
use frameforge_core::media::capabilities::build_capability_registry;
use frameforge_core::media::probe::probe_media_file;
use frameforge_core::media::thumbnails::generate_thumbnail;
use frameforge_core::storage::cache::CacheManager;

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub capabilities: RwLock<CapabilityRegistry>,
    pub scheduler: JobScheduler,
    pub cache: CacheManager,
    pub event_tx: mpsc::UnboundedSender<CoreEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFileFailure {
    pub path: String,
    pub error: SerializableError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBatchResult {
    pub imported: Vec<MediaFile>,
    pub failures: Vec<ImportFileFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchStartResult {
    pub started: Vec<ConversionJob>,
    pub failures: Vec<ImportFileFailure>,
}

fn read_capabilities(state: &AppState) -> Result<CapabilityRegistry, SerializableError> {
    state
        .capabilities
        .read()
        .map(|c| c.clone())
        .map_err(|_| FrameForgeError::Internal("Capability lock poisoned".into()).to_serializable())
}

#[tauri::command]
pub fn get_capabilities(
    state: State<'_, Arc<AppState>>,
) -> Result<CapabilityRegistry, SerializableError> {
    read_capabilities(&state)
}

#[tauri::command]
pub async fn refresh_capabilities(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<CapabilityRegistry, SerializableError> {
    let cfg =
        state.config.read().map(|c| c.clone()).map_err(|_| {
            FrameForgeError::Internal("Config lock poisoned".into()).to_serializable()
        })?;
    let caps = build_capability_registry(&cfg).await;
    if let Ok(mut guard) = state.capabilities.write() {
        *guard = caps.clone();
    }
    let _ = app.emit("frameforge://capabilities", &caps);
    Ok(caps)
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> Result<AppConfig, SerializableError> {
    state
        .config
        .read()
        .map(|c| c.clone())
        .map_err(|_| FrameForgeError::Internal("Config lock poisoned".into()).to_serializable())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    config: AppConfig,
) -> Result<CapabilityRegistry, SerializableError> {
    if let Ok(mut guard) = state.config.write() {
        *guard = config.clone();
    }
    let caps = build_capability_registry(&config).await;
    if let Ok(mut guard) = state.capabilities.write() {
        *guard = caps.clone();
    }
    let _ = app.emit("frameforge://capabilities", &caps);
    Ok(caps)
}

#[tauri::command]
pub async fn import_paths(
    state: State<'_, Arc<AppState>>,
    paths: Vec<String>,
) -> Result<ImportBatchResult, SerializableError> {
    let caps = read_capabilities(&state)?;

    let ffprobe_path = caps
        .ffprobe
        .path
        .as_deref()
        .map(PathBuf::from)
        .filter(|_| caps.ffprobe.available)
        .ok_or_else(|| {
            FrameForgeError::BinaryNotFound {
                tool: "ffprobe".to_string(),
                searched: caps
                    .ffprobe
                    .error
                    .clone()
                    .unwrap_or_else(|| "PATH".to_string()),
            }
            .to_serializable()
        })?;

    let ffmpeg_path = caps
        .ffmpeg
        .path
        .as_deref()
        .map(PathBuf::from)
        .filter(|_| caps.ffmpeg.available);

    let file_candidates = tokio::task::spawn_blocking(move || {
        let mut collected: Vec<PathBuf> = Vec::new();
        for raw in paths {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let p = PathBuf::from(trimmed);
            if p.is_dir() {
                collect_media_files_from_dir(&p, 0, &mut collected);
            } else {
                collected.push(p);
            }
        }
        collected
    })
    .await
    .map_err(|e| {
        FrameForgeError::Internal(format!("Directory scan failed: {e}")).to_serializable()
    })?;

    let mut failures: Vec<ImportFileFailure> = Vec::new();
    let thumbs_dir = state.cache.thumbnails_dir();
    let probe_semaphore = Arc::new(Semaphore::new(4));
    let mut handles = Vec::with_capacity(file_candidates.len());

    for candidate in file_candidates {
        let sem = probe_semaphore.clone();
        let ffprobe = ffprobe_path.clone();
        let ffmpeg = ffmpeg_path.clone();
        let tdir = thumbs_dir.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            let path_str = candidate.display().to_string();
            match probe_media_file(&ffprobe, &candidate).await {
                Ok(mut media) => {
                    if let Some(ff) = ffmpeg
                        && let Some((data_url, cache_path)) =
                            generate_thumbnail(&ff, &media, &tdir).await
                    {
                        media.thumbnail_data_url = Some(data_url);
                        media.thumbnail_cache_path = Some(cache_path);
                    }
                    Ok(media)
                }
                Err(err) => Err(ImportFileFailure {
                    path: path_str,
                    error: err.to_serializable(),
                }),
            }
        }));
    }

    let mut imported = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(Ok(media)) => imported.push(media),
            Ok(Err(fail)) => failures.push(fail),
            Err(join_err) => failures.push(ImportFileFailure {
                path: "<task>".to_string(),
                error: FrameForgeError::Internal(join_err.to_string()).to_serializable(),
            }),
        }
    }

    let cache = state.cache.clone();
    let _ = tokio::task::spawn_blocking(move || cache.enforce_budget()).await;
    Ok(ImportBatchResult { imported, failures })
}

#[tauri::command]
pub fn evaluate_conversion(
    state: State<'_, Arc<AppState>>,
    media: MediaFile,
    settings: ConversionSettings,
) -> Result<ConversionPlan, SerializableError> {
    let caps = read_capabilities(&state)?;
    build_conversion_plan(&media, &settings, &caps).map_err(|e| e.to_serializable())
}

#[tauri::command]
pub fn start_conversion(
    state: State<'_, Arc<AppState>>,
    media: MediaFile,
    settings: ConversionSettings,
) -> Result<ConversionJob, SerializableError> {
    let caps = read_capabilities(&state)?;
    state
        .scheduler
        .enqueue_conversion(media, settings, caps, Some(state.event_tx.clone()))
        .map_err(|e| e.to_serializable())
}

#[tauri::command]
pub fn start_batch_conversion(
    state: State<'_, Arc<AppState>>,
    items: Vec<MediaFile>,
    settings: ConversionSettings,
) -> Result<BatchStartResult, SerializableError> {
    let caps = read_capabilities(&state)?;

    let mut started = Vec::new();
    let mut failures = Vec::new();

    for media in items {
        let path = media.path.clone();
        match state.scheduler.enqueue_conversion(
            media,
            settings.clone(),
            caps.clone(),
            Some(state.event_tx.clone()),
        ) {
            Ok(job) => started.push(job),
            Err(err) => failures.push(ImportFileFailure {
                path,
                error: err.to_serializable(),
            }),
        }
    }

    Ok(BatchStartResult { started, failures })
}

#[tauri::command]
pub fn cancel_job(state: State<'_, Arc<AppState>>, job_id: String) -> bool {
    state.scheduler.store.request_cancel(&job_id)
}

#[tauri::command]
pub fn list_jobs(state: State<'_, Arc<AppState>>) -> Vec<ConversionJob> {
    state.scheduler.store.list()
}

#[tauri::command]
pub fn clear_finished_jobs(state: State<'_, Arc<AppState>>) -> usize {
    state.scheduler.store.remove_finished()
}

#[tauri::command]
pub fn get_presets(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ExportPreset>, SerializableError> {
    let caps = read_capabilities(&state)?;
    Ok(builtin_presets(&caps))
}

#[tauri::command]
pub fn open_output_file(app: AppHandle, path: String) -> Result<(), SerializableError> {
    let target = validate_existing_regular_file(&path)?;
    app.opener()
        .open_path(target.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| {
            FrameForgeError::Internal(format!("Failed to open file: {e}")).to_serializable()
        })
}

#[tauri::command]
pub fn reveal_in_folder(app: AppHandle, path: String) -> Result<(), SerializableError> {
    let target = validate_existing_regular_file(&path)?;
    app.opener().reveal_item_in_dir(target).map_err(|e| {
        FrameForgeError::Internal(format!("Failed to reveal file in folder: {e}")).to_serializable()
    })
}

#[tauri::command]
pub async fn export_probe_json(
    path: String,
    json_content: String,
) -> Result<(), SerializableError> {
    let target = PathBuf::from(path.trim());
    if target.as_os_str().is_empty() {
        return Err(FrameForgeError::ValidationError {
            field: "path".to_string(),
            message: "Destination path cannot be empty".to_string(),
        }
        .to_serializable());
    }
    tokio::fs::write(&target, json_content)
        .await
        .map_err(|source| {
            FrameForgeError::Io {
                context: format!("exporting JSON report to {}", target.display()),
                source,
            }
            .to_serializable()
        })
}

fn validate_existing_regular_file(path: &str) -> Result<PathBuf, SerializableError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(FrameForgeError::InvalidInputPath {
            path: path.to_string(),
            reason: "Path is empty".to_string(),
        }
        .to_serializable());
    }
    let target = PathBuf::from(trimmed);
    if !target.exists() || !target.is_file() {
        return Err(FrameForgeError::InvalidInputPath {
            path: path.to_string(),
            reason: "Output file no longer exists on disk".to_string(),
        }
        .to_serializable());
    }
    Ok(target)
}

fn collect_media_files_from_dir(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 3 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.')
            || name.eq_ignore_ascii_case("desktop.ini")
            || name.eq_ignore_ascii_case("thumbs.db")
        {
            continue;
        }
        if p.is_dir() {
            collect_media_files_from_dir(&p, depth + 1, out);
        } else if p.is_file() && looks_like_media_candidate(&p) {
            out.push(p);
        }
    }
}

fn looks_like_media_candidate(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    matches!(
        ext.as_str(),
        "mp4"
            | "mkv"
            | "mov"
            | "avi"
            | "webm"
            | "mpeg"
            | "mpg"
            | "m4v"
            | "flv"
            | "wmv"
            | "asf"
            | "ts"
            | "mts"
            | "m2ts"
            | "mxf"
            | "3gp"
            | "ogv"
            | "aac"
            | "mp3"
            | "wav"
            | "flac"
            | "alac"
            | "opus"
            | "ogg"
            | "m4a"
            | "ac3"
            | "eac3"
            | "jpg"
            | "jpeg"
            | "png"
            | "webp"
            | "tif"
            | "tiff"
            | "bmp"
            | "gif"
            | "avif"
    )
}
