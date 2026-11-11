import { Upload } from "lucide-react";

export function DropOverlay({ visible }: { visible: boolean }) {
  if (!visible) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-8 backdrop-blur-xs"
      style={{ backgroundColor: "color-mix(in oklab, var(--bg-app) 82%, transparent)" }}
      aria-live="polite"
    >
      <div
        className="flex max-w-md flex-col items-center gap-4 rounded-2xl border-2 border-dashed px-10 py-12 text-center shadow-2xl"
        style={{
          borderColor: "var(--accent-amber)",
          backgroundColor: "var(--bg-panel)",
        }}
      >
        <div
          className="flex h-16 w-16 items-center justify-center rounded-xl"
          style={{ backgroundColor: "var(--accent-amber-soft)", color: "var(--accent-amber)" }}
        >
          <Upload className="h-8 w-8" aria-hidden="true" />
        </div>
        <div>
          <p className="text-lg font-semibold" style={{ color: "var(--text-primary)" }}>
            Drop media files or folders to inspect
          </p>
          <p className="mt-1 text-sm" style={{ color: "var(--text-secondary)" }}>
            FFprobe analyzes actual stream headers locally — MP4, MKV, MOV, WebM, AVI, ProRes, audio &amp; images.
          </p>
        </div>
      </div>
    </div>
  );
}

