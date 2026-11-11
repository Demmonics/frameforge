import { useState } from "react";
import { Check, Copy, Download, FileSearch } from "lucide-react";
import { useAppStore } from "../../store/appStore";
import { formatBitrate, formatBytes, formatDuration, formatFps } from "../../lib/format";
import { exportProbeJson, isTauriRuntime, pickSaveJsonPath } from "../../lib/tauri";

export function InspectorView() {
  const mediaItems = useAppStore((s) => s.mediaItems);
  const activeMediaId = useAppStore((s) => s.activeMediaId);
  const selectMedia = useAppStore((s) => s.selectMedia);
  const pushNotification = useAppStore((s) => s.pushNotification);

  const [copiedReport, setCopiedReport] = useState(false);
  const [showRawJson, setShowRawJson] = useState(false);

  const media = mediaItems.find((m) => m.id === activeMediaId) ?? mediaItems[0] ?? null;

  const handleCopyReport = async () => {
    if (!media) return;
    try {
      await navigator.clipboard.writeText(media.technicalReport);
      setCopiedReport(true);
      setTimeout(() => setCopiedReport(false), 1800);
    } catch {
      pushNotification({ kind: "error", title: "Unable to access system clipboard" });
    }
  };

  const handleExportJson = async () => {
    if (!media) return;
    if (!isTauriRuntime()) return;
    const defaultName = `${media.fileName.replace(/\.[^.]+$/, "")}_ffprobe.json`;
    const savePath = await pickSaveJsonPath(defaultName);
    if (!savePath) return;
    try {
      await exportProbeJson(savePath, media.rawProbeJson);
      pushNotification({
        kind: "success",
        title: "FFprobe JSON exported",
        detail: savePath,
      });
    } catch (err) {
      pushNotification({
        kind: "error",
        title: "Failed to export JSON",
        detail: String(err),
      });
    }
  };

  if (!media) {
    return (
      <div className="flex h-full flex-col items-center justify-center p-8 text-center">
        <FileSearch className="h-10 w-10" style={{ color: "var(--text-muted)" }} aria-hidden="true" />
        <h1 className="mt-4 text-lg font-bold" style={{ color: "var(--text-primary)" }}>
          Media Inspector
        </h1>
        <p className="mt-1 max-w-md text-xs" style={{ color: "var(--text-muted)" }}>
          Import any video, audio, or image file to inspect detailed FFprobe container, color, HDR, VFR, rotation, and stream telemetry.
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col overflow-y-auto p-5">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b pb-4" style={{ borderColor: "var(--border-subtle)" }}>
        <div className="min-w-0 flex-1">
          <h1 className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>
            Media Inspector
          </h1>
          <p className="mt-0.5 truncate font-mono-tech text-xs" title={media.path} style={{ color: "var(--text-muted)" }}>
            {media.path}
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {mediaItems.length > 1 && (
            <div>
              <label htmlFor="inspector-file-select" className="sr-only">
                Select imported file to inspect
              </label>
              <select
                id="inspector-file-select"
                value={media.id}
                onChange={(e) => selectMedia(e.target.value, false)}
                className="ff-input w-56"
              >
                {mediaItems.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.fileName}
                  </option>
                ))}
              </select>
            </div>
          )}

          <button
            type="button"
            onClick={handleCopyReport}
            className="inline-flex min-h-8 items-center gap-1.5 rounded-md border px-3 py-1.5 text-xs font-medium"
            style={{
              backgroundColor: "var(--bg-panel)",
              borderColor: "var(--border-subtle)",
              color: "var(--text-primary)",
            }}
          >
            {copiedReport ? <Check className="h-3.5 w-3.5" aria-hidden="true" /> : <Copy className="h-3.5 w-3.5" aria-hidden="true" />}
            {copiedReport ? "Copied Report" : "Copy Report"}
          </button>

          <button
            type="button"
            onClick={handleExportJson}
            className="inline-flex min-h-8 items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-semibold"
            style={{
              backgroundColor: "var(--accent-amber)",
              color: "var(--accent-amber-fg)",
            }}
          >
            <Download className="h-3.5 w-3.5" aria-hidden="true" />
            Export JSON
          </button>
        </div>
      </div>

      {/* General Container Properties */}
      <section className="mt-4">
        <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
          Container &amp; File Summary
        </h2>
        <div className="mt-2 grid grid-cols-2 gap-3 md:grid-cols-4">
          <InfoTile label="Container" value={`${media.container.formatLongName} (${media.container.formatName})`} />
          <InfoTile label="File Size" value={`${formatBytes(media.sizeBytes)} (${media.sizeBytes.toLocaleString()} B)`} />
          <InfoTile label="Duration" value={formatDuration(media.durationSecs)} />
          <InfoTile label="Overall Bitrate" value={formatBitrate(media.bitRate)} />
          <InfoTile label="Creation Time" value={media.creationTime ?? "Not present"} />
          <InfoTile label="Writing App / Encoder" value={media.encoderTag ?? "Not present"} />
          <InfoTile label="Chapters" value={String(media.chaptersCount)} />
          <InfoTile label="Probe Score" value={media.container.probeScore != null ? `${media.container.probeScore}/100` : "—"} />
        </div>
      </section>

      {/* Video Streams Table */}
      <section className="mt-6">
        <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
          Video Streams ({media.videoStreams.length})
        </h2>
        {media.videoStreams.length === 0 ? (
          <p className="mt-2 text-xs" style={{ color: "var(--text-muted)" }}>
            No video streams present.
          </p>
        ) : (
          <div className="mt-2 overflow-x-auto rounded-lg border" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-panel)" }}>
            <table className="w-full border-collapse text-left text-xs">
              <caption className="sr-only">Video streams metadata</caption>
              <thead>
                <tr className="border-b" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-elevated)", color: "var(--text-secondary)" }}>
                  <th scope="col" className="px-3 py-2 font-semibold">#</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Codec / Profile</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Resolution / Aspect</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Frame Rate / VFR</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Pixel Format / Depth</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Color / HDR</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Bitrate / Rot</th>
                </tr>
              </thead>
              <tbody className="divide-y" style={{ borderColor: "var(--border-subtle)" }}>
                {media.videoStreams.map((v) => (
                  <tr key={v.index}>
                    <td className="tabular-nums px-3 py-2.5 font-mono-tech">
                      #{v.index} {v.isDefault ? "(default)" : ""} {v.isAttachedPic ? "(art)" : ""}
                    </td>
                    <td className="px-3 py-2.5">
                      <div className="font-semibold">{v.codecName.toUpperCase()}</div>
                      <div style={{ color: "var(--text-muted)" }}>
                        {v.profile ?? "—"} {v.level != null ? `L${v.level}` : ""}
                      </div>
                    </td>
                    <td className="tabular-nums px-3 py-2.5">
                      <div className="font-semibold">{v.width}×{v.height}</div>
                      <div style={{ color: "var(--text-muted)" }}>
                        DAR {v.displayAspectRatio ?? "—"} · SAR {v.sampleAspectRatio ?? "—"}
                      </div>
                    </td>
                    <td className="tabular-nums px-3 py-2.5">
                      <div>{formatFps(v.fpsNominal)}</div>
                      <div style={{ color: v.isVfr ? "var(--status-warn)" : "var(--text-muted)" }}>
                        {v.isVfr ? `VFR (avg ${formatFps(v.fpsAverage)})` : "CFR"}
                      </div>
                    </td>
                    <td className="px-3 py-2.5 font-mono-tech">
                      <div>{v.pixFmt ?? "—"}</div>
                      <div style={{ color: "var(--text-muted)" }}>{v.bitDepth ?? 8}-bit</div>
                    </td>
                    <td className="px-3 py-2.5">
                      <div>{v.hdrFormat ?? "SDR"}</div>
                      <div style={{ color: "var(--text-muted)" }}>
                        {v.colorPrimaries ?? "—"} / {v.colorTransfer ?? "—"}
                      </div>
                    </td>
                    <td className="tabular-nums px-3 py-2.5">
                      <div>{formatBitrate(v.bitRate)}</div>
                      <div style={{ color: "var(--text-muted)" }}>
                        {v.rotationDegrees != null ? `${v.rotationDegrees}° rot` : "0°"}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Audio & Subtitle Streams */}
      <div className="mt-6 grid grid-cols-1 gap-5 xl:grid-cols-2">
        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
            Audio Streams ({media.audioStreams.length})
          </h2>
          <div className="mt-2 overflow-x-auto rounded-lg border" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-panel)" }}>
            <table className="w-full border-collapse text-left text-xs">
              <caption className="sr-only">Audio streams metadata</caption>
              <thead>
                <tr className="border-b" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-elevated)", color: "var(--text-secondary)" }}>
                  <th scope="col" className="px-3 py-2 font-semibold">#</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Codec</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Channels / Rate</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Bitrate</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Lang</th>
                </tr>
              </thead>
              <tbody className="divide-y" style={{ borderColor: "var(--border-subtle)" }}>
                {media.audioStreams.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-3 py-4 text-center" style={{ color: "var(--text-muted)" }}>
                      No audio streams.
                    </td>
                  </tr>
                ) : (
                  media.audioStreams.map((a) => (
                    <tr key={a.index}>
                      <td className="tabular-nums px-3 py-2 font-mono-tech">#{a.index}</td>
                      <td className="px-3 py-2 font-semibold">{a.codecName.toUpperCase()}</td>
                      <td className="tabular-nums px-3 py-2">
                        {a.channelLayout ?? `${a.channels ?? "?"}ch`} · {a.sampleRate ? `${a.sampleRate} Hz` : "—"}
                      </td>
                      <td className="tabular-nums px-3 py-2">{formatBitrate(a.bitRate)}</td>
                      <td className="px-3 py-2 font-mono-tech">{a.language ?? "und"}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>

        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
            Subtitle Streams ({media.subtitleStreams.length})
          </h2>
          <div className="mt-2 overflow-x-auto rounded-lg border" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-panel)" }}>
            <table className="w-full border-collapse text-left text-xs">
              <caption className="sr-only">Subtitle streams metadata</caption>
              <thead>
                <tr className="border-b" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-elevated)", color: "var(--text-secondary)" }}>
                  <th scope="col" className="px-3 py-2 font-semibold">#</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Codec</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Language</th>
                  <th scope="col" className="px-3 py-2 font-semibold">Title / Flags</th>
                </tr>
              </thead>
              <tbody className="divide-y" style={{ borderColor: "var(--border-subtle)" }}>
                {media.subtitleStreams.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-3 py-4 text-center" style={{ color: "var(--text-muted)" }}>
                      No subtitle streams.
                    </td>
                  </tr>
                ) : (
                  media.subtitleStreams.map((s) => (
                    <tr key={s.index}>
                      <td className="tabular-nums px-3 py-2 font-mono-tech">#{s.index}</td>
                      <td className="px-3 py-2 font-semibold">{s.codecName}</td>
                      <td className="px-3 py-2 font-mono-tech">{s.language ?? "und"}</td>
                      <td className="px-3 py-2">
                        {s.title ?? "—"} {s.isForced ? "(forced)" : ""}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>
      </div>

      {/* Text Report & Raw JSON */}
      <section className="mt-6 space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
            Technical Report
          </h2>
          <button
            type="button"
            onClick={() => setShowRawJson((v) => !v)}
            className="text-xs font-medium"
            style={{ color: "var(--accent-cyan)" }}
          >
            {showRawJson ? "Show Formatted Report" : "Show Raw FFprobe JSON"}
          </button>
        </div>

        <pre
          className="max-h-72 overflow-auto rounded-lg border p-3.5 font-mono-tech text-xs leading-relaxed"
          style={{
            backgroundColor: "var(--bg-panel)",
            borderColor: "var(--border-subtle)",
            color: "var(--text-secondary)",
          }}
        >
          {showRawJson ? media.rawProbeJson : media.technicalReport}
        </pre>
      </section>
    </div>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div
      className="rounded-lg border p-3"
      style={{
        backgroundColor: "var(--bg-panel)",
        borderColor: "var(--border-subtle)",
      }}
    >
      <p className="text-[11px]" style={{ color: "var(--text-muted)" }}>
        {label}
      </p>
      <p className="mt-0.5 truncate text-xs font-semibold" title={value} style={{ color: "var(--text-primary)" }}>
        {value}
      </p>
    </div>
  );
}

