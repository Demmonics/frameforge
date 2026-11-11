import { describe, expect, it } from "vitest";
import { fileBaseName, formatBitrate, formatBytes, formatDuration, formatEta, formatFps } from "./format";

describe("format utilities", () => {
  it("formats byte sizes cleanly", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(1048576)).toBe("1.00 MB");
    expect(formatBytes(null)).toBe("—");
  });

  it("formats durations, bitrates, eta, and fps", () => {
    expect(formatDuration(65.5)).toBe("01:05.50");
    expect(formatDuration(3661)).toBe("1:01:01");
    expect(formatBitrate(5_500_000)).toBe("5.50 Mbps");
    expect(formatBitrate(192_000)).toBe("192 kbps");
    expect(formatEta(75)).toBe("1m 15s");
    expect(formatFps(29.97)).toBe("29.97 fps");
    expect(fileBaseName("C:\\Videos\\demo\\test clip.mp4")).toBe("test clip.mp4");
  });
});

