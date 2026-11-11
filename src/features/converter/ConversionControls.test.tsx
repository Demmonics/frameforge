import { describe, expect, it, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ConversionControls } from "./ConversionControls";
import { useAppStore, DEFAULT_CONVERSION_SETTINGS } from "../../store/appStore";

describe("ConversionControls", () => {
  beforeEach(() => {
    useAppStore.setState({
      interfaceLevel: "simple",
      settings: { ...DEFAULT_CONVERSION_SETTINGS },
      mediaItems: [],
      selectedMediaIds: [],
      activeMediaId: null,
      plan: null,
      capabilities: {
        ffmpeg: { name: "ffmpeg", available: true },
        ffprobe: { name: "ffprobe", available: true },
        mpv: { name: "mpv", available: false },
        licenseProfile: "GPLv3",
        demuxers: [],
        muxers: [],
        decoders: [],
        encoders: [
          {
            name: "libx264",
            codecFamily: "h264",
            mediaType: "video",
            description: "x264",
            isHardware: false,
            experimental: false,
            frameMt: true,
            sliceMt: true,
          },
        ],
        filters: [],
        hwaccels: [],
        hardwareProbes: [],
        supportedContainers: [
          {
            id: "mp4",
            label: "MP4 (MPEG-4 Part 14)",
            extension: "mp4",
            muxer: "mp4",
            description: "MP4",
            available: true,
            videoCodecs: ["copy", "h264", "hevc"],
            audioCodecs: ["copy", "aac"],
            audioOnly: false,
          },
          {
            id: "mkv",
            label: "MKV (Matroska)",
            extension: "mkv",
            muxer: "matroska",
            description: "MKV",
            available: true,
            videoCodecs: ["copy", "h264", "hevc", "ffv1"],
            audioCodecs: ["copy", "aac", "flac"],
            audioOnly: false,
          },
        ],
        probedAtUtc: "2026-01-01T00:00:00Z",
      },
      presets: [],
    });
  });

  it("switches between Simple, Advanced, and Expert modes and updates container", () => {
    render(<ConversionControls />);

    expect(screen.getByLabelText("Target Format / Container")).toBeTruthy();
    expect(screen.getByLabelText("Conversion Quality Profile")).toBeTruthy();

    fireEvent.click(screen.getByRole("tab", { name: "Advanced" }));
    expect(screen.getByLabelText("Video Codec")).toBeTruthy();
    expect(screen.getByLabelText("Hardware Mode")).toBeTruthy();

    fireEvent.click(screen.getByRole("tab", { name: "Expert" }));
    expect(screen.getByText("Validated Command Preview")).toBeTruthy();

    const containerSelect = screen.getByLabelText("Target Format / Container") as HTMLSelectElement;
    fireEvent.change(containerSelect, { target: { value: "mkv" } });
    expect(useAppStore.getState().settings.container).toBe("mkv");
  });
});

