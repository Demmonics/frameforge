import React from "react";
import {
  Activity,
  AlertCircle,
  Bookmark,
  CheckCircle2,
  Cpu,
  FileSearch,
  Film,
  FolderOpen,
  LayoutDashboard,
  ListChecks,
  Play,
  Plus,
  Settings,
  Sliders,
  Wrench,
  X,
} from "lucide-react";
import { useAppStore, type AppView } from "../../store/appStore";
import { pickMediaFiles, pickMediaFolder, isTauriRuntime } from "../../lib/tauri";
import { DropOverlay } from "./DropOverlay";

interface NavItem {
  id: AppView;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  badge?: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "converter", label: "Converter", icon: Sliders },
  { id: "inspector", label: "Media Inspector", icon: FileSearch },
  { id: "queue", label: "Queue", icon: ListChecks },
  { id: "presets", label: "Presets", icon: Bookmark },
  { id: "player", label: "Player", icon: Play, badge: "M-E" },
  { id: "toolbox", label: "Toolbox", icon: Wrench, badge: "M-G" },
  { id: "settings", label: "Settings", icon: Settings },
];

export function AppShell({ children }: { children: React.ReactNode }) {
  const activeView = useAppStore((s) => s.activeView);
  const setActiveView = useAppStore((s) => s.setActiveView);
  const capabilities = useAppStore((s) => s.capabilities);
  const jobs = useAppStore((s) => s.jobs);
  const mediaItems = useAppStore((s) => s.mediaItems);
  const notifications = useAppStore((s) => s.notifications);
  const dismissNotification = useAppStore((s) => s.dismissNotification);
  const importPaths = useAppStore((s) => s.importPaths);
  const isImporting = useAppStore((s) => s.isImporting);
  const isDraggingFiles = useAppStore((s) => s.isDraggingFiles);

  const activeJobsCount = jobs.filter((j) =>
    ["queued", "validating", "running", "cancelling"].includes(j.state)
  ).length;
  const verifiedGpuCount =
    capabilities?.hardwareProbes.filter((h) => h.verifiedWorking).length ?? 0;

  const handleOpenFiles = async () => {
    if (!isTauriRuntime()) return;
    const paths = await pickMediaFiles();
    if (paths.length > 0) {
      await importPaths(paths);
      if (activeView === "dashboard") setActiveView("converter");
    }
  };

  const handleOpenFolder = async () => {
    if (!isTauriRuntime()) return;
    const paths = await pickMediaFolder();
    if (paths.length > 0) {
      await importPaths(paths);
      if (activeView === "dashboard") setActiveView("converter");
    }
  };

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden" style={{ backgroundColor: "var(--bg-app)" }}>
      <DropOverlay visible={isDraggingFiles} />

      {/* Top Studio Bar */}
      <header
        className="flex h-12 shrink-0 items-center justify-between border-b px-4"
        style={{
          backgroundColor: "var(--bg-rail)",
          borderColor: "var(--border-subtle)",
        }}
      >
        <div className="flex items-center gap-3">
          <div
            className="flex h-7 w-7 items-center justify-center rounded-md border"
            style={{
              backgroundColor: "var(--accent-amber-soft)",
              borderColor: "var(--accent-amber)",
              color: "var(--accent-amber)",
            }}
          >
            <Film className="h-4 w-4" aria-hidden="true" />
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-sm font-bold tracking-wider" style={{ color: "var(--text-primary)" }}>
              FRAMEFORGE
            </span>
            <span className="hidden text-xs sm:inline" style={{ color: "var(--text-muted)" }}>
              Media Studio
            </span>
          </div>

          <div className="ml-3 hidden items-center gap-2 md:flex">
            <span
              className="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium"
              style={{
                borderColor: capabilities?.ffmpeg.available ? "var(--status-ok)" : "var(--status-danger)",
                backgroundColor: capabilities?.ffmpeg.available
                  ? "var(--status-ok-soft)"
                  : "var(--status-danger-soft)",
                color: capabilities?.ffmpeg.available ? "var(--status-ok)" : "var(--status-danger)",
              }}
            >
              <Activity className="h-3 w-3" aria-hidden="true" />
              {capabilities?.ffmpeg.available
                ? `FFmpeg ${capabilities.ffmpeg.version?.split(" ")[2] ?? "Ready"}`
                : "FFmpeg Missing"}
            </span>

            <span
              className="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium"
              style={{
                borderColor: verifiedGpuCount > 0 ? "var(--accent-cyan)" : "var(--border-subtle)",
                backgroundColor: verifiedGpuCount > 0 ? "var(--accent-cyan-soft)" : "var(--bg-elevated)",
                color: verifiedGpuCount > 0 ? "var(--accent-cyan)" : "var(--text-muted)",
              }}
            >
              <Cpu className="h-3 w-3" aria-hidden="true" />
              {verifiedGpuCount > 0 ? `${verifiedGpuCount} GPU Encoders` : "CPU Mode"}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleOpenFiles}
            disabled={isImporting}
            className="inline-flex min-h-8 items-center gap-1.5 rounded-md px-3 py-1 text-xs font-semibold shadow-xs transition-opacity disabled:opacity-60"
            style={{
              backgroundColor: "var(--accent-amber)",
              color: "var(--accent-amber-fg)",
            }}
          >
            <Plus className="h-3.5 w-3.5" aria-hidden="true" />
            {isImporting ? "Probing..." : "Import Media"}
          </button>

          <button
            type="button"
            onClick={handleOpenFolder}
            disabled={isImporting}
            className="inline-flex min-h-8 items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs font-medium transition-colors"
            style={{
              backgroundColor: "var(--bg-panel)",
              borderColor: "var(--border-subtle)",
              color: "var(--text-primary)",
            }}
          >
            <FolderOpen className="h-3.5 w-3.5" aria-hidden="true" />
            <span className="hidden sm:inline">Folder</span>
          </button>
        </div>
      </header>

      <div className="flex min-h-0 flex-1 overflow-hidden">
        {/* Left Navigation Rail */}
        <nav
          aria-label="Primary"
          className="flex w-52 shrink-0 flex-col justify-between border-r p-2.5"
          style={{
            backgroundColor: "var(--bg-rail)",
            borderColor: "var(--border-subtle)",
          }}
        >
          <ul className="flex flex-col gap-1">
            {NAV_ITEMS.map((item) => {
              const Icon = item.icon;
              const isActive = activeView === item.id;
              const count =
                item.id === "queue" && activeJobsCount > 0
                  ? activeJobsCount
                  : item.id === "converter" && mediaItems.length > 0
                  ? mediaItems.length
                  : null;

              return (
                <li key={item.id}>
                  <button
                    type="button"
                    onClick={() => setActiveView(item.id)}
                    aria-current={isActive ? "page" : undefined}
                    className="ff-panel-transition flex min-h-9 w-full items-center justify-between rounded-lg border px-2.5 py-1.5 text-left text-xs font-medium"
                    style={{
                      backgroundColor: isActive ? "var(--bg-panel)" : "transparent",
                      borderColor: isActive ? "var(--border-strong)" : "transparent",
                      color: isActive ? "var(--text-primary)" : "var(--text-secondary)",
                    }}
                  >
                    <span className="flex items-center gap-2.5">
                      <Icon
                        className="h-4 w-4 shrink-0"
                        aria-hidden="true"
                      />
                      <span>{item.label}</span>
                    </span>
                    {count !== null && (
                      <span
                        className="tabular-nums rounded-full px-1.5 py-0.5 text-[11px] font-semibold"
                        style={{
                          backgroundColor:
                            item.id === "queue" ? "var(--accent-cyan-soft)" : "var(--bg-elevated)",
                          color: item.id === "queue" ? "var(--accent-cyan)" : "var(--text-secondary)",
                        }}
                      >
                        {count}
                      </span>
                    )}
                    {item.badge && (
                      <span
                        className="rounded px-1.5 py-0.5 text-[10px] font-semibold"
                        style={{
                          backgroundColor: "var(--bg-elevated)",
                          color: "var(--text-muted)",
                        }}
                      >
                        {item.badge}
                      </span>
                    )}
                  </button>
                </li>
              );
            })}
          </ul>

          <div
            className="rounded-lg border p-2.5 text-[11px]"
            style={{
              backgroundColor: "var(--bg-panel)",
              borderColor: "var(--border-subtle)",
              color: "var(--text-muted)",
            }}
          >
            <p className="font-semibold" style={{ color: "var(--text-secondary)" }}>
              Offline Engine
            </p>
            <p className="mt-0.5 leading-relaxed">
              Local FFprobe inspection &amp; typed FFmpeg subprocesses. No cloud uploads.
            </p>
          </div>
        </nav>

        {/* Main Workspace + Notifications */}
        <main className="relative flex min-w-0 flex-1 flex-col overflow-hidden">
          {notifications.length > 0 && (
            <div className="pointer-events-none absolute right-4 top-3 z-40 flex w-full max-w-md flex-col gap-2">
              {notifications.map((n) => (
                <div
                  key={n.id}
                  role={n.kind === "error" ? "alert" : "status"}
                  className="pointer-events-auto flex items-start gap-2.5 rounded-lg border p-3 shadow-lg"
                  style={{
                    backgroundColor: "var(--bg-panel)",
                    borderColor:
                      n.kind === "error"
                        ? "var(--status-danger)"
                        : n.kind === "success"
                        ? "var(--status-ok)"
                        : "var(--border-strong)",
                  }}
                >
                  {n.kind === "error" ? (
                    <AlertCircle
                      className="mt-0.5 h-4 w-4 shrink-0"
                      style={{ color: "var(--status-danger)" }}
                      aria-hidden="true"
                    />
                  ) : (
                    <CheckCircle2
                      className="mt-0.5 h-4 w-4 shrink-0"
                      style={{ color: "var(--status-ok)" }}
                      aria-hidden="true"
                    />
                  )}
                  <div className="min-w-0 flex-1">
                    <p className="text-xs font-semibold" style={{ color: "var(--text-primary)" }}>
                      {n.title}
                    </p>
                    {n.detail && (
                      <pre
                        className="mt-1 max-h-24 overflow-y-auto whitespace-pre-wrap break-words font-mono-tech text-[11px]"
                        style={{ color: "var(--text-secondary)" }}
                      >
                        {n.detail}
                      </pre>
                    )}
                  </div>
                  <button
                    type="button"
                    onClick={() => dismissNotification(n.id)}
                    aria-label="Dismiss notification"
                    className="rounded p-1 hover:opacity-80"
                    style={{ color: "var(--text-muted)" }}
                  >
                    <X className="h-3.5 w-3.5" aria-hidden="true" />
                  </button>
                </div>
              ))}
            </div>
          )}

          {children}
        </main>
      </div>
    </div>
  );
}

