import { Construction, Play, Wrench } from "lucide-react";
import { useAppStore } from "../../store/appStore";

export function FutureModulesView({ kind }: { kind: "player" | "toolbox" }) {
  const capabilities = useAppStore((s) => s.capabilities);

  const isPlayer = kind === "player";

  return (
    <div className="flex h-full flex-col items-center justify-center p-8 text-center">
      <div
        className="flex h-16 w-16 items-center justify-center rounded-2xl border"
        style={{
          backgroundColor: "var(--bg-panel)",
          borderColor: "var(--border-strong)",
          color: "var(--accent-cyan)",
        }}
      >
        {isPlayer ? (
          <Play className="h-7 w-7" aria-hidden="true" />
        ) : (
          <Wrench className="h-7 w-7" aria-hidden="true" />
        )}
      </div>

      <span
        className="mt-4 inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-semibold"
        style={{
          borderColor: "var(--accent-amber)",
          backgroundColor: "var(--accent-amber-soft)",
          color: "var(--accent-amber)",
        }}
      >
        <Construction className="h-3.5 w-3.5" aria-hidden="true" />
        {isPlayer ? "Scheduled for Milestone E" : "Scheduled for Milestone G"}
      </span>

      <h1 className="mt-3 text-xl font-bold" style={{ color: "var(--text-primary)" }}>
        {isPlayer ? "Universal Media Player (mpv / libmpv)" : "Non-Destructive Editor Toolbox"}
      </h1>

      <p className="mt-2 max-w-lg text-xs leading-relaxed" style={{ color: "var(--text-secondary)" }}>
        {isPlayer
          ? "FRAMEFORGE never fakes universal codec playback with an HTML5 <video> element. Native mpv/libmpv IPC and embedded playback integration are intentionally scoped to Milestone E."
          : "Independent non-destructive FFmpeg utility operations (timecode trim, concat, crop, rotate, speed, loudness normalization, GIF/image sequences, proxies) are scheduled for Milestone G after persistent queuing and hardware benchmarking."}
      </p>

      <div
        className="mt-5 w-full max-w-md rounded-xl border p-4 text-left text-xs"
        style={{
          backgroundColor: "var(--bg-panel)",
          borderColor: "var(--border-subtle)",
        }}
      >
        <p className="font-semibold" style={{ color: "var(--text-primary)" }}>
          Current System Readiness
        </p>
        {isPlayer ? (
          <p className="mt-1.5" style={{ color: "var(--text-secondary)" }}>
            mpv executable status:{" "}
            <span className="font-mono-tech font-semibold">
              {capabilities?.mpv.available
                ? `Detected at ${capabilities.mpv.path} (${capabilities.mpv.version ?? "ok"})`
                : "Not installed on PATH (configure path in Settings when installing mpv for Milestone E)"}
            </span>
          </p>
        ) : (
          <p className="mt-1.5" style={{ color: "var(--text-secondary)" }}>
            FFmpeg filtergraph readiness:{" "}
            <span className="font-mono-tech font-semibold">
              {capabilities?.filters.length ?? 0} live filters discovered from installed FFmpeg.
            </span>
          </p>
        )}
      </div>
    </div>
  );
}

