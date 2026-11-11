import {
  AlertCircle,
  Ban,
  CheckCircle2,
  ChevronDown,
  ChevronUp,
  ExternalLink,
  FolderOpen,
  Loader2,
  Square,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { useAppStore } from "../../store/appStore";
import { fileBaseName, formatBytes, formatEta } from "../../lib/format";
import type { ConversionJob } from "../../types/media";

export function QueueDock({ fullPage = false }: { fullPage?: boolean }) {
  const jobs = useAppStore((s) => s.jobs);
  const cancelJob = useAppStore((s) => s.cancelJob);
  const clearFinishedJobs = useAppStore((s) => s.clearFinishedJobs);
  const openOutputFile = useAppStore((s) => s.openOutputFile);
  const revealInFolder = useAppStore((s) => s.revealInFolder);

  const [collapsed, setCollapsed] = useState(false);
  const [expandedLogId, setExpandedLogId] = useState<string | null>(null);

  const activeCount = jobs.filter((j) =>
    ["queued", "validating", "running", "cancelling"].includes(j.state)
  ).length;
  const finishedCount = jobs.length - activeCount;

  if (!fullPage && jobs.length === 0) {
    return (
      <section
        aria-label="Job Queue Dock"
        className="flex h-10 shrink-0 items-center justify-between border-t px-4 text-xs"
        style={{
          backgroundColor: "var(--bg-rail)",
          borderColor: "var(--border-subtle)",
          color: "var(--text-muted)",
        }}
      >
        <span>Job Queue Ready — no active conversions</span>
        <span>Bounded CPU/GPU scheduling</span>
      </section>
    );
  }

  return (
    <section
      aria-label="Conversion Job Queue"
      className={
        fullPage
          ? "flex h-full min-h-0 flex-col overflow-hidden p-5"
          : "flex shrink-0 flex-col border-t"
      }
      style={{
        backgroundColor: fullPage ? "var(--bg-app)" : "var(--bg-rail)",
        borderColor: "var(--border-subtle)",
      }}
    >
      <div
        className={`flex items-center justify-between ${
          fullPage ? "mb-4" : "h-10 border-b px-4"
        }`}
        style={{ borderColor: "var(--border-subtle)" }}
      >
        <div className="flex items-center gap-2.5">
          {fullPage ? (
            <h1 className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>
              Conversion Queue
            </h1>
          ) : (
            <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
              Active Queue
            </h2>
          )}
          <span
            className="tabular-nums rounded-full px-2 py-0.5 text-[11px] font-semibold"
            style={{
              backgroundColor: activeCount > 0 ? "var(--accent-cyan-soft)" : "var(--bg-panel)",
              color: activeCount > 0 ? "var(--accent-cyan)" : "var(--text-muted)",
            }}
          >
            {activeCount} running · {jobs.length} total
          </span>
        </div>

        <div className="flex items-center gap-2">
          {finishedCount > 0 && (
            <button
              type="button"
              onClick={clearFinishedJobs}
              className="inline-flex min-h-7 items-center gap-1 rounded border px-2 py-0.5 text-xs font-medium"
              style={{
                backgroundColor: "var(--bg-panel)",
                borderColor: "var(--border-subtle)",
                color: "var(--text-secondary)",
              }}
            >
              <Trash2 className="h-3 w-3" aria-hidden="true" />
              Clear Finished
            </button>
          )}
          {!fullPage && (
            <button
              type="button"
              onClick={() => setCollapsed((c) => !c)}
              aria-label={collapsed ? "Expand queue dock" : "Collapse queue dock"}
              className="rounded p-1"
              style={{ color: "var(--text-secondary)" }}
            >
              {collapsed ? (
                <ChevronUp className="h-4 w-4" aria-hidden="true" />
              ) : (
                <ChevronDown className="h-4 w-4" aria-hidden="true" />
              )}
            </button>
          )}
        </div>
      </div>

      {(!collapsed || fullPage) && (
        <div
          className={`min-h-0 flex-1 overflow-y-auto ${
            fullPage ? "space-y-3" : "max-h-56 space-y-2 p-3"
          }`}
        >
          {jobs.length === 0 ? (
            <div
              className="flex h-48 items-center justify-center rounded-xl border border-dashed text-sm"
              style={{
                borderColor: "var(--border-subtle)",
                color: "var(--text-muted)",
              }}
            >
              No conversion jobs queued yet.
            </div>
          ) : (
            jobs.map((job) => (
              <JobRow
                key={job.id}
                job={job}
                logsOpen={expandedLogId === job.id}
                onToggleLogs={() =>
                  setExpandedLogId((cur) => (cur === job.id ? null : job.id))
                }
                onCancel={() => cancelJob(job.id)}
                onOpen={() => openOutputFile(job.outputPath)}
                onReveal={() => revealInFolder(job.outputPath)}
              />
            ))
          )}
        </div>
      )}
    </section>
  );
}

function JobRow({
  job,
  logsOpen,
  onToggleLogs,
  onCancel,
  onOpen,
  onReveal,
}: {
  job: ConversionJob;
  logsOpen: boolean;
  onToggleLogs: () => void;
  onCancel: () => void;
  onOpen: () => void;
  onReveal: () => void;
}) {
  const isActive = ["queued", "validating", "running", "cancelling"].includes(job.state);
  const isCompleted = job.state === "completed";
  const isFailed = job.state === "failed";
  const isCancelled = job.state === "cancelled";

  return (
    <div
      className="rounded-lg border p-3"
      style={{
        backgroundColor: "var(--bg-panel)",
        borderColor: isFailed
          ? "var(--status-danger)"
          : isCompleted
          ? "var(--status-ok)"
          : "var(--border-subtle)",
      }}
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex min-w-0 flex-1 items-center gap-2.5">
          <JobStateIcon state={job.state} />
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <span className="truncate text-xs font-semibold" style={{ color: "var(--text-primary)" }}>
                {fileBaseName(job.inputPath)} → {fileBaseName(job.outputPath)}
              </span>
              <span
                className="rounded px-1.5 py-0.5 text-[10px] font-bold uppercase"
                style={{
                  backgroundColor: "var(--bg-elevated)",
                  color: "var(--text-secondary)",
                }}
              >
                {job.decision}
              </span>
              {job.actualEncoder && (
                <span className="font-mono-tech text-[11px]" style={{ color: "var(--accent-cyan)" }}>
                  {job.actualEncoder}
                </span>
              )}
            </div>
            <p
              className="mt-0.5 truncate font-mono-tech text-[11px]"
              title={job.outputPath}
              style={{ color: "var(--text-muted)" }}
            >
              {job.outputPath}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-1.5">
          {isActive && (
            <button
              type="button"
              disabled={job.state === "cancelling"}
              onClick={onCancel}
              className="inline-flex min-h-7 items-center gap-1 rounded border px-2.5 py-1 text-xs font-semibold disabled:opacity-50"
              style={{
                borderColor: "var(--status-danger)",
                backgroundColor: "var(--status-danger-soft)",
                color: "var(--status-danger)",
              }}
            >
              <Square className="h-3 w-3 fill-current" aria-hidden="true" />
              {job.state === "cancelling" ? "Stopping…" : "Cancel"}
            </button>
          )}
          {isCompleted && (
            <>
              <button
                type="button"
                onClick={onOpen}
                className="inline-flex min-h-7 items-center gap-1 rounded px-2.5 py-1 text-xs font-semibold"
                style={{
                  backgroundColor: "var(--status-ok)",
                  color: "#090b10",
                }}
              >
                <ExternalLink className="h-3 w-3" aria-hidden="true" />
                Open File
              </button>
              <button
                type="button"
                onClick={onReveal}
                className="inline-flex min-h-7 items-center gap-1 rounded border px-2 py-1 text-xs font-medium"
                style={{
                  borderColor: "var(--border-subtle)",
                  color: "var(--text-primary)",
                }}
              >
                <FolderOpen className="h-3 w-3" aria-hidden="true" />
                Folder
              </button>
            </>
          )}
          <button
            type="button"
            onClick={onToggleLogs}
            className="inline-flex min-h-7 items-center rounded border px-2 py-1 text-[11px]"
            style={{
              borderColor: "var(--border-subtle)",
              color: "var(--text-secondary)",
            }}
          >
            {logsOpen ? "Hide Logs" : "Logs"}
          </button>
        </div>
      </div>

      {/* Progress Telemetry */}
      {(isActive || isCompleted) && (
        <div className="mt-2.5 space-y-1.5">
          <progress
            className="ff-progress h-2 w-full"
            value={isCompleted ? 100 : job.progress.percent}
            max={100}
            aria-label={`Progress for ${fileBaseName(job.outputPath)}`}
          />
          <div className="tabular-nums flex flex-wrap items-center justify-between gap-2 text-[11px]" style={{ color: "var(--text-secondary)" }}>
            <span>
              {isCompleted ? "100.0% · Completed" : `${job.progress.percent.toFixed(1)}% · ${job.state.toUpperCase()}`}
            </span>
            <div className="flex flex-wrap items-center gap-3">
              {job.progress.speed != null && <span>{job.progress.speed.toFixed(2)}x speed</span>}
              {job.progress.fps != null && job.progress.fps > 0 && (
                <span>{job.progress.fps.toFixed(1)} fps</span>
              )}
              {job.progress.bitrate && <span>{job.progress.bitrate}</span>}
              {isActive && job.progress.etaSecs != null && (
                <span>ETA {formatEta(job.progress.etaSecs)}</span>
              )}
              {isCompleted && job.outputSizeBytes != null && (
                <span>Output: {formatBytes(job.outputSizeBytes)}</span>
              )}
            </div>
          </div>
        </div>
      )}

      {isCancelled && (
        <p className="mt-1.5 text-xs" style={{ color: "var(--text-muted)" }}>
          Cancelled cleanly; temporary files removed.
        </p>
      )}

      {isFailed && job.error && (
        <div
          className="mt-2 rounded border p-2 text-xs"
          style={{
            borderColor: "var(--status-danger)",
            backgroundColor: "var(--status-danger-soft)",
            color: "var(--status-danger)",
          }}
        >
          <p className="font-semibold">{job.error.message}</p>
          {job.error.details && (
            <pre className="mt-1 max-h-24 overflow-auto whitespace-pre-wrap font-mono-tech text-[11px]">
              {job.error.details}
            </pre>
          )}
        </div>
      )}

      {logsOpen && (
        <pre
          className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap rounded border p-2 font-mono-tech text-[11px]"
          style={{
            backgroundColor: "var(--bg-input)",
            borderColor: "var(--border-subtle)",
            color: "var(--text-secondary)",
          }}
        >
          {job.logs.length > 0 ? job.logs.join("\n") : "No FFmpeg log lines recorded yet."}
        </pre>
      )}
    </div>
  );
}

function JobStateIcon({ state }: { state: ConversionJob["state"] }) {
  switch (state) {
    case "completed":
      return <CheckCircle2 className="h-4 w-4 shrink-0" style={{ color: "var(--status-ok)" }} aria-hidden="true" />;
    case "failed":
      return <AlertCircle className="h-4 w-4 shrink-0" style={{ color: "var(--status-danger)" }} aria-hidden="true" />;
    case "cancelled":
      return <Ban className="h-4 w-4 shrink-0" style={{ color: "var(--text-muted)" }} aria-hidden="true" />;
    default:
      return <Loader2 className="h-4 w-4 shrink-0 animate-spin" style={{ color: "var(--accent-cyan)" }} aria-hidden="true" />;
  }
}

