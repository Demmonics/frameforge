import { beforeEach, describe, expect, it } from "vitest";
import { DEFAULT_CONVERSION_SETTINGS, useAppStore } from "./appStore";
import type { ExportPreset, MediaFile } from "../types/media";

const MOCK_MEDIA: MediaFile = {
  id: "media-1",
  fileName: "sample.mp4",
  path: "C:\\Videos\\sample.mp4",
  sizeBytes: 2_048_000,
  container: {
    formatName: "mov,mp4,m4a,3gp,3g2,mj2",
    formatLongName: "QuickTime / MOV",
    probeScore: 100,
  },
  durationSecs: 10,
  bitRate: 1_500_000,
  videoStreams: [],
  audioStreams: [],
  subtitleStreams: [],
  chaptersCount: 0,
  tags: {},
  technicalReport: "REPORT",
  rawProbeJson: "{}",
  probedAtUtc: "2026-01-01T00:00:00Z",
};

describe("useAppStore", () => {
  beforeEach(() => {
    useAppStore.setState({
      mediaItems: [MOCK_MEDIA],
      selectedMediaIds: ["media-1"],
      activeMediaId: "media-1",
      settings: { ...DEFAULT_CONVERSION_SETTINGS },
      notifications: [],
    });
  });

  it("applies preset while preserving user outputDir and manages notifications", () => {
    useAppStore.getState().updateSettings({ outputDir: "D:\\Exports" });

    const preset: ExportPreset = {
      id: "remux-mkv-lossless",
      name: "Stream Copy to MKV",
      category: "Lossless",
      description: "Remux",
      isBuiltin: true,
      available: true,
      settings: {
        ...DEFAULT_CONVERSION_SETTINGS,
        container: "mkv",
        videoCodec: "copy",
        audioMode: "copy",
        presetId: "remux-mkv-lossless",
        presetName: "MKV_Copy",
      },
    };

    useAppStore.getState().applyPreset(preset);
    const state = useAppStore.getState();
    expect(state.settings.container).toBe("mkv");
    expect(state.settings.videoCodec).toBe("copy");
    expect(state.settings.outputDir).toBe("D:\\Exports");

    state.pushNotification({ kind: "info", title: "Ready" });
    expect(useAppStore.getState().notifications).toHaveLength(1);
    const notifId = useAppStore.getState().notifications[0].id;
    useAppStore.getState().dismissNotification(notifId);
    expect(useAppStore.getState().notifications).toHaveLength(0);
  });
});

