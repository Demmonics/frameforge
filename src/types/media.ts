export interface SerializableError {
  code: string;
  message: string;
  details?: string | null;
  recoverable: boolean;
}

export interface ContainerInfo {
  formatName: string;
  formatLongName: string;
  probeScore?: number | null;
}

export interface HdrMetadata {
  masteringDisplay?: string | null;
  maxCll?: number | null;
  maxFall?: number | null;
}

export interface VideoStreamInfo {
  index: number;
  codecName: string;
  codecLongName: string;
  profile?: string | null;
  level?: number | null;
  width: number;
  height: number;
  codedWidth?: number | null;
  codedHeight?: number | null;
  sampleAspectRatio?: string | null;
  displayAspectRatio?: string | null;
  pixFmt?: string | null;
  bitDepth?: number | null;
  rFrameRate?: string | null;
  avgFrameRate?: string | null;
  fpsNominal?: number | null;
  fpsAverage?: number | null;
  isVfr: boolean;
  vfrNote?: string | null;
  bitRate?: number | null;
  nbFrames?: number | null;
  colorPrimaries?: string | null;
  colorTransfer?: string | null;
  colorSpace?: string | null;
  colorRange?: string | null;
  hdrFormat?: string | null;
  hdrMetadata?: HdrMetadata | null;
  rotationDegrees?: number | null;
  fieldOrder?: string | null;
  language?: string | null;
  title?: string | null;
  encoder?: string | null;
  isDefault: boolean;
  isAttachedPic: boolean;
  tags: Record<string, string>;
}

export interface AudioStreamInfo {
  index: number;
  codecName: string;
  codecLongName: string;
  profile?: string | null;
  sampleRate?: number | null;
  channels?: number | null;
  channelLayout?: string | null;
  sampleFmt?: string | null;
  bitRate?: number | null;
  language?: string | null;
  title?: string | null;
  isDefault: boolean;
  tags: Record<string, string>;
}

export interface SubtitleStreamInfo {
  index: number;
  codecName: string;
  codecLongName: string;
  language?: string | null;
  title?: string | null;
  isDefault: boolean;
  isForced: boolean;
  tags: Record<string, string>;
}

export interface MediaFile {
  id: string;
  fileName: string;
  path: string;
  sizeBytes: number;
  container: ContainerInfo;
  durationSecs?: number | null;
  bitRate?: number | null;
  primaryVideo?: VideoStreamInfo | null;
  primaryAudio?: AudioStreamInfo | null;
  videoStreams: VideoStreamInfo[];
  audioStreams: AudioStreamInfo[];
  subtitleStreams: SubtitleStreamInfo[];
  chaptersCount: number;
  creationTime?: string | null;
  encoderTag?: string | null;
  tags: Record<string, string>;
  thumbnailDataUrl?: string | null;
  thumbnailCachePath?: string | null;
  technicalReport: string;
  rawProbeJson: string;
  probedAtUtc: string;
}

export type MediaKind = "video" | "audio" | "subtitle";

export interface FormatCapability {
  name: string;
  description: string;
}

export interface CodecCapability {
  name: string;
  codecFamily: string;
  mediaType: MediaKind;
  description: string;
  isHardware: boolean;
  experimental: boolean;
  frameMt: boolean;
  sliceMt: boolean;
}

export interface FilterCapability {
  name: string;
  ioSignature: string;
  description: string;
}

export interface BinaryToolStatus {
  name: string;
  available: boolean;
  path?: string | null;
  version?: string | null;
  configuration?: string | null;
  error?: string | null;
}

export interface HardwareEncoderProbe {
  encoderName: string;
  backend: string;
  codecFamily: string;
  availableInFfmpeg: boolean;
  verifiedWorking: boolean;
  supportedPixelFormats: string[];
  supports10bit: boolean;
  diagnostic: string;
}

export interface ContainerExportOption {
  id: string;
  label: string;
  extension: string;
  muxer: string;
  description: string;
  available: boolean;
  videoCodecs: string[];
  audioCodecs: string[];
  audioOnly: boolean;
}

export interface CapabilityRegistry {
  ffmpeg: BinaryToolStatus;
  ffprobe: BinaryToolStatus;
  mpv: BinaryToolStatus;
  licenseProfile: string;
  demuxers: FormatCapability[];
  muxers: FormatCapability[];
  decoders: CodecCapability[];
  encoders: CodecCapability[];
  filters: FilterCapability[];
  hwaccels: string[];
  hardwareProbes: HardwareEncoderProbe[];
  supportedContainers: ContainerExportOption[];
  probedAtUtc: string;
}

export type ConversionInterfaceLevel = "simple" | "advanced" | "expert";
export type RateControlMode = "crf" | "cq" | "bitrate" | "lossless";
export type AudioMode = "copy" | "transcode" | "strip";
export type SubtitleMode = "copyCompatible" | "convertMovText" | "strip";
export type HardwareMode = "auto" | "cpu" | "gpu" | "qualityPriority" | "speedPriority";
export type OverwritePolicy = "renameIfExists" | "overwrite" | "failIfExists";
export type ResolutionPreset = "original" | "uhd4k" | "qhd1440p" | "fhd1080p" | "hd720p" | "custom";

export interface ConversionSettings {
  container: string;
  videoCodec: string;
  videoEncoder?: string | null;
  rateControl: RateControlMode;
  qualityValue: number;
  videoBitrateKbps?: number | null;
  encoderPreset?: string | null;
  resolution: ResolutionPreset;
  customWidth?: number | null;
  customHeight?: number | null;
  targetFps?: string | null;
  pixelFormat?: string | null;
  hardwareMode: HardwareMode;
  allowCpuFallback: boolean;
  videoStreamIndex?: number | null;
  audioMode: AudioMode;
  audioCodec?: string | null;
  audioBitrateKbps?: number | null;
  audioChannels?: number | null;
  audioSampleRate?: number | null;
  audioStreamIndices?: number[] | null;
  subtitleMode: SubtitleMode;
  copyMetadata: boolean;
  faststart: boolean;
  profile?: string | null;
  level?: string | null;
  tune?: string | null;
  gopSize?: number | null;
  bFrames?: number | null;
  colorRange?: string | null;
  outputDir?: string | null;
  filenamePattern: string;
  overwritePolicy: OverwritePolicy;
  presetId?: string | null;
  presetName?: string | null;
}

export type DecisionKind = "copy" | "remux" | "transcode" | "fail";
export type QualityImpact =
  | "losslessStreamCopy"
  | "losslessReencode"
  | "qualityAltered"
  | "audioOnlyExtraction"
  | "incompatible";

export interface ConversionPlan {
  decision: DecisionKind;
  qualityImpact: QualityImpact;
  summary: string;
  qualityExplanation: string;
  warnings: string[];
  errors: string[];
  resolvedOutputPath: string;
  resolvedMuxer: string;
  resolvedVideoEncoder?: string | null;
  resolvedAudioEncoder?: string | null;
  usingHardware: boolean;
  ffmpegProgram: string;
  ffmpegArgs: string[];
  commandPreview: string;
}

export type JobState =
  | "queued"
  | "validating"
  | "running"
  | "paused"
  | "cancelling"
  | "cancelled"
  | "failed"
  | "completed";

export interface JobProgress {
  percent: number;
  frame?: number | null;
  fps?: number | null;
  bitrate?: string | null;
  totalSizeBytes?: number | null;
  outTimeSecs: number;
  speed?: number | null;
  elapsedMs: number;
  etaSecs?: number | null;
}

export interface ConversionJob {
  id: string;
  inputPath: string;
  outputPath: string;
  mediaInfo: MediaFile;
  selectedPreset?: string | null;
  settings: ConversionSettings;
  hardwareMode: HardwareMode;
  decision: DecisionKind;
  state: JobState;
  progress: JobProgress;
  actualEncoder?: string | null;
  logs: string[];
  error?: SerializableError | null;
  outputSizeBytes?: number | null;
  createdAtUtc: string;
  startedAtUtc?: string | null;
  finishedAtUtc?: string | null;
}

export interface ExportPreset {
  id: string;
  name: string;
  category: string;
  description: string;
  isBuiltin: boolean;
  settings: ConversionSettings;
  available: boolean;
  unavailabilityReason?: string | null;
}

export interface AppConfig {
  binaries: {
    ffmpegOverride?: string | null;
    ffprobeOverride?: string | null;
    mpvOverride?: string | null;
  };
  concurrency: {
    maxCpuJobs: number;
    maxGpuJobs: number;
  };
  output: {
    defaultOutputDir?: string | null;
    filenamePattern: string;
    overwritePolicy: OverwritePolicy;
  };
  cacheDir: string;
  maxCacheBytes: number;
  probeHardwareAtStartup: boolean;
}

export interface ImportFileFailure {
  path: string;
  error: SerializableError;
}

export interface ImportBatchResult {
  imported: MediaFile[];
  failures: ImportFileFailure[];
}

export interface BatchStartResult {
  started: ConversionJob[];
  failures: ImportFileFailure[];
}

