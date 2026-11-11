import { Activity, CheckCircle2, Cpu, Film, FolderOpen, Plus, ShieldCheck, XCircle } from "lucide-react";
import { useAppStore } from "../../store/appStore";
import { isTauriRuntime, pickMediaFiles } from "../../lib/tauri";

export function DashboardView() {
  const capabilities = useAppStore((s) => s.capabilities);
  const mediaItems = useAppStore((s) => s.mediaItems);
  const jobs = useAppStore((s) => s.jobs);
  const importPaths = useAppStore((s) => s.importPaths);
  const setActiveView = useAppStore((s) => s.setActiveView);

  const handleImport = async () => {
    if (!isTauriRuntime()) return;
    const paths = await pickMediaFiles();
    if (paths.length > 0) {
      await importPaths(paths);
      setActiveView("converter");
    }
  };

  const verifiedHw = capabilities?.hardwareProbes.filter((h) => h.verifiedWorking) ?? [];
  const completedJobs = jobs.filter((j) => j.state === "completed").length;

  return (
    <div className="flex h-full min-h-0 flex-col overflow-y-auto p-6">
      <div className="flex flex-wrap items-center justify-between gap-4 border-b pb-5" style={{ borderColor: "var(--border-subtle)" }}>
        <div>
          <h1 className="text-xl font-bold tracking-tight" style={{ color: "var(--text-primary)" }}>
            FRAMEFORGE Studio Overview
          </h1>
          <p className="mt-1 text-xs" style={{ color: "var(--text-secondary)" }}>
            Offline-first Rust media workstation backed by live FFprobe inspection, typed FFmpeg execution, and verified GPU probing.
          </p>
        </div>
        <div className="flex items-center gap-2.5">
          <button
            type="button"
            onClick={handleImport}
            className="inline-flex min-h-9 items-center gap-2 rounded-lg px-4 py-2 text-xs font-semibold"
            style={{
              backgroundColor: "var(--accent-amber)",
              color: "var(--accent-amber-fg)",
            }}
          >
            <Plus className="h-4 w-4" aria-hidden="true" />
            Import Media
          </button>
          <button
            type="button"
            onClick={() => setActiveView("converter")}
            className="inline-flex min-h-9 items-center gap-2 rounded-lg border px-4 py-2 text-xs font-semibold"
            style={{
              backgroundColor: "var(--bg-panel)",
              borderColor: "var(--border-subtle)",
              color: "var(--text-primary)",
            }}
          >
            <FolderOpen className="h-4 w-4" aria-hidden="true" />
            Open Converter
          </button>
        </div>
      </div>

      {/* Live Capability Metrics */}
      <div className="mt-5 grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          icon={Activity}
          label="FFmpeg Engine"
          value={capabilities?.ffmpeg.available ? "Ready" : "Unavailable"}
          detail={capabilities?.ffmpeg.version ?? capabilities?.ffmpeg.error ?? "Checking…"}
        />
        <StatCard
          icon={Cpu}
          label="Verified Hardware Encoders"
          value={String(verifiedHw.length)}
          detail={
            verifiedHw.length > 0
              ? verifiedHw.map((h) => h.encoderName).join(", ")
              : "Using CPU software encoders"
          }
        />
        <StatCard
          icon={ShieldCheck}
          label="Runtime Capabilities"
          value={`${capabilities?.supportedContainers.filter((c) => c.available).length ?? 0} Formats`}
          detail={`${capabilities?.encoders.length ?? 0} encoders · ${capabilities?.muxers.length ?? 0} muxers · ${capabilities?.filters.length ?? 0} filters`}
        />
        <StatCard
          icon={Film}
          label="Session Workspace"
          value={`${mediaItems.length} Media`}
          detail={`${completedJobs} completed · ${jobs.length} total jobs`}
        />
      </div>

      {/* Hardware Probe Matrix */}
      <section className="mt-6">
        <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
          Hardware Acceleration Probe Results
        </h2>
        <div className="mt-2.5 grid grid-cols-1 gap-2.5 md:grid-cols-2 xl:grid-cols-3">
          {capabilities?.hardwareProbes.length ? (
            capabilities.hardwareProbes.map((probe) => (
              <div
                key={probe.encoderName}
                className="flex items-start gap-2.5 rounded-lg border p-3"
                style={{
                  backgroundColor: "var(--bg-panel)",
                  borderColor: "var(--border-subtle)",
                }}
              >
                {probe.verifiedWorking ? (
                  <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0" style={{ color: "var(--status-ok)" }} aria-hidden="true" />
                ) : (
                  <XCircle className="mt-0.5 h-4 w-4 shrink-0" style={{ color: "var(--text-muted)" }} aria-hidden="true" />
                )}
                <div className="min-w-0 flex-1">
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-mono-tech text-xs font-bold" style={{ color: "var(--text-primary)" }}>
                      {probe.encoderName}
                    </span>
                    <span className="text-[11px]" style={{ color: "var(--text-muted)" }}>
                      {probe.backend}
                    </span>
                  </div>
                  <p className="mt-1 text-[11px]" style={{ color: "var(--text-secondary)" }}>
                    {probe.diagnostic}
                  </p>
                </div>
              </div>
            ))
          ) : (
            <p className="text-xs" style={{ color: "var(--text-muted)" }}>
              No hardware encoders detected in FFmpeg build.
            </p>
          )}
        </div>
      </section>
    </div>
  );
}

function StatCard({
  icon: Icon,
  label,
  value,
  detail,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <div
      className="rounded-xl border p-4"
      style={{
        backgroundColor: "var(--bg-panel)",
        borderColor: "var(--border-subtle)",
      }}
    >
      <div className="flex items-center gap-2 text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
        <Icon className="h-4 w-4" aria-hidden="true" />
        <span>{label}</span>
      </div>
      <p className="tabular-nums mt-2 text-xl font-bold" style={{ color: "var(--text-primary)" }}>
        {value}
      </p>
      <p className="mt-1 truncate text-[11px]" title={detail} style={{ color: "var(--text-muted)" }}>
        {detail}
      </p>
    </div>
  );
}

