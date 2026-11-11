import { useState } from "react";
import {
  AlertCircle,
  AlertTriangle,
  Check,
  Copy,
  FolderOpen,
  Layers,
  Play,
  ShieldCheck,
  Sparkles,
  Terminal,
} from "lucide-react";
import { useAppStore } from "../../store/appStore";
import { isTauriRuntime, pickOutputDirectory } from "../../lib/tauri";
import type {
  AudioMode,
  ConversionInterfaceLevel,
  HardwareMode,
  OverwritePolicy,
  RateControlMode,
  ResolutionPreset,
  SubtitleMode,
} from "../../types/media";

const LEVELS: { id: ConversionInterfaceLevel; label: string }[] = [
  { id: "simple", label: "Simple" },
  { id: "advanced", label: "Advanced" },
  { id: "expert", label: "Expert" },
];

export function ConversionControls() {
  const interfaceLevel = useAppStore((s) => s.interfaceLevel);
  const setInterfaceLevel = useAppStore((s) => s.setInterfaceLevel);
  const capabilities = useAppStore((s) => s.capabilities);
  const presets = useAppStore((s) => s.presets);
  const mediaItems = useAppStore((s) => s.mediaItems);
  const activeMediaId = useAppStore((s) => s.activeMediaId);
  const selectedMediaIds = useAppStore((s) => s.selectedMediaIds);
  const settings = useAppStore((s) => s.settings);
  const updateSettings = useAppStore((s) => s.updateSettings);
  const applyPreset = useAppStore((s) => s.applyPreset);
  const plan = useAppStore((s) => s.plan);
  const startActiveConversion = useAppStore((s) => s.startActiveConversion);
  const startBatchConversion = useAppStore((s) => s.startBatchConversion);

  const [copiedCmd, setCopiedCmd] = useState(false);

  const activeMedia = mediaItems.find((m) => m.id === activeMediaId) ?? null;
  const availableContainers =
    capabilities?.supportedContainers.filter((c) => c.available) ?? [];
  const activeContainer = availableContainers.find((c) => c.id === settings.container);
  const isAudioOnlyContainer = activeContainer?.audioOnly ?? false;

  const compatibleVideoCodecs = activeContainer?.videoCodecs ?? ["copy", "h264"];
  const compatibleAudioCodecs = (activeContainer?.audioCodecs ?? ["copy", "aac"]).filter(
    (c) => c !== "copy"
  );

  // Encoders matching currently selected codec family
  const matchingVideoEncoders = (capabilities?.encoders ?? []).filter(
    (e) =>
      e.mediaType === "video" &&
      (e.codecFamily === settings.videoCodec || e.name.startsWith(settings.videoCodec))
  );

  const handleChooseOutputDir = async () => {
    if (!isTauriRuntime()) return;
    const dir = await pickOutputDirectory();
    if (dir) {
      updateSettings({ outputDir: dir });
    }
  };

  const handleCopyCommand = async () => {
    if (!plan?.commandPreview) return;
    try {
      await navigator.clipboard.writeText(plan.commandPreview);
      setCopiedCmd(true);
      setTimeout(() => setCopiedCmd(false), 1800);
    } catch {
      // clipboard blocked
    }
  };

  const handleSimpleQualityTier = (tier: string) => {
    switch (tier) {
      case "copy":
        updateSettings({
          videoCodec: "copy",
          audioMode: "copy",
          subtitleMode: "copyCompatible",
          resolution: "original",
          targetFps: null,
          pixelFormat: null,
        });
        break;
      case "high":
        updateSettings({
          videoCodec: isAudioOnlyContainer ? "copy" : settings.videoCodec === "copy" ? "h264" : settings.videoCodec,
          rateControl: "crf",
          qualityValue: 17,
          encoderPreset: "slow",
          audioMode: "transcode",
          audioBitrateKbps: 256,
        });
        break;
      case "balanced":
        updateSettings({
          videoCodec: isAudioOnlyContainer ? "copy" : settings.videoCodec === "copy" ? "h264" : settings.videoCodec,
          rateControl: "crf",
          qualityValue: 21,
          encoderPreset: "medium",
          audioMode: "transcode",
          audioBitrateKbps: 192,
        });
        break;
      case "compact":
        updateSettings({
          videoCodec: isAudioOnlyContainer ? "copy" : settings.videoCodec === "copy" ? "h264" : settings.videoCodec,
          rateControl: "crf",
          qualityValue: 26,
          encoderPreset: "medium",
          audioMode: "transcode",
          audioBitrateKbps: 128,
        });
        break;
    }
  };

  const currentSimpleTier =
    settings.videoCodec === "copy" && settings.audioMode === "copy"
      ? "copy"
      : settings.qualityValue <= 18
      ? "high"
      : settings.qualityValue <= 23
      ? "balanced"
      : "compact";

  const canRun = Boolean(activeMedia && plan && plan.decision !== "fail");

  return (
    <aside
      aria-label="Conversion Controls"
      className="flex min-h-0 flex-col border-l"
      style={{
        backgroundColor: "var(--bg-panel)",
        borderColor: "var(--border-subtle)",
      }}
    >
      {/* Header + Interface Level Switcher */}
      <div
        className="flex flex-col gap-2.5 border-b px-3.5 py-3"
        style={{ borderColor: "var(--border-subtle)" }}
      >
        <div className="flex items-center justify-between gap-2">
          <h2 className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-secondary)" }}>
            Export Settings
          </h2>

          <div
            role="tablist"
            aria-label="Interface Complexity Level"
            className="inline-flex rounded-lg border p-0.5"
            style={{
              backgroundColor: "var(--bg-elevated)",
              borderColor: "var(--border-subtle)",
            }}
          >
            {LEVELS.map((lvl) => {
              const selected = interfaceLevel === lvl.id;
              return (
                <button
                  key={lvl.id}
                  role="tab"
                  type="button"
                  aria-selected={selected}
                  onClick={() => setInterfaceLevel(lvl.id)}
                  className="rounded-md px-2.5 py-1 text-[11px] font-semibold transition-colors"
                  style={{
                    backgroundColor: selected ? "var(--bg-panel)" : "transparent",
                    color: selected ? "var(--text-primary)" : "var(--text-muted)",
                    boxShadow: selected ? "0 1px 2px rgba(0,0,0,0.2)" : "none",
                  }}
                >
                  {lvl.label}
                </button>
              );
            })}
          </div>
        </div>

        {/* Preset Selector */}
        <div>
          <label htmlFor="preset-quick-select" className="sr-only">
            Apply Export Preset
          </label>
          <select
            id="preset-quick-select"
            value={settings.presetId ?? ""}
            onChange={(e) => {
              const found = presets.find((p) => p.id === e.target.value);
              if (found) applyPreset(found);
            }}
            className="w-full rounded-md border px-2.5 py-1.5 text-xs"
            style={{
              backgroundColor: "var(--bg-input)",
              borderColor: "var(--border-subtle)",
              color: "var(--text-primary)",
            }}
          >
            <option value="">Custom Settings (Pick Preset…)</option>
            {presets.map((p) => (
              <option key={p.id} value={p.id} disabled={!p.available}>
                {p.category}: {p.name} {!p.available ? "(Unavailable)" : ""}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Scrollable Settings Controls */}
      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-3.5 text-xs">
        {/* Container Selection */}
        <FieldGroup label="Target Format / Container" htmlFor="ctl-container">
          <select
            id="ctl-container"
            value={settings.container}
            onChange={(e) => {
              const nextId = e.target.value;
              const nextContainer = availableContainers.find((c) => c.id === nextId);
              const nextVideoCodec = nextContainer?.audioOnly
                ? "copy"
                : nextContainer?.videoCodecs.includes(settings.videoCodec)
                ? settings.videoCodec
                : nextContainer?.videoCodecs.find((c) => c !== "copy") ?? "h264";
              const nextAudioCodec =
                nextContainer?.audioCodecs.find((c) => c !== "copy") ?? "aac";
              updateSettings({
                container: nextId,
                videoCodec: nextVideoCodec,
                videoEncoder: null,
                audioCodec: nextAudioCodec,
                audioMode: nextContainer?.audioOnly ? "transcode" : settings.audioMode,
              });
            }}
            className="ff-input"
          >
            {availableContainers.length === 0 ? (
              <option value="mp4">MP4 (MPEG-4 Part 14)</option>
            ) : (
              availableContainers.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.label} (.{c.extension})
                </option>
              ))
            )}
          </select>
        </FieldGroup>

        {/* SIMPLE MODE CONTROLS */}
        {interfaceLevel === "simple" && !isAudioOnlyContainer && (
          <FieldGroup label="Conversion Quality Profile" htmlFor="ctl-simple-tier">
            <select
              id="ctl-simple-tier"
              value={currentSimpleTier}
              onChange={(e) => handleSimpleQualityTier(e.target.value)}
              className="ff-input"
            >
              <option value="copy">Lossless Stream Copy (Remux Only — Instant)</option>
              <option value="high">High Quality Archive (CRF 17 · Slow)</option>
              <option value="balanced">Balanced Delivery (CRF 21 · Medium)</option>
              <option value="compact">Compact File Size (CRF 26 · Medium)</option>
            </select>
          </FieldGroup>
        )}

        {/* ADVANCED & EXPERT CONTROLS */}
        {(interfaceLevel === "advanced" || interfaceLevel === "expert") && (
          <>
            {!isAudioOnlyContainer && (
              <div className="space-y-3 rounded-lg border p-3" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-elevated)" }}>
                <p className="text-[11px] font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
                  Video Encoding
                </p>

                <div className="grid grid-cols-2 gap-2.5">
                  <FieldGroup label="Video Codec" htmlFor="ctl-vcodec">
                    <select
                      id="ctl-vcodec"
                      value={settings.videoCodec}
                      onChange={(e) =>
                        updateSettings({
                          videoCodec: e.target.value,
                          videoEncoder: null,
                        })
                      }
                      className="ff-input"
                    >
                      {compatibleVideoCodecs.map((vc) => (
                        <option key={vc} value={vc}>
                          {vc === "copy" ? "Stream Copy (No Re-encode)" : vc.toUpperCase()}
                        </option>
                      ))}
                    </select>
                  </FieldGroup>

                  <FieldGroup label="Hardware Mode" htmlFor="ctl-hw-mode">
                    <select
                      id="ctl-hw-mode"
                      disabled={settings.videoCodec === "copy"}
                      value={settings.hardwareMode}
                      onChange={(e) =>
                        updateSettings({
                          hardwareMode: e.target.value as HardwareMode,
                          videoEncoder: null,
                        })
                      }
                      className="ff-input"
                    >
                      <option value="auto">Auto (Verified Best)</option>
                      <option value="cpu">CPU Software</option>
                      <option value="gpu">GPU Hardware</option>
                      <option value="qualityPriority">Quality Priority</option>
                      <option value="speedPriority">Speed Priority</option>
                    </select>
                  </FieldGroup>
                </div>

                {settings.videoCodec !== "copy" && (
                  <>
                    <FieldGroup label="Specific Encoder" htmlFor="ctl-vencoder">
                      <select
                        id="ctl-vencoder"
                        value={settings.videoEncoder ?? ""}
                        onChange={(e) =>
                          updateSettings({
                            videoEncoder: e.target.value || null,
                          })
                        }
                        className="ff-input"
                      >
                        <option value="">Auto Select via Hardware Mode</option>
                        {matchingVideoEncoders.map((enc) => {
                          const probe = capabilities?.hardwareProbes.find(
                            (p) => p.encoderName === enc.name
                          );
                          const status = probe
                            ? probe.verifiedWorking
                              ? "✓ Verified HW"
                              : "⚠ Unverified HW"
                            : enc.isHardware
                            ? "HW"
                            : "CPU";
                          return (
                            <option key={enc.name} value={enc.name}>
                              {enc.name} ({status})
                            </option>
                          );
                        })}
                      </select>
                    </FieldGroup>

                    <div className="grid grid-cols-2 gap-2.5">
                      <FieldGroup label="Rate Control" htmlFor="ctl-rc">
                        <select
                          id="ctl-rc"
                          value={settings.rateControl}
                          onChange={(e) =>
                            updateSettings({ rateControl: e.target.value as RateControlMode })
                          }
                          className="ff-input"
                        >
                          <option value="crf">Constant Quality (CRF / CQ)</option>
                          <option value="bitrate">Target Bitrate (kbps)</option>
                          <option value="lossless">Lossless</option>
                        </select>
                      </FieldGroup>

                      {settings.rateControl === "bitrate" ? (
                        <FieldGroup label="Bitrate (kbps)" htmlFor="ctl-vbr">
                          <input
                            id="ctl-vbr"
                            type="number"
                            min={100}
                            max={250000}
                            step={250}
                            value={settings.videoBitrateKbps ?? 6000}
                            onChange={(e) =>
                              updateSettings({
                                videoBitrateKbps: Number(e.target.value) || 6000,
                              })
                            }
                            className="ff-input tabular-nums"
                          />
                        </FieldGroup>
                      ) : settings.rateControl !== "lossless" ? (
                        <FieldGroup
                          label={`Quality (${settings.qualityValue})`}
                          htmlFor="ctl-crf"
                        >
                          <div className="flex items-center gap-2 pt-1">
                            <input
                              id="ctl-crf"
                              type="range"
                              min={0}
                              max={45}
                              value={settings.qualityValue}
                              onChange={(e) =>
                                updateSettings({ qualityValue: Number(e.target.value) })
                              }
                              className="w-full"
                            />
                          </div>
                        </FieldGroup>
                      ) : (
                        <div className="flex items-end pb-1.5 text-[11px]" style={{ color: "var(--status-ok)" }}>
                          Mathematically lossless
                        </div>
                      )}
                    </div>

                    <div className="grid grid-cols-2 gap-2.5">
                      <FieldGroup label="Encoder Preset" htmlFor="ctl-enc-preset">
                        <select
                          id="ctl-enc-preset"
                          value={settings.encoderPreset ?? "medium"}
                          onChange={(e) =>
                            updateSettings({ encoderPreset: e.target.value || null })
                          }
                          className="ff-input"
                        >
                          <option value="ultrafast">Ultrafast</option>
                          <option value="veryfast">Veryfast</option>
                          <option value="fast">Fast</option>
                          <option value="medium">Medium</option>
                          <option value="slow">Slow</option>
                          <option value="slower">Slower</option>
                          <option value="veryslow">Veryslow</option>
                        </select>
                      </FieldGroup>

                      <FieldGroup label="Pixel Format" htmlFor="ctl-pixfmt">
                        <select
                          id="ctl-pixfmt"
                          value={settings.pixelFormat ?? ""}
                          onChange={(e) =>
                            updateSettings({ pixelFormat: e.target.value || null })
                          }
                          className="ff-input"
                        >
                          <option value="">Auto</option>
                          <option value="yuv420p">yuv420p (8-bit 4:2:0)</option>
                          <option value="yuv420p10le">yuv420p10le (10-bit 4:2:0)</option>
                          <option value="yuv422p10le">yuv422p10le (10-bit 4:2:2)</option>
                          <option value="nv12">nv12</option>
                        </select>
                      </FieldGroup>
                    </div>

                    <div className="grid grid-cols-2 gap-2.5">
                      <FieldGroup label="Resolution" htmlFor="ctl-res">
                        <select
                          id="ctl-res"
                          value={settings.resolution}
                          onChange={(e) =>
                            updateSettings({ resolution: e.target.value as ResolutionPreset })
                          }
                          className="ff-input"
                        >
                          <option value="original">Original</option>
                          <option value="uhd4k">3840p (4K UHD)</option>
                          <option value="qhd1440p">2560p (1440p)</option>
                          <option value="fhd1080p">1920p (1080p)</option>
                          <option value="hd720p">1280p (720p)</option>
                          <option value="custom">Custom Dimensions</option>
                        </select>
                      </FieldGroup>

                      <FieldGroup label="Frame Rate" htmlFor="ctl-fps">
                        <select
                          id="ctl-fps"
                          value={settings.targetFps ?? ""}
                          onChange={(e) =>
                            updateSettings({ targetFps: e.target.value || null })
                          }
                          className="ff-input"
                        >
                          <option value="">Original</option>
                          <option value="60">60 fps</option>
                          <option value="59.94">59.94 fps</option>
                          <option value="30">30 fps</option>
                          <option value="29.97">29.97 fps</option>
                          <option value="24">24 fps</option>
                          <option value="23.976">23.976 fps</option>
                        </select>
                      </FieldGroup>
                    </div>

                    {settings.resolution === "custom" && (
                      <div className="grid grid-cols-2 gap-2.5">
                        <FieldGroup label="Custom Width" htmlFor="ctl-cw">
                          <input
                            id="ctl-cw"
                            type="number"
                            min={16}
                            max={16384}
                            step={2}
                            value={settings.customWidth ?? 1920}
                            onChange={(e) =>
                              updateSettings({ customWidth: Number(e.target.value) || 1920 })
                            }
                            className="ff-input tabular-nums"
                          />
                        </FieldGroup>
                        <FieldGroup label="Custom Height" htmlFor="ctl-ch">
                          <input
                            id="ctl-ch"
                            type="number"
                            min={16}
                            max={16384}
                            step={2}
                            value={settings.customHeight ?? 1080}
                            onChange={(e) =>
                              updateSettings({ customHeight: Number(e.target.value) || 1080 })
                            }
                            className="ff-input tabular-nums"
                          />
                        </FieldGroup>
                      </div>
                    )}

                    <label className="flex items-center gap-2 pt-1 text-xs" style={{ color: "var(--text-secondary)" }}>
                      <input
                        type="checkbox"
                        checked={settings.allowCpuFallback}
                        onChange={(e) =>
                          updateSettings({ allowCpuFallback: e.target.checked })
                        }
                      />
                      Allow safe CPU fallback if GPU init fails
                    </label>
                  </>
                )}

                {activeMedia && activeMedia.videoStreams.length > 1 && (
                  <FieldGroup label="Source Video Stream" htmlFor="ctl-vstream">
                    <select
                      id="ctl-vstream"
                      value={settings.videoStreamIndex ?? ""}
                      onChange={(e) =>
                        updateSettings({
                          videoStreamIndex: e.target.value === "" ? null : Number(e.target.value),
                        })
                      }
                      className="ff-input"
                    >
                      <option value="">Primary Default Video Stream</option>
                      {activeMedia.videoStreams.map((vs) => (
                        <option key={vs.index} value={vs.index}>
                          Stream #{vs.index}: {vs.codecName} ({vs.width}×{vs.height})
                        </option>
                      ))}
                    </select>
                  </FieldGroup>
                )}
              </div>
            )}

            {/* Audio & Subtitles */}
            <div className="space-y-3 rounded-lg border p-3" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-elevated)" }}>
              <p className="text-[11px] font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
                Audio &amp; Subtitles
              </p>

              <div className="grid grid-cols-2 gap-2.5">
                <FieldGroup label="Audio Mode" htmlFor="ctl-amode">
                  <select
                    id="ctl-amode"
                    value={settings.audioMode}
                    onChange={(e) =>
                      updateSettings({ audioMode: e.target.value as AudioMode })
                    }
                    className="ff-input"
                  >
                    <option value="transcode">Transcode Audio</option>
                    <option value="copy">Stream Copy</option>
                    {!isAudioOnlyContainer && <option value="strip">Strip Audio (-an)</option>}
                  </select>
                </FieldGroup>

                {settings.audioMode === "transcode" && (
                  <FieldGroup label="Audio Codec" htmlFor="ctl-acodec">
                    <select
                      id="ctl-acodec"
                      value={settings.audioCodec ?? "aac"}
                      onChange={(e) => updateSettings({ audioCodec: e.target.value })}
                      className="ff-input"
                    >
                      {compatibleAudioCodecs.map((ac) => (
                        <option key={ac} value={ac}>
                          {ac.toUpperCase()}
                        </option>
                      ))}
                    </select>
                  </FieldGroup>
                )}
              </div>

              {settings.audioMode === "transcode" && (
                <div className="grid grid-cols-2 gap-2.5">
                  <FieldGroup label="Audio Bitrate" htmlFor="ctl-abr">
                    <select
                      id="ctl-abr"
                      value={settings.audioBitrateKbps ?? 192}
                      onChange={(e) =>
                        updateSettings({ audioBitrateKbps: Number(e.target.value) })
                      }
                      className="ff-input"
                    >
                      <option value={96}>96 kbps</option>
                      <option value={128}>128 kbps</option>
                      <option value={192}>192 kbps</option>
                      <option value={256}>256 kbps</option>
                      <option value={320}>320 kbps</option>
                    </select>
                  </FieldGroup>

                  <FieldGroup label="Channels" htmlFor="ctl-ach">
                    <select
                      id="ctl-ach"
                      value={settings.audioChannels ?? ""}
                      onChange={(e) =>
                        updateSettings({
                          audioChannels: e.target.value === "" ? null : Number(e.target.value),
                        })
                      }
                      className="ff-input"
                    >
                      <option value="">Original</option>
                      <option value="1">Mono (1 ch)</option>
                      <option value="2">Stereo (2 ch)</option>
                      <option value="6">5.1 Surround (6 ch)</option>
                    </select>
                  </FieldGroup>
                </div>
              )}

              {!isAudioOnlyContainer && (
                <FieldGroup label="Subtitles" htmlFor="ctl-subs">
                  <select
                    id="ctl-subs"
                    value={settings.subtitleMode}
                    onChange={(e) =>
                      updateSettings({ subtitleMode: e.target.value as SubtitleMode })
                    }
                    className="ff-input"
                  >
                    <option value="strip">Strip Subtitles</option>
                    <option value="copyCompatible">Copy Compatible Tracks</option>
                    <option value="convertMovText">Convert Text Subtitles to mov_text</option>
                  </select>
                </FieldGroup>
              )}

              <div className="flex flex-wrap gap-4 pt-1">
                <label className="flex items-center gap-2 text-xs" style={{ color: "var(--text-secondary)" }}>
                  <input
                    type="checkbox"
                    checked={settings.copyMetadata}
                    onChange={(e) => updateSettings({ copyMetadata: e.target.checked })}
                  />
                  Preserve Metadata
                </label>
                {activeContainer?.id === "mp4" || activeContainer?.id === "mov" ? (
                  <label className="flex items-center gap-2 text-xs" style={{ color: "var(--text-secondary)" }}>
                    <input
                      type="checkbox"
                      checked={settings.faststart}
                      onChange={(e) => updateSettings({ faststart: e.target.checked })}
                    />
                    MP4 Faststart
                  </label>
                ) : null}
              </div>
            </div>
          </>
        )}

        {/* EXPERT PARAMETERS + VALIDATED COMMAND PREVIEW */}
        {interfaceLevel === "expert" && (
          <div className="space-y-3 rounded-lg border p-3" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-elevated)" }}>
            <p className="text-[11px] font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
              Expert Encoder Tuning
            </p>

            <div className="grid grid-cols-2 gap-2.5">
              <FieldGroup label="Profile" htmlFor="ctl-profile">
                <input
                  id="ctl-profile"
                  type="text"
                  placeholder="e.g. high, main10"
                  value={settings.profile ?? ""}
                  onChange={(e) => updateSettings({ profile: e.target.value || null })}
                  className="ff-input font-mono-tech"
                />
              </FieldGroup>
              <FieldGroup label="Level" htmlFor="ctl-level">
                <input
                  id="ctl-level"
                  type="text"
                  placeholder="e.g. 4.1, 5.1"
                  value={settings.level ?? ""}
                  onChange={(e) => updateSettings({ level: e.target.value || null })}
                  className="ff-input font-mono-tech"
                />
              </FieldGroup>
              <FieldGroup label="Tune" htmlFor="ctl-tune">
                <input
                  id="ctl-tune"
                  type="text"
                  placeholder="e.g. film, animation"
                  value={settings.tune ?? ""}
                  onChange={(e) => updateSettings({ tune: e.target.value || null })}
                  className="ff-input font-mono-tech"
                />
              </FieldGroup>
              <FieldGroup label="GOP Size (-g)" htmlFor="ctl-gop">
                <input
                  id="ctl-gop"
                  type="number"
                  min={1}
                  max={600}
                  placeholder="Auto"
                  value={settings.gopSize ?? ""}
                  onChange={(e) =>
                    updateSettings({
                      gopSize: e.target.value === "" ? null : Number(e.target.value),
                    })
                  }
                  className="ff-input tabular-nums"
                />
              </FieldGroup>
            </div>

            <div>
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-1.5 text-[11px] font-semibold" style={{ color: "var(--text-secondary)" }}>
                  <Terminal className="h-3.5 w-3.5" aria-hidden="true" />
                  Validated Command Preview
                </span>
                {plan?.commandPreview && (
                  <button
                    type="button"
                    onClick={handleCopyCommand}
                    className="inline-flex items-center gap-1 text-[11px] font-medium"
                    style={{ color: "var(--accent-cyan)" }}
                  >
                    {copiedCmd ? <Check className="h-3 w-3" aria-hidden="true" /> : <Copy className="h-3 w-3" aria-hidden="true" />}
                    {copiedCmd ? "Copied" : "Copy"}
                  </button>
                )}
              </div>
              <pre
                className="mt-1.5 max-h-32 overflow-auto whitespace-pre-wrap break-all rounded-md border p-2 font-mono-tech text-[11px]"
                style={{
                  backgroundColor: "var(--bg-input)",
                  borderColor: "var(--border-subtle)",
                  color: "var(--text-secondary)",
                }}
              >
                {plan?.commandPreview || "Select a media file to preview typed FFmpeg arguments."}
              </pre>
            </div>
          </div>
        )}

        {/* DESTINATION & NAMING */}
        <div className="space-y-2.5 rounded-lg border p-3" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-elevated)" }}>
          <p className="text-[11px] font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
            Output Destination
          </p>

          <FieldGroup label="Output Folder" htmlFor="ctl-outdir">
            <div className="flex gap-1.5">
              <input
                id="ctl-outdir"
                type="text"
                readOnly
                value={settings.outputDir ?? "Same folder as source file"}
                className="ff-input min-w-0 flex-1 truncate font-mono-tech"
              />
              <button
                type="button"
                onClick={handleChooseOutputDir}
                aria-label="Browse output directory"
                className="inline-flex min-h-8 shrink-0 items-center gap-1 rounded-md border px-2.5 text-xs font-medium"
                style={{
                  backgroundColor: "var(--bg-panel)",
                  borderColor: "var(--border-subtle)",
                  color: "var(--text-primary)",
                }}
              >
                <FolderOpen className="h-3.5 w-3.5" aria-hidden="true" />
              </button>
              {settings.outputDir && (
                <button
                  type="button"
                  onClick={() => updateSettings({ outputDir: null })}
                  className="rounded-md border px-2 text-[11px]"
                  style={{
                    borderColor: "var(--border-subtle)",
                    color: "var(--text-muted)",
                  }}
                >
                  Reset
                </button>
              )}
            </div>
          </FieldGroup>

          <div className="grid grid-cols-2 gap-2.5">
            <FieldGroup label="Filename Pattern" htmlFor="ctl-pattern">
              <input
                id="ctl-pattern"
                type="text"
                value={settings.filenamePattern}
                onChange={(e) => updateSettings({ filenamePattern: e.target.value })}
                className="ff-input font-mono-tech"
              />
            </FieldGroup>
            <FieldGroup label="If Output Exists" htmlFor="ctl-overwrite">
              <select
                id="ctl-overwrite"
                value={settings.overwritePolicy}
                onChange={(e) =>
                  updateSettings({ overwritePolicy: e.target.value as OverwritePolicy })
                }
                className="ff-input"
              >
                <option value="renameIfExists">Auto-Rename (_1)</option>
                <option value="failIfExists">Fail Safely</option>
                <option value="overwrite">Overwrite Existing</option>
              </select>
            </FieldGroup>
          </div>
        </div>
      </div>

      {/* Decision Banner & Convert Action Footer */}
      <div
        className="space-y-2.5 border-t p-3.5"
        style={{
          borderColor: "var(--border-subtle)",
          backgroundColor: "var(--bg-elevated)",
        }}
      >
        {plan && <DecisionBanner plan={plan} />}

        <div className="flex flex-col gap-2">
          <button
            type="button"
            disabled={!canRun}
            onClick={startActiveConversion}
            className="flex min-h-10 w-full items-center justify-center gap-2 rounded-lg px-4 py-2 text-xs font-bold shadow-sm transition-opacity disabled:cursor-not-allowed disabled:opacity-45"
            style={{
              backgroundColor: "var(--accent-amber)",
              color: "var(--accent-amber-fg)",
            }}
          >
            <Play className="h-4 w-4 fill-current" aria-hidden="true" />
            {plan?.decision === "remux" || plan?.decision === "copy"
              ? "Remux / Copy Active File"
              : "Start Conversion"}
          </button>

          {selectedMediaIds.length > 1 && (
            <button
              type="button"
              disabled={!canRun}
              onClick={startBatchConversion}
              className="flex min-h-9 w-full items-center justify-center gap-2 rounded-lg border px-4 py-1.5 text-xs font-semibold transition-opacity disabled:cursor-not-allowed disabled:opacity-45"
              style={{
                backgroundColor: "var(--bg-panel)",
                borderColor: "var(--accent-cyan)",
                color: "var(--accent-cyan)",
              }}
            >
              <Layers className="h-3.5 w-3.5" aria-hidden="true" />
              Batch Convert ({selectedMediaIds.length} Selected)
            </button>
          )}
        </div>
      </div>
    </aside>
  );
}

function DecisionBanner({ plan }: { plan: NonNullable<ReturnType<typeof useAppStore.getState>["plan"]> }) {
  const isFail = plan.decision === "fail";
  const isCopyOrRemux = plan.decision === "copy" || plan.decision === "remux";

  const borderColor = isFail
    ? "var(--status-danger)"
    : isCopyOrRemux
    ? "var(--status-ok)"
    : "var(--accent-cyan)";
  const bgColor = isFail
    ? "var(--status-danger-soft)"
    : isCopyOrRemux
    ? "var(--status-ok-soft)"
    : "var(--accent-cyan-soft)";
  const textColor = isFail
    ? "var(--status-danger)"
    : isCopyOrRemux
    ? "var(--status-ok)"
    : "var(--accent-cyan)";

  return (
    <div
      className="rounded-lg border p-2.5 text-xs"
      style={{ borderColor, backgroundColor: bgColor }}
    >
      <div className="flex items-start gap-2">
        {isFail ? (
          <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" style={{ color: textColor }} aria-hidden="true" />
        ) : isCopyOrRemux ? (
          <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0" style={{ color: textColor }} aria-hidden="true" />
        ) : (
          <Sparkles className="mt-0.5 h-4 w-4 shrink-0" style={{ color: textColor }} aria-hidden="true" />
        )}
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-1.5">
            <span className="font-bold uppercase tracking-wide" style={{ color: textColor }}>
              {plan.decision}
            </span>
            <span className="font-semibold" style={{ color: "var(--text-primary)" }}>
              {plan.summary}
            </span>
          </div>
          <p className="mt-1 leading-relaxed" style={{ color: "var(--text-secondary)" }}>
            {plan.qualityExplanation}
          </p>
          {plan.resolvedOutputPath && !isFail && (
            <p
              className="mt-1 truncate font-mono-tech text-[11px]"
              title={plan.resolvedOutputPath}
              style={{ color: "var(--text-muted)" }}
            >
              Out: {plan.resolvedOutputPath}
            </p>
          )}
          {plan.warnings.map((w, i) => (
            <p key={i} className="mt-1 flex items-start gap-1 text-[11px]" style={{ color: "var(--status-warn)" }}>
              <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" aria-hidden="true" />
              <span>{w}</span>
            </p>
          ))}
          {plan.errors.map((e, i) => (
            <p key={i} className="mt-1 text-[11px] font-medium" style={{ color: "var(--status-danger)" }}>
              {e}
            </p>
          ))}
        </div>
      </div>
    </div>
  );
}

function FieldGroup({
  label,
  htmlFor,
  children,
}: {
  label: string;
  htmlFor: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1">
      <label
        htmlFor={htmlFor}
        className="text-[11px] font-medium"
        style={{ color: "var(--text-secondary)" }}
      >
        {label}
      </label>
      {children}
    </div>
  );
}

