mod commands;

use std::sync::{Arc, RwLock};
use tauri::{Emitter, Manager};
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use frameforge_core::core::config::{AppConfig, ConcurrencyConfig};
use frameforge_core::core::events::CoreEvent;
use frameforge_core::jobs::scheduler::JobScheduler;
use frameforge_core::media::capabilities::build_capability_registry;
use frameforge_core::storage::cache::CacheManager;

use crate::commands::AppState;

/// Launches the desktop Tauri application runtime and initializes background capability probing.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let cache_dir = app
                .path()
                .app_cache_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("frameforge-cache"));

            let config = AppConfig {
                cache_dir: cache_dir.clone(),
                ..AppConfig::default()
            };

            let cache = CacheManager::new(cache_dir, config.max_cache_bytes)?;

            let initial_caps = tauri::async_runtime::block_on(build_capability_registry(&config));
            tracing::info!(
                ffmpeg_available = initial_caps.ffmpeg.available,
                ffprobe_available = initial_caps.ffprobe.available,
                hw_probes = initial_caps.hardware_probes.len(),
                "Initialized FrameForge capability registry"
            );

            let (event_tx, mut event_rx) = mpsc::unbounded_channel::<CoreEvent>();
            let state = Arc::new(AppState {
                config: RwLock::new(config),
                capabilities: RwLock::new(initial_caps),
                scheduler: JobScheduler::new(&ConcurrencyConfig::default()),
                cache,
                event_tx,
            });

            app.manage(state);

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some(ev) = event_rx.recv().await {
                    match ev {
                        CoreEvent::JobUpdated(job) => {
                            let _ = handle.emit("frameforge://job-updated", &*job);
                        }
                        CoreEvent::JobLog { job_id, line } => {
                            let _ = handle.emit(
                                "frameforge://job-log",
                                serde_json::json!({ "jobId": job_id, "line": line }),
                            );
                        }
                        CoreEvent::CapabilitiesUpdated(caps) => {
                            let _ = handle.emit("frameforge://capabilities", &*caps);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_capabilities,
            commands::refresh_capabilities,
            commands::get_config,
            commands::update_config,
            commands::import_paths,
            commands::evaluate_conversion,
            commands::start_conversion,
            commands::start_batch_conversion,
            commands::cancel_job,
            commands::list_jobs,
            commands::clear_finished_jobs,
            commands::get_presets,
            commands::open_output_file,
            commands::reveal_in_folder,
            commands::export_probe_json,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        tracing::error!(error = %err, "Failed while running FrameForge Tauri application");
        std::process::exit(1);
    }
}
