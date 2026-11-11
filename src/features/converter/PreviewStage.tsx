import { AlertTriangle, FileSearch, Film, Music, Sparkles } from "lucide-react";
import { useAppStore } from "../../store/appStore";
import { formatBitrate, formatBytes, formatDuration, formatFps } from "../../lib/format";

export function PreviewStage() {
  const mediaItems = useAppStore((s) => s.mediaItems);
  const activeMediaId = useAppStore((s) => s.activeMediaId);
  const setActiveView = useAppStore((s) => s.setActiveView);

  const media = mediaItems.find((m) => m.id === activeMediaId) ?? null;

  if (!media) {
    return (
      <section
        aria-label="Media Preview Stage"
        className="flex min-h-0 flex-col items-center justify-center p-8 text-center"
        style={{ backgroundColor: "var(--bg-app)" }}
      >
        <div
          className="flex h-16 w-16 items-center justify-center rounded-2xl border"
          style={{
            backgroundColor: "var(--bg-panel)",
            borderColor: "var(--border-subtle)",
            color: "var(--text-muted)",
          }}
        >
          <Film className="h-7 w-7" aria-hidden="true" />
        </div>
        <h1 className="mt-4 text-base font-semibold" style={{ color: "var(--text-primary)" }}>
          No Media Selected
        </h1>
        <p className="mt-1 max-w-sm text-xs leading-relaxed" style={{ color: "var(--text-muted)" }}>
          Import a video or audio file via drag-and-drop or the Media Bin to inspect streams with FFprobe and configure conversion.
        </p>
      </section>
    );
  }

  const v = media.primaryVideo;
  const a = media.primaryAudio;

  return (
    <section
      aria-label="Media Preview and Summary"
      className="flex min-h-0 flex-col overflow-y-auto p-4"
      style={{ backgroundColor: "var(--bg-app)" }}
    >
      {/* Header Row */}
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <h1
            className="truncate text-base font-semibold"
            title={media.fileName}
            style={{ color: "var(--text-primary)" }}
          >
            {media.fileName}
          </h1>
          <p
            className="mt-0.5 truncate font-mono-tech text-[11px]"
            title={media.path}
            style={{ color: "var(--text-muted)" }}
          >
            {media.path}
          </p>
        </div>

        <button
          type="button"
          onClick={() => setActiveView("inspector")}
          className="inline-flex min-h-8 shrink-0 items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs font-medium"
          style={{
            backgroundColor: "var(--bg-panel)",
            borderColor: "var(--border-subtle)",
            color: "var(--text-primary)",
          }}
        >
          <FileSearch className="h-3.5 w-3.5" aria-hidden="true" />
          Full FFprobe Report
        </button>
      </div>

      {/* Preview Frame Stage */}
      <div
        className="mt-3 flex aspect-video max-h-64 w-full items-center justify-center overflow-hidden rounded-xl border"
        style={{
          backgroundColor: "#07090d",
          borderColor: "var(--border-subtle)",
        }}
      >
        {media.thumbnailDataUrl ? (
          <img
            src={media.thumbnailDataUrl}
            alt={`Extracted keyframe preview for ${media.fileName}`}
            className="h-full w-full object-contain"
          />
        ) : (
          <div className="flex flex-col items-center gap-2 text-center">
            {v ? (
              <Film className="h-10 w-10 text-slate-500" aria-hidden="true" />
            ) : (
              <Music className="h-10 w-10 text-sky-400" aria-hidden="true" />
            )}
            <span className="text-xs text-slate-400">
              {v ? "Thumbnail unavailable for stream" : "Audio-Only Source"}
            </span>
          </div>
        )}
      </div>

      {/* Alert Banners for VFR / HDR / Rotation */}
      {(v?.isVfr || v?.hdrFormat || (v?.rotationDegrees != null && v.rotationDegrees !== 0)) && (
        <div className="mt-3 flex flex-col gap-1.5">
          {v.isVfr && (
            <div
              className="flex items-center gap-2 rounded-lg border px-3 py-2 text-xs"
              style={{
                backgroundColor: "var(--status-warn-soft)",
                borderColor: "var(--status-warn)",
                color: "var(--text-primary)",
              }}
            >
              <AlertTriangle className="h-4 w-4 shrink-0" style={{ color: "var(--status-warn)" }} aria-hidden="true" />
              <span>{v.vfrNote ?? "Variable frame rate detected."}</span>
            </div>
          )}
          {v.hdrFormat && (
            <div
              className="flex items-center gap-2 rounded-lg border px-3 py-2 text-xs"
              style={{
                backgroundColor: "var(--accent-cyan-soft)",
                borderColor: "var(--accent-cyan)",
                color: "var(--text-primary)",
              }}
            >
              <Sparkles className="h-4 w-4 shrink-0" style={{ color: "var(--accent-cyan)" }} aria-hidden="true" />
              <span>
                {v.hdrFormat}
                {v.colorPrimaries ? ` · ${v.colorPrimaries}` : ""}
                {v.hdrMetadata?.maxCll ? ` · MaxCLL ${v.hdrMetadata.maxCll} nits` : ""}
              </span>
            </div>
          )}
          {v.rotationDegrees != null && v.rotationDegrees !== 0 && (
            <div
              className="flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs"
              style={{
                backgroundColor: "var(--bg-panel)",
                borderColor: "var(--border-subtle)",
                color: "var(--text-secondary)",
              }}
            >
              <span>Display Matrix Rotation: {v.rotationDegrees}°</span>
            </div>
          )}
        </div>
      )}

      {/* Key Telemetry Cards */}
      <div className="mt-3 grid grid-cols-2 gap-2.5 xl:grid-cols-3">
        <MetricCard label="Container" value={media.container.formatName.split(",")[0].toUpperCase()} sub={media.container.formatLongName} />
        <MetricCard label="Duration" value={formatDuration(media.durationSecs)} sub={formatBytes(media.sizeBytes)} />
        <MetricCard label="Overall Bitrate" value={formatBitrate(media.bitRate)} sub={`${media.videoStreams.length}V · ${media.audioStreams.length}A · ${media.subtitleStreams.length}S`} />
        {v && (
          <>
            <MetricCard
              label={`Video #${v.index}`}
              value={`${v.width} × ${v.height}`}
              sub={`${v.codecName.toUpperCase()}${v.profile ? ` (${v.profile})` : ""}`}
            />
            <MetricCard
              label="Frame Rate"
              value={formatFps(v.fpsNominal)}
              sub={v.isVfr ? `Avg ${formatFps(v.fpsAverage)}` : "Constant (CFR)"}
            />
            <MetricCard
              label="Color & Bit Depth"
              value={`${v.bitDepth ?? 8}-bit`}
              sub={v.pixFmt ?? "unknown"}
            />
          </>
        )}
        {a && (
          <MetricCard
            label={`Audio #${a.index}`}
            value={a.codecName.toUpperCase()}
            sub={`${a.sampleRate ? `${a.sampleRate} Hz` : "—"} · ${a.channelLayout ?? `${a.channels ?? "?"}ch`}`}
          />
        )}
      </div>
    </section>
  );
}

function MetricCard({ label, value, sub }: { label: string; value: string; sub: string }) {
  return (
    <div
      className="rounded-lg border px-3 py-2"
      style={{
        backgroundColor: "var(--bg-panel)",
        borderColor: "var(--border-subtle)",
      }}
    >
      <p className="text-[11px] font-medium" style={{ color: "var(--text-muted)" }}>
        {label}
      </p>
      <p className="tabular-nums mt-0.5 truncate text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
        {value}
      </p>
      <p className="mt-0.5 truncate text-[11px]" title={sub} style={{ color: "var(--text-secondary)" }}>
        {sub}
      </p>
    </div>
  );
}

