import { Check, Film, Music, Plus, Trash2, Upload } from "lucide-react";
import { useAppStore } from "../../store/appStore";
import { formatBytes, formatDuration } from "../../lib/format";
import { pickMediaFiles, isTauriRuntime } from "../../lib/tauri";

export function MediaBin() {
  const mediaItems = useAppStore((s) => s.mediaItems);
  const selectedMediaIds = useAppStore((s) => s.selectedMediaIds);
  const activeMediaId = useAppStore((s) => s.activeMediaId);
  const selectMedia = useAppStore((s) => s.selectMedia);
  const selectAllMedia = useAppStore((s) => s.selectAllMedia);
  const removeMedia = useAppStore((s) => s.removeMedia);
  const clearMedia = useAppStore((s) => s.clearMedia);
  const importPaths = useAppStore((s) => s.importPaths);

  const handleImport = async () => {
    if (!isTauriRuntime()) return;
    const paths = await pickMediaFiles();
    if (paths.length > 0) await importPaths(paths);
  };

  return (
    <aside
      aria-label="Imported Media Bin"
      className="flex min-h-0 flex-col border-r"
      style={{
        backgroundColor: "var(--bg-panel)",
        borderColor: "var(--border-subtle)",
      }}
    >
      <div
        className="flex items-center justify-between border-b px-3 py-2.5"
        style={{ borderColor: "var(--border-subtle)" }}
      >
        <div className="flex items-center gap-2">
          <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
            Media Bin
          </h2>
          <span
            className="tabular-nums rounded px-1.5 py-0.5 text-[11px] font-medium"
            style={{ backgroundColor: "var(--bg-elevated)", color: "var(--text-muted)" }}
          >
            {mediaItems.length}
          </span>
        </div>

        <div className="flex items-center gap-1">
          {mediaItems.length > 1 && (
            <button
              type="button"
              onClick={selectAllMedia}
              className="rounded px-2 py-1 text-[11px] font-medium hover:opacity-80"
              style={{ color: "var(--accent-cyan)" }}
            >
              Select All
            </button>
          )}
          {mediaItems.length > 0 && (
            <button
              type="button"
              onClick={clearMedia}
              aria-label="Clear media bin"
              className="rounded p-1 hover:opacity-80"
              style={{ color: "var(--text-muted)" }}
            >
              <Trash2 className="h-3.5 w-3.5" aria-hidden="true" />
            </button>
          )}
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto p-2.5">
        {mediaItems.length === 0 ? (
          <div
            className="flex h-full min-h-64 flex-col items-center justify-center rounded-xl border border-dashed p-5 text-center"
            style={{
              borderColor: "var(--border-strong)",
              backgroundColor: "var(--bg-elevated)",
            }}
          >
            <Upload className="h-8 w-8" style={{ color: "var(--accent-amber)" }} aria-hidden="true" />
            <p className="mt-3 text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
              Drop videos or audio here
            </p>
            <p className="mt-1 text-xs" style={{ color: "var(--text-muted)" }}>
              Supports MP4, MKV, MOV, WebM, AVI, ProRes, DNxHR, FLAC, WAV, and folders.
            </p>
            <button
              type="button"
              onClick={handleImport}
              className="mt-4 inline-flex min-h-8 items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-semibold"
              style={{
                backgroundColor: "var(--accent-amber)",
                color: "var(--accent-amber-fg)",
              }}
            >
              <Plus className="h-3.5 w-3.5" aria-hidden="true" />
              Browse Files
            </button>
          </div>
        ) : (
          <ul className="flex flex-col gap-2">
            {mediaItems.map((item) => {
              const isActive = item.id === activeMediaId;
              const isSelected = selectedMediaIds.includes(item.id);
              const v = item.primaryVideo;

              return (
                <li
                  key={item.id}
                  className="ff-panel-transition group relative rounded-lg border p-2"
                  style={{
                    backgroundColor: isActive ? "var(--bg-elevated)" : "var(--bg-input)",
                    borderColor: isActive
                      ? "var(--accent-amber)"
                      : isSelected
                      ? "var(--accent-cyan)"
                      : "var(--border-subtle)",
                  }}
                >
                  <div className="flex items-start gap-2.5">
                    <button
                      type="button"
                      onClick={() => selectMedia(item.id, true)}
                      aria-label={`Select ${item.fileName} for batch conversion`}
                      aria-pressed={isSelected}
                      className="mt-1 flex h-5 w-5 shrink-0 items-center justify-center rounded border"
                      style={{
                        borderColor: isSelected ? "var(--accent-cyan)" : "var(--border-strong)",
                        backgroundColor: isSelected ? "var(--accent-cyan)" : "transparent",
                        color: "#090b10",
                      }}
                    >
                      {isSelected && <Check className="h-3 w-3 stroke-[3]" aria-hidden="true" />}
                    </button>

                    <button
                      type="button"
                      onClick={() => selectMedia(item.id, false)}
                      className="flex min-w-0 flex-1 items-start gap-2.5 text-left"
                    >
                      <div
                        className="relative flex h-12 w-20 shrink-0 items-center justify-center overflow-hidden rounded border"
                        style={{
                          backgroundColor: "#090b10",
                          borderColor: "var(--border-subtle)",
                        }}
                      >
                        {item.thumbnailDataUrl ? (
                          <img
                            src={item.thumbnailDataUrl}
                            alt={`Frame preview of ${item.fileName}`}
                            className="h-full w-full object-cover"
                          />
                        ) : v ? (
                          <Film className="h-5 w-5" style={{ color: "var(--text-muted)" }} aria-hidden="true" />
                        ) : (
                          <Music className="h-5 w-5" style={{ color: "var(--accent-cyan)" }} aria-hidden="true" />
                        )}
                        {item.durationSecs != null && (
                          <span
                            className="tabular-nums absolute bottom-0.5 right-0.5 rounded px-1 py-0 text-[10px] font-semibold"
                            style={{ backgroundColor: "rgba(0,0,0,0.78)", color: "#f8fafc" }}
                          >
                            {formatDuration(item.durationSecs)}
                          </span>
                        )}
                      </div>

                      <div className="min-w-0 flex-1">
                        <p
                          className="truncate text-xs font-semibold"
                          title={item.path}
                          style={{ color: "var(--text-primary)" }}
                        >
                          {item.fileName}
                        </p>
                        <div className="mt-1 flex flex-wrap items-center gap-1 text-[11px]" style={{ color: "var(--text-muted)" }}>
                          <span className="font-mono-tech uppercase">{v ? v.codecName : item.primaryAudio?.codecName ?? "media"}</span>
                          {v && v.width > 0 && (
                            <>
                              <span>·</span>
                              <span className="tabular-nums">{v.width}×{v.height}</span>
                            </>
                          )}
                          <span>·</span>
                          <span className="tabular-nums">{formatBytes(item.sizeBytes)}</span>
                        </div>
                        {(v?.isVfr || v?.hdrFormat) && (
                          <div className="mt-1 flex flex-wrap gap-1">
                            {v.isVfr && (
                              <span
                                className="rounded px-1 py-0 text-[10px] font-semibold"
                                style={{
                                  backgroundColor: "var(--status-warn-soft)",
                                  color: "var(--status-warn)",
                                }}
                              >
                                VFR
                              </span>
                            )}
                            {v.hdrFormat && (
                              <span
                                className="rounded px-1 py-0 text-[10px] font-semibold"
                                style={{
                                  backgroundColor: "var(--accent-cyan-soft)",
                                  color: "var(--accent-cyan)",
                                }}
                              >
                                HDR
                              </span>
                            )}
                          </div>
                        )}
                      </div>
                    </button>

                    <button
                      type="button"
                      onClick={() => removeMedia(item.id)}
                      aria-label={`Remove ${item.fileName}`}
                      className="rounded p-1 opacity-70 hover:opacity-100"
                      style={{ color: "var(--text-muted)" }}
                    >
                      <Trash2 className="h-3.5 w-3.5" aria-hidden="true" />
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </aside>
  );
}

