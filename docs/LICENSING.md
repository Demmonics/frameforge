# FRAMEFORGE Licensing & Redistribution Compliance Audit

## Overview

FRAMEFORGE delegates media decoding, probing, transcoding, and future playback to external native tools (**FFmpeg**, **FFprobe**, and planned **mpv/libmpv**). Simply invoking an external binary via subprocess does **not** automatically absolve license obligations if binaries are bundled in a distributed installer.

## Current Development Discovery

During Milestones A–C, FRAMEFORGE discovers user-installed `ffmpeg`, `ffprobe`, and optional `mpv` binaries from `PATH` or user-configured overrides in Settings, and inspects `ffmpeg -version` configuration flags at runtime:

- Example observed local build: `ffmpeg version 8.0.1-full_build-www.gyan.dev`
- Observed build flags: `--enable-gpl --enable-version3` (plus GPL/LGPL codec libraries such as `libx264`, `libx265`, `libvidstab`, `libxvid`, etc.)
- Resulting FFmpeg binary license profile: **GPLv3** (no `--enable-nonfree`).

## Obligations Before Public / Commercial Release (Milestone H)

1. **Application Source & Dependencies**:
   - FRAMEFORGE Rust & TypeScript code (`MIT` workspace) uses permissive MIT/Apache-2.0 crates (`tauri`, `tokio`, `serde`, `thiserror`, `tracing`).
2. **FFmpeg Bundling Options**:
   - **Option A (LGPLv2.1+/LGPLv3 commercial-friendly bundling)**: Build/distribute an LGPL-only FFmpeg binary (`--disable-gpl --disable-nonfree`) and exclude GPL-only encoders (`libx264`, `libx265`), OR dynamically link LGPL FFmpeg libraries with relinking compliance. Note that excluding `x264`/`x265` shifts H.264/HEVC encoding to OS-provided encoders (`h264_mf`, hardware `nvenc`/`qsv`/`amf`, or OpenH264).
   - **Option B (GPLv3 binary bundling)**: If bundling a GPL FFmpeg binary (`--enable-gpl`) inside the installer, ensure full GPL compliance across combined distribution terms or provide clean installer-time user acquisition of unmodified external GPL FFmpeg builds with clear license notices and source offer URLs.
   - **Never** bundle `--enable-nonfree` FFmpeg builds (e.g., `libfdk_aac` mixed with GPL).
3. **mpv / libmpv (Milestone E)**:
   - `libmpv` built with GPL FFmpeg/components is GPLv2+/GPLv3; verify exact build flags and `COPYING` notices before bundling.
4. **Codec Patent Considerations**:
   - Patent licensing for H.264/AVC, HEVC/H.265, etc. is independent of open-source software copyright licenses and must be reviewed separately by legal counsel prior to commercial distribution.

## Release Checklist

- [ ] Pin exact FFmpeg/FFprobe/mpv release version, SHA-256 checksum, and build origin URL.
- [ ] Record exact `ffmpeg -version` configure flags in release manifest.
- [ ] Verify `--enable-nonfree` is absent.
- [ ] Include third-party `LICENSES/` directory and attribution notices in installer package.
- [ ] Include corresponding FFmpeg/mpv source code archive or written source offer per GPL/LGPL terms.

