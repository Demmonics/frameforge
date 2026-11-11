use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::core::models::{
    AudioStreamInfo, ContainerInfo, HdrMetadata, MediaFile, SubtitleStreamInfo, VideoStreamInfo,
};
use crate::media::streams::{select_primary_audio_stream, select_primary_video_stream};

#[derive(Debug, Deserialize)]
pub struct RawFfprobeOutput {
    #[serde(default)]
    pub streams: Vec<RawFfprobeStream>,
    #[serde(default)]
    pub chapters: Vec<serde_json::Value>,
    pub format: Option<RawFfprobeFormat>,
}

#[derive(Debug, Deserialize)]
pub struct RawFfprobeFormat {
    pub filename: Option<String>,
    pub format_name: Option<String>,
    pub format_long_name: Option<String>,
    pub duration: Option<String>,
    pub size: Option<String>,
    pub bit_rate: Option<String>,
    pub probe_score: Option<i32>,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct RawDisposition {
    #[serde(default)]
    pub default: i32,
    #[serde(default)]
    pub forced: i32,
    #[serde(default)]
    pub attached_pic: i32,
}

#[derive(Debug, Deserialize)]
pub struct RawFfprobeStream {
    pub index: usize,
    pub codec_name: Option<String>,
    pub codec_long_name: Option<String>,
    pub profile: Option<String>,
    pub codec_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub coded_width: Option<u32>,
    pub coded_height: Option<u32>,
    pub sample_aspect_ratio: Option<String>,
    pub display_aspect_ratio: Option<String>,
    pub pix_fmt: Option<String>,
    pub level: Option<i32>,
    pub field_order: Option<String>,
    pub r_frame_rate: Option<String>,
    pub avg_frame_rate: Option<String>,
    pub bit_rate: Option<String>,
    pub bits_per_raw_sample: Option<String>,
    pub nb_frames: Option<String>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub sample_fmt: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    pub disposition: Option<RawDisposition>,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    #[serde(default)]
    pub side_data_list: Vec<serde_json::Value>,
}

#[must_use]
pub fn parse_rational_fps(raw: Option<&str>) -> Option<f64> {
    let text = raw?.trim();
    if text.is_empty() || text == "0/0" {
        return None;
    }
    if let Some((num_s, den_s)) = text.split_once('/') {
        let num: f64 = num_s.trim().parse().ok()?;
        let den: f64 = den_s.trim().parse().ok()?;
        if den <= 0.0 || num <= 0.0 {
            return None;
        }
        let fps = num / den;
        if fps.is_finite() && fps > 0.0 && fps < 10_000.0 {
            Some((fps * 1000.0).round() / 1000.0)
        } else {
            None
        }
    } else {
        let fps: f64 = text.parse().ok()?;
        if fps.is_finite() && fps > 0.0 && fps < 10_000.0 {
            Some(fps)
        } else {
            None
        }
    }
}

#[must_use]
pub fn detect_vfr(r_fps: Option<f64>, avg_fps: Option<f64>) -> (bool, Option<String>) {
    match (r_fps, avg_fps) {
        (Some(nominal), Some(avg)) if nominal > 0.0 && avg > 0.0 => {
            let rel_diff = (nominal - avg).abs() / nominal;
            if rel_diff > 0.015 {
                (
                    true,
                    Some(format!(
                        "Variable frame rate indicated (avg {avg:.3} fps vs nominal {nominal:.3} fps)"
                    )),
                )
            } else {
                (false, None)
            }
        }
        _ => (false, None),
    }
}

#[must_use]
pub fn infer_bit_depth(bits_per_raw_sample: Option<&str>, pix_fmt: Option<&str>) -> Option<u8> {
    if let Some(raw) = bits_per_raw_sample
        && let Ok(parsed) = raw.trim().parse::<u8>()
        && parsed > 0
    {
        return Some(parsed);
    }
    let fmt = pix_fmt?.to_ascii_lowercase();
    if fmt.contains("16le") || fmt.contains("16be") || fmt.contains("p016") {
        Some(16)
    } else if fmt.contains("12le") || fmt.contains("12be") {
        Some(12)
    } else if fmt.contains("10le")
        || fmt.contains("10be")
        || fmt.contains("p010")
        || fmt.contains("p210")
    {
        Some(10)
    } else {
        Some(8)
    }
}

#[must_use]
pub fn extract_rotation_degrees(
    tags: &BTreeMap<String, String>,
    side_data_list: &[serde_json::Value],
) -> Option<i32> {
    for sd in side_data_list {
        if let Some(rot) = sd.get("rotation").and_then(serde_json::Value::as_f64)
            && rot.is_finite()
        {
            #[allow(clippy::cast_possible_truncation)]
            let rounded = rot.round().clamp(-360.0, 360.0) as i32;
            return Some(rounded);
        }
    }
    if let Some(rot_tag) = find_tag_case_insensitive(tags, "rotate")
        && let Ok(deg) = rot_tag.trim().parse::<i32>()
    {
        return Some(deg);
    }
    None
}

#[must_use]
pub fn extract_hdr_info(
    color_transfer: Option<&str>,
    color_primaries: Option<&str>,
    side_data_list: &[serde_json::Value],
) -> (Option<String>, Option<HdrMetadata>) {
    let mut mastering_display = None;
    let mut max_cll = None;
    let mut max_fall = None;

    for sd in side_data_list {
        let side_type = sd
            .get("side_data_type")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if side_type.contains("mastering display") {
            let min_l = sd.get("min_luminance").and_then(|v| v.as_str());
            let max_l = sd.get("max_luminance").and_then(|v| v.as_str());
            mastering_display = Some(match (min_l, max_l) {
                (Some(min_v), Some(max_v)) => format!("L(min={min_v}, max={max_v})"),
                _ => "Present".to_string(),
            });
        }
        if side_type.contains("content light level") {
            max_cll = sd
                .get("max_content")
                .and_then(serde_json::Value::as_u64)
                .and_then(|n| u32::try_from(n).ok());
            max_fall = sd
                .get("max_average")
                .and_then(serde_json::Value::as_u64)
                .and_then(|n| u32::try_from(n).ok());
        }
    }

    let transfer = color_transfer.unwrap_or_default().to_ascii_lowercase();
    let primaries = color_primaries.unwrap_or_default().to_ascii_lowercase();

    let hdr_format = if transfer == "smpte2084" {
        if max_cll.is_some() || mastering_display.is_some() {
            Some("HDR10 (PQ / SMPTE ST 2084)".to_string())
        } else {
            Some("PQ (SMPTE ST 2084)".to_string())
        }
    } else if transfer == "arib-std-b67" {
        Some("HLG (ARIB STD-B67)".to_string())
    } else if primaries == "bt2020" && (mastering_display.is_some() || max_cll.is_some()) {
        Some("BT.2020 HDR".to_string())
    } else {
        None
    };

    let metadata = if mastering_display.is_some() || max_cll.is_some() || max_fall.is_some() {
        Some(HdrMetadata {
            mastering_display,
            max_cll,
            max_fall,
        })
    } else {
        None
    };

    (hdr_format, metadata)
}

#[must_use]
pub fn find_tag_case_insensitive<'a>(
    tags: &'a BTreeMap<String, String>,
    key: &str,
) -> Option<&'a String> {
    tags.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v)
}

#[must_use]
pub fn normalize_probe_output(
    id: String,
    file_name: String,
    abs_path: String,
    size_bytes: u64,
    raw: RawFfprobeOutput,
    raw_probe_json: String,
    probed_at_utc: String,
) -> MediaFile {
    let format = raw.format.as_ref();
    let container = ContainerInfo {
        format_name: format
            .and_then(|f| f.format_name.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        format_long_name: format
            .and_then(|f| f.format_long_name.clone())
            .unwrap_or_else(|| "Unknown container".to_string()),
        probe_score: format.and_then(|f| f.probe_score),
    };

    let duration_secs = format
        .and_then(|f| f.duration.as_deref())
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|d| d.is_finite() && *d >= 0.0);

    let format_bit_rate = format
        .and_then(|f| f.bit_rate.as_deref())
        .and_then(|s| s.parse::<u64>().ok());

    let format_tags = format.map(|f| f.tags.clone()).unwrap_or_default();
    let creation_time = find_tag_case_insensitive(&format_tags, "creation_time").cloned();
    let format_encoder = find_tag_case_insensitive(&format_tags, "encoder").cloned();

    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();
    let mut subtitle_streams = Vec::new();

    for s in raw.streams {
        let disp = s.disposition.as_ref();
        let is_default = disp.is_some_and(|d| d.default == 1);
        let is_forced = disp.is_some_and(|d| d.forced == 1);
        let is_attached_pic = disp.is_some_and(|d| d.attached_pic == 1);

        match s.codec_type.as_deref().unwrap_or_default() {
            "video" => video_streams.push(parse_video_stream(s, is_default, is_attached_pic)),
            "audio" => audio_streams.push(parse_audio_stream(s, is_default)),
            "subtitle" => subtitle_streams.push(parse_subtitle_stream(s, is_default, is_forced)),
            _ => {}
        }
    }

    let primary_video = select_primary_video_stream(&video_streams).cloned();
    let primary_audio = select_primary_audio_stream(&audio_streams).cloned();
    let encoder_tag =
        format_encoder.or_else(|| primary_video.as_ref().and_then(|v| v.encoder.clone()));

    let mut media = MediaFile {
        id,
        file_name,
        path: abs_path,
        size_bytes,
        container,
        duration_secs,
        bit_rate: format_bit_rate,
        primary_video,
        primary_audio,
        video_streams,
        audio_streams,
        subtitle_streams,
        chapters_count: raw.chapters.len(),
        creation_time,
        encoder_tag,
        tags: format_tags,
        thumbnail_data_url: None,
        thumbnail_cache_path: None,
        technical_report: String::new(),
        raw_probe_json,
        probed_at_utc,
    };

    media.technical_report = build_technical_report(&media);
    media
}

fn parse_video_stream(
    s: RawFfprobeStream,
    is_default: bool,
    is_attached_pic: bool,
) -> VideoStreamInfo {
    let fps_nominal = parse_rational_fps(s.r_frame_rate.as_deref());
    let fps_average = parse_rational_fps(s.avg_frame_rate.as_deref());
    let (is_vfr, vfr_note) = detect_vfr(fps_nominal, fps_average);
    let bit_depth = infer_bit_depth(s.bits_per_raw_sample.as_deref(), s.pix_fmt.as_deref());
    let rotation_degrees = extract_rotation_degrees(&s.tags, &s.side_data_list);
    let (hdr_format, hdr_metadata) = extract_hdr_info(
        s.color_transfer.as_deref(),
        s.color_primaries.as_deref(),
        &s.side_data_list,
    );

    VideoStreamInfo {
        index: s.index,
        codec_name: s.codec_name.unwrap_or_else(|| "unknown".to_string()),
        codec_long_name: s.codec_long_name.unwrap_or_default(),
        profile: s.profile,
        level: s.level,
        width: s.width.unwrap_or(0),
        height: s.height.unwrap_or(0),
        coded_width: s.coded_width,
        coded_height: s.coded_height,
        sample_aspect_ratio: s.sample_aspect_ratio,
        display_aspect_ratio: s.display_aspect_ratio,
        pix_fmt: s.pix_fmt,
        bit_depth,
        r_frame_rate: s.r_frame_rate,
        avg_frame_rate: s.avg_frame_rate,
        fps_nominal,
        fps_average,
        is_vfr,
        vfr_note,
        bit_rate: s.bit_rate.as_deref().and_then(|b| b.parse::<u64>().ok()),
        nb_frames: s.nb_frames.as_deref().and_then(|n| n.parse::<u64>().ok()),
        color_primaries: s.color_primaries,
        color_transfer: s.color_transfer,
        color_space: s.color_space,
        color_range: s.color_range,
        hdr_format,
        hdr_metadata,
        rotation_degrees,
        field_order: s.field_order,
        language: find_tag_case_insensitive(&s.tags, "language").cloned(),
        title: find_tag_case_insensitive(&s.tags, "title").cloned(),
        encoder: find_tag_case_insensitive(&s.tags, "encoder").cloned(),
        is_default,
        is_attached_pic,
        tags: s.tags,
    }
}

fn parse_audio_stream(s: RawFfprobeStream, is_default: bool) -> AudioStreamInfo {
    AudioStreamInfo {
        index: s.index,
        codec_name: s.codec_name.unwrap_or_else(|| "unknown".to_string()),
        codec_long_name: s.codec_long_name.unwrap_or_default(),
        profile: s.profile,
        sample_rate: s.sample_rate.as_deref().and_then(|r| r.parse::<u32>().ok()),
        channels: s.channels,
        channel_layout: s.channel_layout,
        sample_fmt: s.sample_fmt,
        bit_rate: s.bit_rate.as_deref().and_then(|b| b.parse::<u64>().ok()),
        language: find_tag_case_insensitive(&s.tags, "language").cloned(),
        title: find_tag_case_insensitive(&s.tags, "title").cloned(),
        is_default,
        tags: s.tags,
    }
}

fn parse_subtitle_stream(
    s: RawFfprobeStream,
    is_default: bool,
    is_forced: bool,
) -> SubtitleStreamInfo {
    SubtitleStreamInfo {
        index: s.index,
        codec_name: s.codec_name.unwrap_or_else(|| "unknown".to_string()),
        codec_long_name: s.codec_long_name.unwrap_or_default(),
        language: find_tag_case_insensitive(&s.tags, "language").cloned(),
        title: find_tag_case_insensitive(&s.tags, "title").cloned(),
        is_default,
        is_forced,
        tags: s.tags,
    }
}

#[must_use]
pub fn build_technical_report(media: &MediaFile) -> String {
    let mut out = String::new();
    append_general_report(&mut out, media);
    for (idx, v) in media.video_streams.iter().enumerate() {
        append_video_report(&mut out, idx, v);
    }
    for (idx, a) in media.audio_streams.iter().enumerate() {
        append_audio_report(&mut out, idx, a);
    }
    for (idx, s) in media.subtitle_streams.iter().enumerate() {
        let _ = writeln!(out, "\n=== SUBTITLE #{} (Stream #{}) ===", idx + 1, s.index);
        let _ = writeln!(out, "Codec            : {}", s.codec_name);
        if let Some(lang) = &s.language {
            let _ = writeln!(out, "Language         : {lang}");
        }
        if let Some(title) = &s.title {
            let _ = writeln!(out, "Title            : {title}");
        }
    }
    out
}

fn append_general_report(out: &mut String, media: &MediaFile) {
    let _ = writeln!(out, "=== GENERAL ===");
    let _ = writeln!(out, "File Name        : {}", media.file_name);
    let _ = writeln!(out, "Full Path        : {}", media.path);
    let _ = writeln!(out, "File Size        : {} bytes", media.size_bytes);
    let _ = writeln!(
        out,
        "Container        : {} ({})",
        media.container.format_long_name, media.container.format_name
    );
    if let Some(dur) = media.duration_secs {
        let _ = writeln!(out, "Duration         : {dur:.3} s");
    }
    if let Some(br) = media.bit_rate {
        let _ = writeln!(out, "Overall Bitrate  : {} kbps", br / 1000);
    }
    if let Some(ct) = &media.creation_time {
        let _ = writeln!(out, "Creation Time    : {ct}");
    }
    if let Some(enc) = &media.encoder_tag {
        let _ = writeln!(out, "Writing App      : {enc}");
    }
}

fn append_video_report(out: &mut String, idx: usize, v: &VideoStreamInfo) {
    let _ = writeln!(out, "\n=== VIDEO #{} (Stream #{}) ===", idx + 1, v.index);
    let _ = writeln!(
        out,
        "Codec            : {} ({})",
        v.codec_name,
        v.profile.as_deref().unwrap_or("n/a")
    );
    let _ = writeln!(out, "Resolution       : {}x{}", v.width, v.height);
    if let Some(dar) = &v.display_aspect_ratio {
        let _ = writeln!(
            out,
            "Aspect Ratio     : DAR {} / SAR {}",
            dar,
            v.sample_aspect_ratio.as_deref().unwrap_or("n/a")
        );
    }
    if let Some(fps) = v.fps_nominal {
        let _ = writeln!(
            out,
            "Frame Rate       : {:.3} fps (avg {:.3} fps{})",
            fps,
            v.fps_average.unwrap_or(fps),
            if v.is_vfr { ", VFR" } else { ", CFR" }
        );
    }
    if let Some(pf) = &v.pix_fmt {
        let _ = writeln!(
            out,
            "Pixel Format     : {} ({}-bit)",
            pf,
            v.bit_depth.unwrap_or(8)
        );
    }
    if v.color_primaries.is_some() || v.color_transfer.is_some() {
        let _ = writeln!(
            out,
            "Color / HDR      : primaries={} transfer={} space={} hdr={}",
            v.color_primaries.as_deref().unwrap_or("n/a"),
            v.color_transfer.as_deref().unwrap_or("n/a"),
            v.color_space.as_deref().unwrap_or("n/a"),
            v.hdr_format.as_deref().unwrap_or("SDR")
        );
    }
    if let Some(rot) = v.rotation_degrees {
        let _ = writeln!(out, "Rotation         : {rot}°");
    }
    if let Some(br) = v.bit_rate {
        let _ = writeln!(out, "Bitrate          : {} kbps", br / 1000);
    }
}

fn append_audio_report(out: &mut String, idx: usize, a: &AudioStreamInfo) {
    let _ = writeln!(out, "\n=== AUDIO #{} (Stream #{}) ===", idx + 1, a.index);
    let _ = writeln!(out, "Codec            : {}", a.codec_name);
    let _ = writeln!(
        out,
        "Sample Rate      : {} Hz",
        a.sample_rate
            .map_or_else(|| "n/a".into(), |r| r.to_string())
    );
    let _ = writeln!(
        out,
        "Channels         : {} ({})",
        a.channels.map_or_else(|| "n/a".into(), |c| c.to_string()),
        a.channel_layout.as_deref().unwrap_or("n/a")
    );
    if let Some(br) = a.bit_rate {
        let _ = writeln!(out, "Bitrate          : {} kbps", br / 1000);
    }
    if let Some(lang) = &a.language {
        let _ = writeln!(out, "Language         : {lang}");
    }
}
