import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  AppConfig,
  BatchStartResult,
  CapabilityRegistry,
  ConversionJob,
  ConversionPlan,
  ConversionSettings,
  ExportPreset,
  ImportBatchResult,
  MediaFile,
} from "../types/media";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function getCapabilities(): Promise<CapabilityRegistry> {
  return invoke<CapabilityRegistry>("get_capabilities");
}

export async function refreshCapabilities(): Promise<CapabilityRegistry> {
  return invoke<CapabilityRegistry>("refresh_capabilities");
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function updateConfig(config: AppConfig): Promise<CapabilityRegistry> {
  return invoke<CapabilityRegistry>("update_config", { config });
}

export async function importPaths(paths: string[]): Promise<ImportBatchResult> {
  return invoke<ImportBatchResult>("import_paths", { paths });
}

export async function evaluateConversion(
  media: MediaFile,
  settings: ConversionSettings
): Promise<ConversionPlan> {
  return invoke<ConversionPlan>("evaluate_conversion", { media, settings });
}

export async function startConversion(
  media: MediaFile,
  settings: ConversionSettings
): Promise<ConversionJob> {
  return invoke<ConversionJob>("start_conversion", { media, settings });
}

export async function startBatchConversion(
  items: MediaFile[],
  settings: ConversionSettings
): Promise<BatchStartResult> {
  return invoke<BatchStartResult>("start_batch_conversion", { items, settings });
}

export async function cancelJob(jobId: string): Promise<boolean> {
  return invoke<boolean>("cancel_job", { jobId });
}

export async function listJobs(): Promise<ConversionJob[]> {
  return invoke<ConversionJob[]>("list_jobs");
}

export async function clearFinishedJobs(): Promise<number> {
  return invoke<number>("clear_finished_jobs");
}

export async function getPresets(): Promise<ExportPreset[]> {
  return invoke<ExportPreset[]>("get_presets");
}

export async function openOutputFile(path: string): Promise<void> {
  return invoke<void>("open_output_file", { path });
}

export async function revealInFolder(path: string): Promise<void> {
  return invoke<void>("reveal_in_folder", { path });
}

export async function exportProbeJson(path: string, jsonContent: string): Promise<void> {
  return invoke<void>("export_probe_json", { path, jsonContent });
}

export async function pickMediaFiles(): Promise<string[]> {
  const result = await open({
    multiple: true,
    title: "Import Media Files",
    filters: [
      {
        name: "Media Files",
        extensions: [
          "mp4", "mkv", "mov", "avi", "webm", "mpeg", "mpg", "m4v", "flv", "wmv",
          "asf", "ts", "mts", "m2ts", "mxf", "3gp", "ogv", "mp3", "wav", "flac",
          "aac", "m4a", "opus", "ogg", "ac3", "png", "jpg", "jpeg", "webp", "tif",
          "tiff", "bmp", "gif", "avif",
        ],
      },
      { name: "All Files", extensions: ["*"] },
    ],
  });
  if (!result) return [];
  return Array.isArray(result) ? result : [result];
}

export async function pickMediaFolder(): Promise<string[]> {
  const result = await open({
    directory: true,
    multiple: true,
    title: "Import Media Folder",
  });
  if (!result) return [];
  return Array.isArray(result) ? result : [result];
}

export async function pickOutputDirectory(): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
    title: "Choose Output Directory",
  });
  return typeof result === "string" ? result : null;
}

export async function pickSaveJsonPath(defaultName: string): Promise<string | null> {
  return save({
    defaultPath: defaultName,
    title: "Export FFprobe JSON Report",
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
}

export async function subscribeAppEvents(handlers: {
  onJobUpdated: (job: ConversionJob) => void;
  onJobLog: (payload: { jobId: string; line: string }) => void;
  onCapabilities: (caps: CapabilityRegistry) => void;
  onDragEnter: () => void;
  onDragLeave: () => void;
  onDropPaths: (paths: string[]) => void;
}): Promise<UnlistenFn> {
  const u1 = await listen<ConversionJob>("frameforge://job-updated", (e) => {
    handlers.onJobUpdated(e.payload);
  });
  const u2 = await listen<{ jobId: string; line: string }>("frameforge://job-log", (e) => {
    handlers.onJobLog(e.payload);
  });
  const u3 = await listen<CapabilityRegistry>("frameforge://capabilities", (e) => {
    handlers.onCapabilities(e.payload);
  });
  const u4 = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === "enter" || event.payload.type === "over") {
      handlers.onDragEnter();
    } else if (event.payload.type === "leave") {
      handlers.onDragLeave();
    } else if (event.payload.type === "drop") {
      handlers.onDragLeave();
      if (event.payload.paths && event.payload.paths.length > 0) {
        handlers.onDropPaths(event.payload.paths);
      }
    }
  });

  return () => {
    u1();
    u2();
    u3();
    u4();
  };
}

