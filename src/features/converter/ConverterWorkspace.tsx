import { MediaBin } from "./MediaBin";
import { PreviewStage } from "./PreviewStage";
import { ConversionControls } from "./ConversionControls";
import { QueueDock } from "../queue/QueueDock";

export function ConverterWorkspace() {
  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="grid min-h-0 flex-1 grid-cols-1 lg:grid-cols-[290px_minmax(0,1fr)_380px]">
        <MediaBin />
        <PreviewStage />
        <ConversionControls />
      </div>
      <QueueDock />
    </div>
  );
}

