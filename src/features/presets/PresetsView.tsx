import { ArrowRight, Bookmark, CheckCircle2, XCircle } from "lucide-react";
import { useAppStore } from "../../store/appStore";

export function PresetsView() {
  const presets = useAppStore((s) => s.presets);
  const applyPreset = useAppStore((s) => s.applyPreset);
  const setActiveView = useAppStore((s) => s.setActiveView);

  return (
    <div className="flex h-full min-h-0 flex-col overflow-y-auto p-5">
      <div className="border-b pb-4" style={{ borderColor: "var(--border-subtle)" }}>
        <h1 className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>
          Export Presets
        </h1>
        <p className="mt-1 text-xs" style={{ color: "var(--text-secondary)" }}>
          Each preset is verified live against your installed FFmpeg muxers and encoders before activation.
        </p>
      </div>

      <div className="mt-5 grid grid-cols-1 gap-3.5 md:grid-cols-2 xl:grid-cols-3">
        {presets.map((preset) => (
          <div
            key={preset.id}
            className="flex flex-col justify-between rounded-xl border p-4"
            style={{
              backgroundColor: "var(--bg-panel)",
              borderColor: "var(--border-subtle)",
              opacity: preset.available ? 1 : 0.65,
            }}
          >
            <div>
              <div className="flex items-start justify-between gap-2">
                <div>
                  <span
                    className="inline-block rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider"
                    style={{
                      backgroundColor: "var(--bg-elevated)",
                      color: "var(--accent-cyan)",
                    }}
                  >
                    {preset.category}
                  </span>
                  <h2 className="mt-1.5 flex items-center gap-1.5 text-sm font-bold" style={{ color: "var(--text-primary)" }}>
                    <Bookmark className="h-3.5 w-3.5" style={{ color: "var(--accent-amber)" }} aria-hidden="true" />
                    {preset.name}
                  </h2>
                </div>
                {preset.available ? (
                  <CheckCircle2 className="h-4 w-4 shrink-0" style={{ color: "var(--status-ok)" }} aria-label="Available" />
                ) : (
                  <XCircle className="h-4 w-4 shrink-0" style={{ color: "var(--status-danger)" }} aria-label="Unavailable" />
                )}
              </div>

              <p className="mt-2 text-xs leading-relaxed" style={{ color: "var(--text-secondary)" }}>
                {preset.description}
              </p>

              <div className="mt-3 flex flex-wrap gap-1.5 font-mono-tech text-[11px]">
                <span className="rounded border px-1.5 py-0.5" style={{ borderColor: "var(--border-subtle)" }}>
                  .{preset.settings.container}
                </span>
                <span className="rounded border px-1.5 py-0.5" style={{ borderColor: "var(--border-subtle)" }}>
                  video:{preset.settings.videoEncoder ?? preset.settings.videoCodec}
                </span>
                <span className="rounded border px-1.5 py-0.5" style={{ borderColor: "var(--border-subtle)" }}>
                  audio:{preset.settings.audioMode === "copy" ? "copy" : preset.settings.audioCodec ?? preset.settings.audioMode}
                </span>
              </div>

              {preset.unavailabilityReason && (
                <p className="mt-2 text-[11px]" style={{ color: "var(--status-danger)" }}>
                  {preset.unavailabilityReason}
                </p>
              )}
            </div>

            <button
              type="button"
              disabled={!preset.available}
              onClick={() => {
                applyPreset(preset);
                setActiveView("converter");
              }}
              className="mt-4 inline-flex min-h-8 items-center justify-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold disabled:cursor-not-allowed disabled:opacity-45"
              style={{
                backgroundColor: "var(--accent-amber)",
                color: "var(--accent-amber-fg)",
              }}
            >
              Use in Converter
              <ArrowRight className="h-3.5 w-3.5" aria-hidden="true" />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

