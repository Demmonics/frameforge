# FRAMEFORGE Architecture

## Workspace Layout

FRAMEFORGE uses a clean separation between its pure Rust media engine (`crates/frameforge-core`), the Tauri 2 desktop shell (`src-tauri`), and the React presentation layer (`src`).

### 1. Core Engine (`crates/frameforge-core/src/`)

- **`core/`**:
  - `errors.rs`: Structured `FrameForgeError` enum and `SerializableError` IPC payload.
  - `config.rs`: `AppConfig`, binary overrides, CPU/GPU worker limits, cache limits, output defaults.
  - `models.rs`: Shared domain types (`MediaFile`, `CapabilityRegistry`, `ConversionSettings`, `ConversionPlan`, `ConversionJob`, `ExportPreset`).
  - `events.rs`: `CoreEvent` stream pushed to the UI (`JobUpdated`, `JobLog`, `CapabilitiesUpdated`).
- **`media/`**:
  - `capabilities.rs`: Runs `ffmpeg -version`, `-demuxers`, `-muxers`, `-decoders`, `-encoders`, `-filters`, and `-hwaccels` asynchronously to construct the runtime capability registry.
  - `formats.rs`: Container definitions (`MP4`, `MKV`, `MOV`, `WebM`, `AVI`, `TS`, `M4A`, `FLAC`, `WAV`, `MP3`) and codec compatibility rules.
  - `probe.rs`: Spawns `ffprobe` safely with `-protocol_whitelist file,pipe` and no shell interpolation; classifies corrupt/empty/encrypted inputs.
  - `metadata.rs`: Parses FFprobe JSON into normalized stream metadata (DAR/SAR, VFR detection, bit depth, HDR PQ/HLG + Mastering Display/MaxCLL, rotation from display matrix or tags) and builds copyable ASCII technical reports.
  - `streams.rs`: Stream selection helpers that locate primary video streams without assuming index `0`.
  - `thumbnails.rs`: Extracts cached 480px-wide JPEG keyframe thumbnails.
- **`hardware/`**:
  - `detection.rs`: Performs concurrent 1-frame null-mux device initialization tests on candidate hardware encoders (`*_nvenc`, `*_qsv`, `*_amf`, `*_mf`) to distinguish installed names from working drivers.
  - `encoders.rs`, `decoders.rs`, `fallback.rs`, `benchmark.rs`: Mode-driven encoder selection (`Auto`, `Cpu`, `Gpu`, `QualityPriority`, `SpeedPriority`) and explicit CPU fallback rules.
- **`conversion/`**:
  - `validation.rs`: Evaluates compatibility and classifies jobs as `Copy`, `Remux`, `Transcode`, or `Fail`, explaining quality impact.
  - `command_builder.rs`: Builds typed `Vec<OsString>` argument lists and validated command previews; rejects unsafe control/shell characters on expert inputs.
  - `profiles.rs`: Built-in presets validated live against `CapabilityRegistry`.
  - `transcoder.rs`: Runs bounded FFmpeg subprocesses with `-progress pipe:1`, bounded stderr ring buffers, cooperative cancellation, and optional explicit CPU fallback.
- **`jobs/`**:
  - `progress.rs`: Parses `out_time_us`, `fps`, `speed`, `bitrate`, `total_size`, and computes ETA.
  - `cancellation.rs`: `CancellationToken` coordination.
  - `queue.rs` & `scheduler.rs`: Semaphore-bounded CPU and GPU worker pools.
  - `persistence.rs`: Trait boundary for Milestone D SQLite persistence.
- **`storage/`**:
  - `temporary_files.rs`: `StagedOutput` writes to `.stem.frameforge-<id>.tmp.<ext>` in the destination directory and performs atomic rename on success (or guaranteed cleanup on drop/cancel/error).
  - `output_manager.rs`: Filename template expansion, sanitization, collision deduplication, and free disk space check.
  - `cache.rs`: Thumbnail cache size enforcement.
  - `history.rs`: Foundation types for Milestone D job history.
- **`platform/`**:
  - `windows.rs` (active), `linux.rs` & `macos.rs` (cross-platform abstractions). Handles `CREATE_NO_WINDOW`, binary discovery, and Win32 `GetDiskFreeSpaceExW` disk checks.
- **`playback/`** & **`editing/`**:
  - Domain boundaries prepared for Milestone E (`mpv/libmpv`) and Milestone G (non-destructive operation recipes).

### 2. Tauri Desktop Shell (`src-tauri/src/`)

- `lib.rs`: Initializes `tracing-subscriber`, initializes `AppState`, runs startup capability probing, and forwards `CoreEvent` updates to the WebView via `frameforge://` events.
- `commands.rs`: Exposes strictly typed IPC commands (`import_paths`, `evaluate_conversion`, `start_conversion`, `start_batch_conversion`, `cancel_job`, `open_output_file`, `reveal_in_folder`, `export_probe_json`, `get_presets`, `get_capabilities`, `refresh_capabilities`, `get_config`, `update_config`).
- `capabilities/default.json`: Restricts frontend permissions to core window/events and native file/folder open/save dialogs.

### 3. React Frontend (`src/`)

- Strictly a presentation & UI-state layer managed with **Zustand** (`src/store/appStore.ts`) and typed Tauri IPC wrappers (`src/lib/tauri.ts`).
- Feature boundaries: `converter/`, `inspector/`, `queue/`, `presets/`, `dashboard/`, `settings/`, `roadmap/`, and `shared/`.

