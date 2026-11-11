import { useEffect } from "react";
import { ThemeProvider } from "./features/shared/ThemeContext";
import { AppShell } from "./features/shared/AppShell";
import { ConverterWorkspace } from "./features/converter/ConverterWorkspace";
import { DashboardView } from "./features/dashboard/DashboardView";
import { InspectorView } from "./features/inspector/InspectorView";
import { QueueDock } from "./features/queue/QueueDock";
import { PresetsView } from "./features/presets/PresetsView";
import { SettingsView } from "./features/settings/SettingsView";
import { FutureModulesView } from "./features/roadmap/FutureModulesView";
import { useAppStore } from "./store/appStore";
import { isTauriRuntime, subscribeAppEvents } from "./lib/tauri";

function MainWorkspace() {
  const activeView = useAppStore((s) => s.activeView);
  const initialize = useAppStore((s) => s.initialize);
  const importPaths = useAppStore((s) => s.importPaths);
  const setDraggingFiles = useAppStore((s) => s.setDraggingFiles);
  const handleJobUpdatedEvent = useAppStore((s) => s.handleJobUpdatedEvent);
  const handleJobLogEvent = useAppStore((s) => s.handleJobLogEvent);
  const handleCapabilitiesEvent = useAppStore((s) => s.handleCapabilitiesEvent);

  useEffect(() => {
    void initialize();

    if (!isTauriRuntime()) return;
    let unlisten: (() => void) | undefined;
    let active = true;

    void subscribeAppEvents({
      onJobUpdated: handleJobUpdatedEvent,
      onJobLog: ({ jobId, line }) => handleJobLogEvent(jobId, line),
      onCapabilities: handleCapabilitiesEvent,
      onDragEnter: () => setDraggingFiles(true),
      onDragLeave: () => setDraggingFiles(false),
      onDropPaths: (paths) => {
        void importPaths(paths);
      },
    }).then((fn) => {
      if (active) {
        unlisten = fn;
      } else {
        fn();
      }
    });

    return () => {
      active = false;
      if (unlisten) unlisten();
    };
  }, [
    initialize,
    importPaths,
    setDraggingFiles,
    handleJobUpdatedEvent,
    handleJobLogEvent,
    handleCapabilitiesEvent,
  ]);

  switch (activeView) {
    case "dashboard":
      return <DashboardView />;
    case "converter":
      return <ConverterWorkspace />;
    case "inspector":
      return <InspectorView />;
    case "queue":
      return <QueueDock fullPage />;
    case "presets":
      return <PresetsView />;
    case "settings":
      return <SettingsView />;
    case "player":
      return <FutureModulesView kind="player" />;
    case "toolbox":
      return <FutureModulesView kind="toolbox" />;
  }
}

export function App() {
  return (
    <ThemeProvider>
      <AppShell>
        <MainWorkspace />
      </AppShell>
    </ThemeProvider>
  );
}

export default App;

