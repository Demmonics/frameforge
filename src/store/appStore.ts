import { create } from "zustand";
import type {
  AppConfig,
  CapabilityRegistry,
  ConversionInterfaceLevel,
  ConversionJob,
  ConversionPlan,
  ConversionSettings,
  ExportPreset,
  MediaFile,
  SerializableError,
} from "../types/media";
import * as api from "../lib/tauri";

export type AppView =
  | "dashboard"
  | "converter"
  | "player"
  | "inspector"
  | "toolbox"
  | "queue"
  | "presets"
  | "settings";

export interface AppNotification {
  id: string;
  kind: "error" | "warning" | "success" | "info";
  title: string;
  detail?: string;
}

export const DEFAULT_CONVERSION_SETTINGS: ConversionSettings = {
  container: "mp4",
  videoCodec: "h264",
  videoEncoder: null,
  rateControl: "crf",
  qualityValue: 20,
  videoBitrateKbps: 6000,
  encoderPreset: "medium",
  resolution: "original",
  customWidth: 1920,
  customHeight: 1080,
  targetFps: null,
  pixelFormat: null,
  hardwareMode: "auto",
  allowCpuFallback: true,
  videoStreamIndex: null,
  audioMode: "transcode",
  audioCodec: "aac",
  audioBitrateKbps: 192,
  audioChannels: null,
  audioSampleRate: null,
  audioStreamIndices: null,
  subtitleMode: "strip",
  copyMetadata: true,
  faststart: true,
  profile: null,
  level: null,
  tune: null,
  gopSize: null,
  bFrames: null,
  colorRange: null,
  outputDir: null,
  filenamePattern: "{name}_converted",
  overwritePolicy: "renameIfExists",
  presetId: null,
  presetName: null,
};

interface AppStoreState {
  initialized: boolean;
  activeView: AppView;
  interfaceLevel: ConversionInterfaceLevel;
  capabilities: CapabilityRegistry | null;
  config: AppConfig | null;
  presets: ExportPreset[];
  mediaItems: MediaFile[];
  selectedMediaIds: string[];
  activeMediaId: string | null;
  settings: ConversionSettings;
  plan: ConversionPlan | null;
  planLoading: boolean;
  jobs: ConversionJob[];
  selectedJobId: string | null;
  notifications: AppNotification[];
  isImporting: boolean;
  isDraggingFiles: boolean;

  setActiveView: (view: AppView) => void;
  setInterfaceLevel: (level: ConversionInterfaceLevel) => void;
  setDraggingFiles: (dragging: boolean) => void;
  dismissNotification: (id: string) => void;
  pushNotification: (n: Omit<AppNotification, "id">) => void;

  initialize: () => Promise<void>;
  refreshCapabilities: () => Promise<void>;
  saveConfig: (config: AppConfig) => Promise<void>;
  importPaths: (paths: string[]) => Promise<void>;
  selectMedia: (id: string, toggleMulti?: boolean) => void;
  selectAllMedia: () => void;
  removeMedia: (id: string) => void;
  clearMedia: () => void;
  updateSettings: (patch: Partial<ConversionSettings>) => void;
  applyPreset: (preset: ExportPreset) => void;
  evaluateCurrentPlan: () => Promise<void>;
  startActiveConversion: () => Promise<void>;
  startBatchConversion: () => Promise<void>;
  cancelJob: (jobId: string) => Promise<void>;
  clearFinishedJobs: () => Promise<void>;
  setSelectedJobId: (jobId: string | null) => void;
  openOutputFile: (path: string) => Promise<void>;
  revealInFolder: (path: string) => Promise<void>;
  handleJobUpdatedEvent: (job: ConversionJob) => void;
  handleJobLogEvent: (jobId: string, line: string) => void;
  handleCapabilitiesEvent: (caps: CapabilityRegistry) => void;
}

function formatErr(err: unknown): { title: string; detail?: string } {
  if (typeof err === "object" && err !== null && "message" in err) {
    const se = err as SerializableError;
    return {
      title: se.message,
      detail: se.details ?? undefined,
    };
  }
  return { title: err instanceof Error ? err.message : String(err) };
}

export const useAppStore = create<AppStoreState>((set, get) => ({
  initialized: false,
  activeView: "converter",
  interfaceLevel: "simple",
  capabilities: null,
  config: null,
  presets: [],
  mediaItems: [],
  selectedMediaIds: [],
  activeMediaId: null,
  settings: { ...DEFAULT_CONVERSION_SETTINGS },
  plan: null,
  planLoading: false,
  jobs: [],
  selectedJobId: null,
  notifications: [],
  isImporting: false,
  isDraggingFiles: false,

  setActiveView: (view) => set({ activeView: view }),
  setInterfaceLevel: (level) => set({ interfaceLevel: level }),
  setDraggingFiles: (dragging) => set({ isDraggingFiles: dragging }),
  setSelectedJobId: (jobId) => set({ selectedJobId: jobId }),

  dismissNotification: (id) =>
    set((s) => ({ notifications: s.notifications.filter((n) => n.id !== id) })),

  pushNotification: (n) => {
    const id = `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
    set((s) => ({
      notifications: [{ id, ...n }, ...s.notifications].slice(0, 6),
    }));
  },

  initialize: async () => {
    if (!api.isTauriRuntime()) {
      set({ initialized: true });
      return;
    }
    try {
      const [caps, cfg, presets, jobs] = await Promise.all([
        api.getCapabilities(),
        api.getConfig(),
        api.getPresets(),
        api.listJobs(),
      ]);
      set({
        initialized: true,
        capabilities: caps,
        config: cfg,
        presets,
        jobs,
      });
      if (!caps.ffmpeg.available || !caps.ffprobe.available) {
        get().pushNotification({
          kind: "error",
          title: "FFmpeg / FFprobe binary missing",
          detail: "Configure valid FFmpeg and FFprobe paths in Settings or install them on PATH.",
        });
      }
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
      set({ initialized: true });
    }
  },

  refreshCapabilities: async () => {
    try {
      const caps = await api.refreshCapabilities();
      const presets = await api.getPresets();
      set({ capabilities: caps, presets });
      await get().evaluateCurrentPlan();
      get().pushNotification({
        kind: "success",
        title: "FFmpeg capability registry refreshed",
      });
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  saveConfig: async (config) => {
    try {
      const caps = await api.updateConfig(config);
      const presets = await api.getPresets();
      set({ config, capabilities: caps, presets });
      await get().evaluateCurrentPlan();
      get().pushNotification({
        kind: "success",
        title: "Settings updated",
      });
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  importPaths: async (paths) => {
    if (paths.length === 0) return;
    set({ isImporting: true });
    try {
      const result = await api.importPaths(paths);
      set((s) => {
        const existingByPath = new Map(s.mediaItems.map((m) => [m.path, m]));
        for (const item of result.imported) {
          existingByPath.set(item.path, item);
        }
        const merged = Array.from(existingByPath.values());
        const newActiveId =
          result.imported[0]?.id ?? s.activeMediaId ?? merged[0]?.id ?? null;
        const selected =
          s.selectedMediaIds.length > 0
            ? s.selectedMediaIds
            : newActiveId
            ? [newActiveId]
            : [];
        return {
          mediaItems: merged,
          activeMediaId: newActiveId,
          selectedMediaIds: selected,
          isImporting: false,
        };
      });

      for (const fail of result.failures) {
        get().pushNotification({
          kind: "error",
          title: `Failed to import ${fail.path.split(/[/\\]/).pop() ?? fail.path}`,
          detail: `${fail.error.message}${fail.error.details ? `\n${fail.error.details}` : ""}`,
        });
      }

      if (result.imported.length > 0) {
        await get().evaluateCurrentPlan();
      }
    } catch (err) {
      set({ isImporting: false });
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  selectMedia: (id, toggleMulti = false) => {
    set((s) => {
      if (!toggleMulti) {
        return { activeMediaId: id, selectedMediaIds: [id] };
      }
      const exists = s.selectedMediaIds.includes(id);
      const nextSelected = exists
        ? s.selectedMediaIds.filter((x) => x !== id)
        : [...s.selectedMediaIds, id];
      const nextActive = exists
        ? nextSelected[0] ?? null
        : id;
      return { activeMediaId: nextActive, selectedMediaIds: nextSelected };
    });
    void get().evaluateCurrentPlan();
  },

  selectAllMedia: () => {
    set((s) => ({
      selectedMediaIds: s.mediaItems.map((m) => m.id),
      activeMediaId: s.activeMediaId ?? s.mediaItems[0]?.id ?? null,
    }));
  },

  removeMedia: (id) => {
    set((s) => {
      const remaining = s.mediaItems.filter((m) => m.id !== id);
      const selected = s.selectedMediaIds.filter((x) => x !== id);
      const active =
        s.activeMediaId === id ? remaining[0]?.id ?? null : s.activeMediaId;
      return {
        mediaItems: remaining,
        selectedMediaIds: selected.length > 0 ? selected : active ? [active] : [],
        activeMediaId: active,
        plan: remaining.length === 0 ? null : s.plan,
      };
    });
    void get().evaluateCurrentPlan();
  },

  clearMedia: () => {
    set({
      mediaItems: [],
      selectedMediaIds: [],
      activeMediaId: null,
      plan: null,
    });
  },

  updateSettings: (patch) => {
    set((s) => ({
      settings: {
        ...s.settings,
        ...patch,
        presetId: patch.presetId !== undefined ? patch.presetId : null,
      },
    }));
    void get().evaluateCurrentPlan();
  },

  applyPreset: (preset) => {
    set((s) => ({
      settings: {
        ...preset.settings,
        outputDir: s.settings.outputDir ?? preset.settings.outputDir ?? null,
      },
    }));
    void get().evaluateCurrentPlan();
  },

  evaluateCurrentPlan: async () => {
    const { mediaItems, activeMediaId, settings } = get();
    const media = mediaItems.find((m) => m.id === activeMediaId);
    if (!media || !api.isTauriRuntime()) {
      set({ plan: null, planLoading: false });
      return;
    }
    set({ planLoading: true });
    try {
      const plan = await api.evaluateConversion(media, settings);
      set({ plan, planLoading: false });
    } catch (err) {
      set({ planLoading: false });
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  startActiveConversion: async () => {
    const { mediaItems, activeMediaId, settings } = get();
    const media = mediaItems.find((m) => m.id === activeMediaId);
    if (!media) return;
    try {
      const job = await api.startConversion(media, settings);
      set((s) => ({
        jobs: [job, ...s.jobs.filter((j) => j.id !== job.id)],
        selectedJobId: job.id,
      }));
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({
        kind: "error",
        title: "Cannot start conversion",
        detail: `${info.title}${info.detail ? `\n${info.detail}` : ""}`,
      });
    }
  },

  startBatchConversion: async () => {
    const { mediaItems, selectedMediaIds, settings } = get();
    const targets =
      selectedMediaIds.length > 0
        ? mediaItems.filter((m) => selectedMediaIds.includes(m.id))
        : mediaItems;
    if (targets.length === 0) return;

    try {
      const res = await api.startBatchConversion(targets, settings);
      if (res.started.length > 0) {
        set((s) => {
          const existing = new Map(s.jobs.map((j) => [j.id, j]));
          for (const j of res.started) {
            existing.set(j.id, j);
          }
          return {
            jobs: Array.from(existing.values()),
            selectedJobId: res.started[0].id,
          };
        });
      }
      for (const fail of res.failures) {
        get().pushNotification({
          kind: "error",
          title: `Skipped ${fail.path.split(/[/\\]/).pop()}`,
          detail: fail.error.message,
        });
      }
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  cancelJob: async (jobId) => {
    try {
      await api.cancelJob(jobId);
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  clearFinishedJobs: async () => {
    try {
      await api.clearFinishedJobs();
      const jobs = await api.listJobs();
      set({ jobs });
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  openOutputFile: async (path) => {
    try {
      await api.openOutputFile(path);
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  revealInFolder: async (path) => {
    try {
      await api.revealInFolder(path);
    } catch (err) {
      const info = formatErr(err);
      get().pushNotification({ kind: "error", title: info.title, detail: info.detail });
    }
  },

  handleJobUpdatedEvent: (job) => {
    set((s) => {
      const idx = s.jobs.findIndex((j) => j.id === job.id);
      if (idx === -1) {
        return { jobs: [job, ...s.jobs] };
      }
      const next = [...s.jobs];
      next[idx] = job;
      return { jobs: next };
    });
  },

  handleJobLogEvent: (jobId, line) => {
    set((s) => {
      const idx = s.jobs.findIndex((j) => j.id === jobId);
      if (idx === -1) return s;
      const existing = s.jobs[idx];
      if (existing.logs[existing.logs.length - 1] === line) return s;
      const next = [...s.jobs];
      next[idx] = {
        ...existing,
        logs: [...existing.logs.slice(-399), line],
      };
      return { jobs: next };
    });
  },

  handleCapabilitiesEvent: (caps) => {
    set({ capabilities: caps });
  },
}));

