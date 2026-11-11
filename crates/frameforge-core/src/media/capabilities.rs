use chrono::Utc;
use std::path::Path;
use tokio::process::Command;

use crate::core::config::AppConfig;
use crate::core::models::{
    BinaryToolStatus, CapabilityRegistry, CodecCapability, FilterCapability, FormatCapability,
    MediaKind,
};
use crate::hardware::detection::probe_hardware_encoders;
use crate::media::formats::build_supported_containers;
use crate::platform::{configure_background_command, discover_binary};

pub async fn build_capability_registry(config: &AppConfig) -> CapabilityRegistry {
    let ffmpeg_res = discover_binary("ffmpeg", config.binaries.ffmpeg_override.as_deref());
    let ffprobe_res = discover_binary("ffprobe", config.binaries.ffprobe_override.as_deref());
    let mpv_res = discover_binary("mpv", config.binaries.mpv_override.as_deref());

    let ffmpeg_status = match &ffmpeg_res {
        Ok(path) => query_tool_version("ffmpeg", path).await,
        Err(err) => BinaryToolStatus {
            name: "ffmpeg".to_string(),
            available: false,
            path: None,
            version: None,
            configuration: None,
            error: Some(err.to_string()),
        },
    };

    let ffprobe_status = match &ffprobe_res {
        Ok(path) => query_tool_version("ffprobe", path).await,
        Err(err) => BinaryToolStatus {
            name: "ffprobe".to_string(),
            available: false,
            path: None,
            version: None,
            configuration: None,
            error: Some(err.to_string()),
        },
    };

    let mpv_status = match &mpv_res {
        Ok(path) => query_tool_version("mpv", path).await,
        Err(err) => BinaryToolStatus {
            name: "mpv".to_string(),
            available: false,
            path: None,
            version: None,
            configuration: None,
            error: Some(err.to_string()),
        },
    };

    let license_profile = detect_license_profile(ffmpeg_status.configuration.as_deref());

    let (demuxers, muxers, decoders, encoders, filters, hwaccels) = match &ffmpeg_res {
        Ok(ffmpeg_path) if ffmpeg_status.available => {
            let (demux, mux, dec, enc, flt, hwa) = tokio::join!(
                run_ffmpeg_query(ffmpeg_path, "-demuxers"),
                run_ffmpeg_query(ffmpeg_path, "-muxers"),
                run_ffmpeg_query(ffmpeg_path, "-decoders"),
                run_ffmpeg_query(ffmpeg_path, "-encoders"),
                run_ffmpeg_query(ffmpeg_path, "-filters"),
                run_ffmpeg_query(ffmpeg_path, "-hwaccels"),
            );
            (
                parse_format_list(&demux),
                parse_format_list(&mux),
                parse_codec_list(&dec),
                parse_codec_list(&enc),
                parse_filter_list(&flt),
                parse_hwaccel_list(&hwa),
            )
        }
        _ => (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
    };

    let hardware_probes = match &ffmpeg_res {
        Ok(ffmpeg_path) if ffmpeg_status.available && config.probe_hardware_at_startup => {
            probe_hardware_encoders(ffmpeg_path, &encoders).await
        }
        _ => Vec::new(),
    };

    let supported_containers = build_supported_containers(&muxers, &encoders);

    CapabilityRegistry {
        ffmpeg: ffmpeg_status,
        ffprobe: ffprobe_status,
        mpv: mpv_status,
        license_profile,
        demuxers,
        muxers,
        decoders,
        encoders,
        filters,
        hwaccels,
        hardware_probes,
        supported_containers,
        probed_at_utc: Utc::now().to_rfc3339(),
    }
}

async fn query_tool_version(name: &str, path: &Path) -> BinaryToolStatus {
    let mut cmd = Command::new(path);
    cmd.arg("-version");
    configure_background_command(&mut cmd);

    match cmd.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            let version = lines.first().map(|l| l.trim().to_string());
            let configuration = lines
                .iter()
                .find(|l| l.trim_start().starts_with("configuration:"))
                .map(|l| l.trim().to_string());

            BinaryToolStatus {
                name: name.to_string(),
                available: output.status.success(),
                path: Some(path.display().to_string()),
                version,
                configuration,
                error: if output.status.success() {
                    None
                } else {
                    Some(String::from_utf8_lossy(&output.stderr).trim().to_string())
                },
            }
        }
        Err(err) => BinaryToolStatus {
            name: name.to_string(),
            available: false,
            path: Some(path.display().to_string()),
            version: None,
            configuration: None,
            error: Some(err.to_string()),
        },
    }
}

async fn run_ffmpeg_query(ffmpeg_path: &Path, flag: &str) -> String {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args(["-hide_banner", flag]);
    configure_background_command(&mut cmd);
    match cmd.output().await {
        Ok(out) => String::from_utf8_lossy(&out.stdout).into_owned(),
        Err(_) => String::new(),
    }
}

fn detect_license_profile(config_line: Option<&str>) -> String {
    let Some(cfg) = config_line else {
        return "Unknown (FFmpeg unavailable)".to_string();
    };
    let gpl = cfg.contains("--enable-gpl");
    let nonfree = cfg.contains("--enable-nonfree");
    let v3 = cfg.contains("--enable-version3");
    match (gpl, nonfree, v3) {
        (_, true, _) => "Nonfree / Unredistributable (audit required)".to_string(),
        (true, false, true) => "GPLv3 (requires GPLv3 source compliance on bundling)".to_string(),
        (true, false, false) => "GPLv2+ (requires GPL source compliance on bundling)".to_string(),
        (false, false, true) => "LGPLv3".to_string(),
        (false, false, false) => "LGPLv2.1+".to_string(),
    }
}

pub fn parse_format_list(raw: &str) -> Vec<FormatCapability> {
    let mut out = Vec::new();
    let mut past_divider = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "---" || trimmed.starts_with("---") {
            past_divider = true;
            continue;
        }
        if !past_divider {
            continue;
        }
        // Format lines look like: "  E  mp4             MP4 (MPEG-4 Part 14)"
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let _flags = parts.next();
        if let Some(rest) = parts.next() {
            let rest = rest.trim_start();
            let mut inner = rest.splitn(2, char::is_whitespace);
            if let Some(name) = inner.next() {
                let desc = inner.next().unwrap_or("").trim().to_string();
                if !name.is_empty() {
                    out.push(FormatCapability {
                        name: name.to_string(),
                        description: desc,
                    });
                }
            }
        }
    }
    out
}

pub fn parse_codec_list(raw: &str) -> Vec<CodecCapability> {
    let mut out = Vec::new();
    let mut past_divider = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("------") {
            past_divider = true;
            continue;
        }
        if !past_divider {
            continue;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let Some(flags) = parts.next() else {
            continue;
        };
        if flags.len() < 6 {
            continue;
        }
        let media_type = match flags.chars().next() {
            Some('V') => MediaKind::Video,
            Some('A') => MediaKind::Audio,
            Some('S') => MediaKind::Subtitle,
            _ => continue,
        };
        let frame_mt = flags.chars().nth(1) == Some('F');
        let slice_mt = flags.chars().nth(2) == Some('S');
        let experimental = flags.chars().nth(3) == Some('X');

        let Some(rest) = parts.next() else {
            continue;
        };
        let rest = rest.trim_start();
        let mut inner = rest.splitn(2, char::is_whitespace);
        let Some(name) = inner.next() else {
            continue;
        };
        let desc = inner.next().unwrap_or("").trim();
        let codec_family = extract_codec_family(name, desc);
        let is_hardware = name.ends_with("_nvenc")
            || name.ends_with("_qsv")
            || name.ends_with("_amf")
            || name.ends_with("_mf")
            || name.ends_with("_vaapi")
            || name.ends_with("_videotoolbox")
            || name.ends_with("_cuvid");

        out.push(CodecCapability {
            name: name.to_string(),
            codec_family,
            media_type,
            description: desc.to_string(),
            is_hardware,
            experimental,
            frame_mt,
            slice_mt,
        });
    }
    out
}

fn extract_codec_family(name: &str, desc: &str) -> String {
    if let Some(start) = desc.rfind("(codec ")
        && let Some(end) = desc[start..].find(')')
    {
        return desc[start + 7..start + end].trim().to_string();
    }
    name.to_string()
}

/// Parses output from `ffmpeg -filters` into structured filter descriptors.
#[must_use]
pub fn parse_filter_list(raw: &str) -> Vec<FilterCapability> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Filters:") || trimmed.contains(" = ") {
            continue;
        }
        let mut iter = trimmed.split_whitespace();
        let Some(_flags) = iter.next() else {
            continue;
        };
        let Some(name) = iter.next() else {
            continue;
        };
        let Some(io_sig) = iter.next() else {
            continue;
        };
        if io_sig.contains("->") {
            let description: String = iter.collect::<Vec<_>>().join(" ");
            out.push(FilterCapability {
                name: name.to_string(),
                io_signature: io_sig.to_string(),
                description,
            });
        }
    }
    out
}

pub fn parse_hwaccel_list(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.ends_with(':'))
        .map(ToString::to_string)
        .collect()
}
