import { useEffect, useState } from "react";
import { RefreshCw, Save, ShieldAlert } from "lucide-react";
import { useAppStore } from "../../store/appStore";
import { useTheme, type ThemePreference } from "../shared/ThemeContext";

export function SettingsView() {
  const config = useAppStore((s) => s.config);
  const capabilities = useAppStore((s) => s.capabilities);
  const saveConfig = useAppStore((s) => s.saveConfig);
  const refreshCapabilities = useAppStore((s) => s.refreshCapabilities);

  const { preference, systemResolved, setPreference } = useTheme();

  const [ffmpegPath, setFfmpegPath] = useState("");
  const [ffprobePath, setFfprobePath] = useState("");
  const [mpvPath, setMpvPath] = useState("");
  const [maxCpu, setMaxCpu] = useState(2);
  const [maxGpu, setMaxGpu] = useState(2);

  useEffect(() => {
    if (!config) return;
    setFfmpegPath(config.binaries.ffmpegOverride ?? "");
    setFfprobePath(config.binaries.ffprobeOverride ?? "");
    setMpvPath(config.binaries.mpvOverride ?? "");
    setMaxCpu(config.concurrency.maxCpuJobs);
    setMaxGpu(config.concurrency.maxGpuJobs);
  }, [config]);

  const handleSave = async () => {
    if (!config) return;
    await saveConfig({
      ...config,
      binaries: {
        ffmpegOverride: ffmpegPath.trim() || null,
        ffprobeOverride: ffprobePath.trim() || null,
        mpvOverride: mpvPath.trim() || null,
      },
      concurrency: {
        maxCpuJobs: Math.max(1, maxCpu),
        maxGpuJobs: Math.max(1, maxGpu),
      },
    });
  };

  return (
    <div className="flex h-full min-h-0 flex-col overflow-y-auto p-6">
      <div className="flex items-center justify-between border-b pb-4" style={{ borderColor: "var(--border-subtle)" }}>
        <div>
          <h1 className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>
            Settings &amp; Engine Diagnostics
          </h1>
          <p className="mt-0.5 text-xs" style={{ color: "var(--text-secondary)" }}>
            Configure appearance, media binary discovery, worker concurrency, and inspect build licensing.
          </p>
        </div>

        <button
          type="button"
          onClick={refreshCapabilities}
          className="inline-flex min-h-8 items-center gap-1.5 rounded-md border px-3 py-1.5 text-xs font-medium"
          style={{
            backgroundColor: "var(--bg-panel)",
            borderColor: "var(--border-subtle)",
            color: "var(--text-primary)",
          }}
        >
          <RefreshCw className="h-3.5 w-3.5" aria-hidden="true" />
          Re-probe FFmpeg
        </button>
      </div>

      <div className="mt-5 grid max-w-4xl gap-5">
        {/* Theme Section (3-state explicit control) */}
        <section className="rounded-xl border p-4" style={{ backgroundColor: "var(--bg-panel)", borderColor: "var(--border-subtle)" }}>
          <h2 className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
            Color Scheme
          </h2>
          <p className="mt-1 text-xs" style={{ color: "var(--text-muted)" }}>
            Follows your OS color scheme by default or pins an explicit theme.
          </p>

          <div className="mt-3 flex flex-wrap gap-2" role="radiogroup" aria-label="Color Scheme Preference">
            {(
              [
                { id: "system", label: `System (Currently ${systemResolved})` },
                { id: "dark", label: "Dark" },
                { id: "light", label: "Light" },
              ] as { id: ThemePreference; label: string }[]
            ).map((opt) => {
              const active = preference === opt.id;
              return (
                <button
                  key={opt.id}
                  type="button"
                  role="radio"
                  aria-checked={active}
                  onClick={() => setPreference(opt.id)}
                  className="min-h-8 rounded-lg border px-3.5 py-1.5 text-xs font-semibold"
                  style={{
                    backgroundColor: active ? "var(--accent-amber-soft)" : "var(--bg-elevated)",
                    borderColor: active ? "var(--accent-amber)" : "var(--border-subtle)",
                    color: active ? "var(--accent-amber)" : "var(--text-secondary)",
                  }}
                >
                  {opt.label}
                </button>
              );
            })}
          </div>
        </section>

        {/* Binary Paths & Concurrency */}
        <section className="space-y-4 rounded-xl border p-4" style={{ backgroundColor: "var(--bg-panel)", borderColor: "var(--border-subtle)" }}>
          <h2 className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
            Media Binaries &amp; Worker Limits
          </h2>

          <div className="grid gap-3 md:grid-cols-2">
            <div>
              <label htmlFor="cfg-ffmpeg" className="block text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
                FFmpeg Override Path (optional)
              </label>
              <input
                id="cfg-ffmpeg"
                type="text"
                value={ffmpegPath}
                onChange={(e) => setFfmpegPath(e.target.value)}
                placeholder={capabilities?.ffmpeg.path ?? "Auto-discovered from PATH"}
                className="ff-input mt-1 font-mono-tech"
              />
            </div>

            <div>
              <label htmlFor="cfg-ffprobe" className="block text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
                FFprobe Override Path (optional)
              </label>
              <input
                id="cfg-ffprobe"
                type="text"
                value={ffprobePath}
                onChange={(e) => setFfprobePath(e.target.value)}
                placeholder={capabilities?.ffprobe.path ?? "Auto-discovered from PATH"}
                className="ff-input mt-1 font-mono-tech"
              />
            </div>

            <div>
              <label htmlFor="cfg-mpv" className="block text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
                mpv Override Path (Milestone E)
              </label>
              <input
                id="cfg-mpv"
                type="text"
                value={mpvPath}
                onChange={(e) => setMpvPath(e.target.value)}
                placeholder={capabilities?.mpv.path ?? "Not detected on PATH"}
                className="ff-input mt-1 font-mono-tech"
              />
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div>
                <label htmlFor="cfg-cpu" className="block text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
                  Max CPU Workers
                </label>
                <input
                  id="cfg-cpu"
                  type="number"
                  min={1}
                  max={8}
                  value={maxCpu}
                  onChange={(e) => setMaxCpu(Number(e.target.value) || 1)}
                  className="ff-input tabular-nums mt-1"
                />
              </div>
              <div>
                <label htmlFor="cfg-gpu" className="block text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
                  Max GPU Workers
                </label>
                <input
                  id="cfg-gpu"
                  type="number"
                  min={1}
                  max={8}
                  value={maxGpu}
                  onChange={(e) => setMaxGpu(Number(e.target.value) || 1)}
                  className="ff-input tabular-nums mt-1"
                />
              </div>
            </div>
          </div>

          <button
            type="button"
            onClick={handleSave}
            className="inline-flex min-h-9 items-center gap-1.5 rounded-lg px-4 py-2 text-xs font-semibold"
            style={{
              backgroundColor: "var(--accent-amber)",
              color: "var(--accent-amber-fg)",
            }}
          >
            <Save className="h-3.5 w-3.5" aria-hidden="true" />
            Save Settings
          </button>
        </section>

        {/* Licensing & Redistribution Audit */}
        <section className="rounded-xl border p-4" style={{ backgroundColor: "var(--bg-panel)", borderColor: "var(--border-subtle)" }}>
          <div className="flex items-center gap-2">
            <ShieldAlert className="h-4 w-4" style={{ color: "var(--accent-cyan)" }} aria-hidden="true" />
            <h2 className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
              Binary Versions &amp; Licensing Audit
            </h2>
          </div>

          <dl className="mt-3 space-y-2 text-xs">
            <div className="flex flex-wrap justify-between gap-2 border-b pb-1.5" style={{ borderColor: "var(--border-subtle)" }}>
              <dt style={{ color: "var(--text-muted)" }}>Active FFmpeg Binary</dt>
              <dd className="font-mono-tech" style={{ color: "var(--text-primary)" }}>
                {capabilities?.ffmpeg.path ?? "Not found"}
              </dd>
            </div>
            <div className="flex flex-wrap justify-between gap-2 border-b pb-1.5" style={{ borderColor: "var(--border-subtle)" }}>
              <dt style={{ color: "var(--text-muted)" }}>Detected License Profile</dt>
              <dd className="font-semibold" style={{ color: "var(--status-warn)" }}>
                {capabilities?.licenseProfile ?? "Unknown"}
              </dd>
            </div>
            <div className="flex flex-wrap justify-between gap-2">
              <dt style={{ color: "var(--text-muted)" }}>Detected Hardware Backends</dt>
              <dd className="font-mono-tech" style={{ color: "var(--text-primary)" }}>
                {capabilities?.hwaccels.join(", ") || "None"}
              </dd>
            </div>
          </dl>
        </section>
      </div>
    </div>
  );
}

