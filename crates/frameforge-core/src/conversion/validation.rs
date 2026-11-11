use crate::core::models::{
    AudioMode, CapabilityRegistry, ConversionSettings, DecisionKind, MediaFile, QualityImpact,
    RateControlMode, ResolutionPreset, SubtitleMode,
};
use crate::hardware::encoders::{SelectedEncoder, select_video_encoder};
use crate::media::formats::{
    find_container, is_audio_stream_copy_compatible_with_container,
    is_video_stream_copy_compatible_with_container, normalize_codec_family,
    preferred_audio_encoder,
};
use crate::media::streams::find_video_stream_by_index;

#[derive(Debug, Clone)]
pub struct ValidatedEvaluation {
    pub decision: DecisionKind,
    pub quality_impact: QualityImpact,
    pub summary: String,
    pub quality_explanation: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub muxer: String,
    pub extension: String,
    pub audio_only: bool,
    pub supports_faststart: bool,
    pub selected_video_encoder: Option<SelectedEncoder>,
    pub resolved_audio_encoder: Option<String>,
    pub selected_video_stream_index: Option<usize>,
    pub selected_audio_indices: Vec<usize>,
}

/// Validates whether an expert parameter value contains only safe `FFmpeg` CLI token characters.
#[must_use]
pub fn is_safe_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+' | ':'))
}

/// Evaluates compatibility, quality preservation, and warning/failure diagnostics before running `FFmpeg`.
#[must_use]
pub fn evaluate_conversion_settings(
    media: &MediaFile,
    settings: &ConversionSettings,
    registry: &CapabilityRegistry,
) -> ValidatedEvaluation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if !registry.ffmpeg.available {
        errors.push("FFmpeg executable is unavailable or failed version check.".to_string());
    }

    let Some(container_def) = find_container(&settings.container) else {
        errors.push(format!(
            "Unsupported target container '{}'.",
            settings.container
        ));
        return ValidatedEvaluation {
            decision: DecisionKind::Fail,
            quality_impact: QualityImpact::Incompatible,
            summary: format!("Unsupported container '{}'", settings.container),
            quality_explanation: "Conversion cannot proceed without a valid output container."
                .to_string(),
            warnings,
            errors,
            muxer: settings.container.clone(),
            extension: settings.container.clone(),
            audio_only: false,
            supports_faststart: false,
            selected_video_encoder: None,
            resolved_audio_encoder: None,
            selected_video_stream_index: None,
            selected_audio_indices: Vec::new(),
        };
    };

    if !registry
        .muxers
        .iter()
        .any(|m| m.name == container_def.muxer)
    {
        errors.push(format!(
            "Installed FFmpeg build does not support muxer '{}' for {}.",
            container_def.muxer, container_def.label
        ));
    }

    validate_expert_inputs(settings, &mut errors, &mut warnings);
    let selected_audio_indices = resolve_selected_audio_indices(media, settings);

    if container_def.audio_only {
        return evaluate_audio_only(
            media,
            settings,
            container_def,
            selected_audio_indices,
            registry,
            errors,
            warnings,
        );
    }

    evaluate_video_container(
        media,
        settings,
        container_def,
        selected_audio_indices,
        registry,
        errors,
        warnings,
    )
}

fn validate_expert_inputs(
    settings: &ConversionSettings,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    for (field, opt) in [
        ("profile", settings.profile.as_deref()),
        ("level", settings.level.as_deref()),
        ("tune", settings.tune.as_deref()),
        ("colorRange", settings.color_range.as_deref()),
        ("pixelFormat", settings.pixel_format.as_deref()),
        ("encoderPreset", settings.encoder_preset.as_deref()),
    ] {
        if let Some(val) = opt.map(str::trim).filter(|s| !s.is_empty())
            && !is_safe_token(val)
        {
            errors.push(format!(
                "Field '{field}' contains invalid characters; only alphanumeric and . _ - + : are allowed."
            ));
        }
    }

    if let Some(fps) = settings
        .target_fps
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let valid_fps = fps.split_once('/').map_or_else(
            || fps.parse::<f64>().is_ok_and(|v| v > 0.0 && v <= 1000.0),
            |(n, d)| n.parse::<f64>().is_ok() && d.parse::<f64>().is_ok(),
        );
        if !valid_fps {
            errors.push(format!("Invalid target frame rate '{fps}'."));
        }
    }

    if settings.resolution == ResolutionPreset::Custom {
        match (settings.custom_width, settings.custom_height) {
            (Some(w), Some(h)) if (16..=16384).contains(&w) && (16..=16384).contains(&h) => {
                if w % 2 != 0 || h % 2 != 0 {
                    warnings.push("Odd custom dimensions may fail on YUV 4:2:0 encoders; even dimensions are recommended.".to_string());
                }
            }
            _ => errors
                .push("Custom resolution requires valid width and height (16..16384).".to_string()),
        }
    }
}

fn resolve_selected_audio_indices(media: &MediaFile, settings: &ConversionSettings) -> Vec<usize> {
    if settings.audio_mode == AudioMode::Strip {
        return Vec::new();
    }
    match &settings.audio_stream_indices {
        Some(explicit) if !explicit.is_empty() => explicit
            .iter()
            .copied()
            .filter(|idx| media.audio_streams.iter().any(|a| a.index == *idx))
            .collect(),
        _ => media.audio_streams.iter().map(|a| a.index).collect(),
    }
}

fn evaluate_audio_only(
    media: &MediaFile,
    settings: &ConversionSettings,
    container_def: &crate::media::formats::ContainerDefinition,
    selected_audio_indices: Vec<usize>,
    registry: &CapabilityRegistry,
    mut errors: Vec<String>,
    warnings: Vec<String>,
) -> ValidatedEvaluation {
    if media.audio_streams.is_empty() {
        errors.push("Input file has no audio streams to extract.".to_string());
    } else if settings.audio_mode == AudioMode::Strip {
        errors.push("Cannot strip audio when exporting to an audio-only container.".to_string());
    }

    let resolved_audio_encoder = resolve_audio_selection(
        media,
        settings,
        container_def,
        &selected_audio_indices,
        registry,
        &mut errors,
    );

    let (decision, quality_impact, summary) = if let Some(first_err) = errors.first() {
        (
            DecisionKind::Fail,
            QualityImpact::Incompatible,
            first_err.clone(),
        )
    } else if settings.audio_mode == AudioMode::Copy {
        (
            DecisionKind::Remux,
            QualityImpact::AudioOnlyExtraction,
            format!("Audio-only stream copy into {}", container_def.label),
        )
    } else {
        (
            DecisionKind::Transcode,
            QualityImpact::AudioOnlyExtraction,
            format!(
                "Extract & transcode audio ({}) to {}",
                resolved_audio_encoder.as_deref().unwrap_or("audio"),
                container_def.label
            ),
        )
    };

    ValidatedEvaluation {
        decision,
        quality_impact,
        summary,
        quality_explanation:
            "Audio-only extraction drops video tracks; video stream is not re-encoded or included."
                .to_string(),
        warnings,
        errors,
        muxer: container_def.muxer.to_string(),
        extension: container_def.extension.to_string(),
        audio_only: true,
        supports_faststart: container_def.supports_faststart,
        selected_video_encoder: None,
        resolved_audio_encoder,
        selected_video_stream_index: None,
        selected_audio_indices,
    }
}

fn evaluate_video_container(
    media: &MediaFile,
    settings: &ConversionSettings,
    container_def: &crate::media::formats::ContainerDefinition,
    selected_audio_indices: Vec<usize>,
    registry: &CapabilityRegistry,
    mut errors: Vec<String>,
    mut warnings: Vec<String>,
) -> ValidatedEvaluation {
    let primary_video =
        find_video_stream_by_index(&media.video_streams, settings.video_stream_index);
    let selected_video_stream_index = primary_video.map(|v| v.index);
    if primary_video.is_none() {
        errors.push("Input file contains no video stream.".to_string());
    }

    let video_codec_norm = normalize_codec_family(&settings.video_codec);
    let is_video_copy = video_codec_norm == "copy";
    let selected_video_encoder = validate_video_selection(
        primary_video,
        &video_codec_norm,
        is_video_copy,
        settings,
        container_def,
        registry,
        &mut errors,
        &mut warnings,
    );

    let resolved_audio_encoder =
        if settings.audio_mode == AudioMode::Strip || media.audio_streams.is_empty() {
            None
        } else {
            resolve_audio_selection(
                media,
                settings,
                container_def,
                &selected_audio_indices,
                registry,
                &mut errors,
            )
        };

    check_subtitle_notes(media, settings, container_def, &mut warnings);

    let (decision, quality_impact, summary, quality_explanation) = classify_video_decision(
        media,
        settings,
        container_def,
        &video_codec_norm,
        is_video_copy,
        selected_video_encoder.as_ref(),
        resolved_audio_encoder.as_deref(),
        &errors,
    );

    ValidatedEvaluation {
        decision,
        quality_impact,
        summary,
        quality_explanation,
        warnings,
        errors,
        muxer: container_def.muxer.to_string(),
        extension: container_def.extension.to_string(),
        audio_only: false,
        supports_faststart: container_def.supports_faststart,
        selected_video_encoder,
        resolved_audio_encoder,
        selected_video_stream_index,
        selected_audio_indices,
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_video_selection(
    primary_video: Option<&crate::core::models::VideoStreamInfo>,
    video_codec_norm: &str,
    is_video_copy: bool,
    settings: &ConversionSettings,
    container_def: &crate::media::formats::ContainerDefinition,
    registry: &CapabilityRegistry,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Option<SelectedEncoder> {
    if is_video_copy {
        if let Some(v) = primary_video
            && !is_video_stream_copy_compatible_with_container(&v.codec_name, container_def.id)
        {
            errors.push(format!(
                "Cannot stream-copy '{}' video into {} container. Select a compatible encoder ({}) or use MKV.",
                v.codec_name,
                container_def.label,
                container_def
                    .compatible_video_codecs
                    .iter()
                    .filter(|c| **c != "copy")
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if settings.resolution != ResolutionPreset::Original
            || settings
                .target_fps
                .as_deref()
                .map(str::trim)
                .is_some_and(|s| !s.is_empty())
            || settings
                .pixel_format
                .as_deref()
                .map(str::trim)
                .is_some_and(|s| !s.is_empty())
        {
            errors.push(
                "Stream copy ('copy') cannot resize video, change frame rate, or convert pixel format. Choose a video encoder or reset filters to Original."
                    .to_string(),
            );
        }
        return None;
    }

    if !container_def
        .compatible_video_codecs
        .contains(&video_codec_norm)
    {
        errors.push(format!(
            "Video codec '{}' is incompatible with {} container.",
            settings.video_codec, container_def.label
        ));
        return None;
    }

    match select_video_encoder(
        video_codec_norm,
        settings.video_encoder.as_deref(),
        settings.hardware_mode,
        settings.rate_control,
        settings.allow_cpu_fallback,
        registry,
    ) {
        Ok(sel) => {
            warnings.extend(sel.notes.clone());
            if let Some(v) = primary_video {
                if v.bit_depth.unwrap_or(8) >= 10
                    && sel.encoder_name == "libx264"
                    && settings.pixel_format.is_none()
                {
                    warnings.push(
                        "Source is 10-bit; standard libx264 will convert to 8-bit unless a 10-bit pixel format / HEVC / AV1 encoder is used."
                            .to_string(),
                    );
                }
                if v.hdr_format.is_some()
                    && !matches!(video_codec_norm, "hevc" | "av1" | "prores" | "ffv1")
                {
                    warnings.push(format!(
                        "Source contains {} HDR metadata; encoding with '{}' without tone-mapping may alter HDR colors.",
                        v.hdr_format.as_deref().unwrap_or("HDR"),
                        sel.encoder_name
                    ));
                }
            }
            Some(sel)
        }
        Err(msg) => {
            errors.push(msg);
            None
        }
    }
}

fn check_subtitle_notes(
    media: &MediaFile,
    settings: &ConversionSettings,
    container_def: &crate::media::formats::ContainerDefinition,
    warnings: &mut Vec<String>,
) {
    if media.subtitle_streams.is_empty() {
        return;
    }
    match settings.subtitle_mode {
        SubtitleMode::CopyCompatible => {
            let has_bitmap = media.subtitle_streams.iter().any(|s| {
                matches!(
                    s.codec_name.as_str(),
                    "hdmv_pgs_subtitle" | "dvd_subtitle" | "dvb_subtitle"
                )
            });
            if (container_def.id == "mp4" || container_def.id == "mov") && has_bitmap {
                warnings.push(
                    "Bitmap subtitles (PGS/VobSub) cannot be stream-copied into MP4/MOV and will be omitted unless MKV is selected."
                        .to_string(),
                );
            } else if container_def.id == "mp4"
                && media
                    .subtitle_streams
                    .iter()
                    .any(|s| !matches!(s.codec_name.as_str(), "mov_text" | "tx3g"))
            {
                warnings.push(
                    "Non-MP4 text subtitles (SRT/ASS) cannot be copied raw into MP4; switch Subtitles to 'Convert to mov_text' or 'Strip', or use MKV."
                        .to_string(),
                );
            } else if container_def.id == "webm" {
                warnings.push("WebM only supports WebVTT subtitles; incompatible subtitle tracks will be omitted.".to_string());
            }
        }
        SubtitleMode::ConvertMovText => {
            if container_def.id != "mp4" && container_def.id != "mov" {
                warnings.push(
                    "mov_text subtitle conversion is intended for MP4/MOV; subtitles will be copied/omitted as appropriate."
                        .to_string(),
                );
            }
        }
        SubtitleMode::Strip => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_video_decision(
    media: &MediaFile,
    settings: &ConversionSettings,
    container_def: &crate::media::formats::ContainerDefinition,
    video_codec_norm: &str,
    is_video_copy: bool,
    selected_video_encoder: Option<&SelectedEncoder>,
    resolved_audio_encoder: Option<&str>,
    errors: &[String],
) -> (DecisionKind, QualityImpact, String, String) {
    if let Some(first_err) = errors.first() {
        return (
            DecisionKind::Fail,
            QualityImpact::Incompatible,
            first_err.clone(),
            "Fix the compatibility issues above before running this job.".to_string(),
        );
    }

    if is_video_copy
        && (settings.audio_mode == AudioMode::Copy
            || settings.audio_mode == AudioMode::Strip
            || media.audio_streams.is_empty())
    {
        let same_container =
            crate::conversion::remuxer::container_matches_source(media, container_def.id);
        let dec = if same_container {
            DecisionKind::Copy
        } else {
            DecisionKind::Remux
        };
        let label = if dec == DecisionKind::Copy {
            "COPY (Direct Stream Copy)"
        } else {
            "REMUX (Lossless Container Change)"
        };
        return (
            dec,
            QualityImpact::LosslessStreamCopy,
            format!("{label} → {}", container_def.label),
            "Video frames are copied bit-for-bit without decoding or re-encoding. Zero generational quality loss."
                .to_string(),
        );
    }

    if is_video_copy {
        return (
            DecisionKind::Transcode,
            QualityImpact::LosslessStreamCopy,
            format!(
                "REMUX Video + TRANSCODE Audio ({}) → {}",
                resolved_audio_encoder.unwrap_or("audio"),
                container_def.label
            ),
            "Video stream is copied losslessly without re-encoding; only audio tracks are re-encoded."
                .to_string(),
        );
    }

    let enc_name = selected_video_encoder.map_or("encoder", |e| e.encoder_name.as_str());
    let is_lossless = settings.rate_control == RateControlMode::Lossless
        || video_codec_norm == "ffv1"
        || (enc_name == "libx264"
            && settings.rate_control == RateControlMode::Crf
            && settings.quality_value == 0);
    let qi = if is_lossless {
        QualityImpact::LosslessReencode
    } else {
        QualityImpact::QualityAltered
    };
    let qual_msg = if is_lossless {
        format!("Video is re-encoded mathematically losslessly via {enc_name}.")
    } else {
        format!(
            "Video will be decoded and re-encoded with {enc_name}, altering compression and potentially reducing visual fidelity depending on quality/bitrate."
        )
    };
    (
        DecisionKind::Transcode,
        qi,
        format!("TRANSCODE ({enc_name}) → {}", container_def.label),
        qual_msg,
    )
}

fn resolve_audio_selection(
    media: &MediaFile,
    settings: &ConversionSettings,
    container_def: &crate::media::formats::ContainerDefinition,
    selected_audio_indices: &[usize],
    registry: &CapabilityRegistry,
    errors: &mut Vec<String>,
) -> Option<String> {
    match settings.audio_mode {
        AudioMode::Strip => None,
        AudioMode::Copy => {
            for idx in selected_audio_indices {
                if let Some(a) = media.audio_streams.iter().find(|s| s.index == *idx)
                    && !is_audio_stream_copy_compatible_with_container(
                        &a.codec_name,
                        container_def.id,
                    )
                {
                    errors.push(format!(
                        "Audio stream #{} ('{}') cannot be stream-copied into {}. Switch Audio Mode to Transcode or Strip.",
                        a.index, a.codec_name, container_def.label
                    ));
                }
            }
            Some("copy".to_string())
        }
        AudioMode::Transcode => {
            let requested = settings.audio_codec.as_deref().map_or_else(
                || {
                    container_def
                        .compatible_audio_codecs
                        .iter()
                        .find(|c| **c != "copy")
                        .copied()
                        .unwrap_or("aac")
                        .to_string()
                },
                normalize_codec_family,
            );

            if !container_def
                .compatible_audio_codecs
                .contains(&requested.as_str())
            {
                errors.push(format!(
                    "Audio codec '{requested}' is incompatible with {}.",
                    container_def.label
                ));
                return None;
            }

            if let Some(enc) = preferred_audio_encoder(&requested, &registry.encoders) {
                Some(enc)
            } else {
                errors.push(format!(
                    "No FFmpeg audio encoder available for '{requested}'."
                ));
                None
            }
        }
    }
}
