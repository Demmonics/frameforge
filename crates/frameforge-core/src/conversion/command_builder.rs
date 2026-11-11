use std::ffi::OsString;
use std::path::Path;

use crate::conversion::validation::evaluate_conversion_settings;
use crate::core::errors::Result;
use crate::core::models::{
    AudioMode, CapabilityRegistry, ConversionPlan, ConversionSettings, DecisionKind, MediaFile,
    RateControlMode, ResolutionPreset, SubtitleMode,
};
use crate::hardware::encoders::{encoder_supports_crf, is_amf, is_nvenc, is_qsv};
use crate::media::formats::normalize_codec_family;
use crate::storage::output_manager::resolve_output_path;

/// Validates settings against `registry` and constructs the conversion preview plan.
///
/// # Errors
/// Currently infallible (`Ok(ConversionPlan)`); returns `Result` to reserve future strict planning errors.
pub fn build_conversion_plan(
    media: &MediaFile,
    settings: &ConversionSettings,
    registry: &CapabilityRegistry,
) -> Result<ConversionPlan> {
    let eval = evaluate_conversion_settings(media, settings, registry);
    let preview_output_path = resolve_output_path(media, settings, &eval.extension)
        .unwrap_or_else(|_| Path::new("output").with_extension(&eval.extension));

    let ffmpeg_program = registry
        .ffmpeg
        .path
        .clone()
        .unwrap_or_else(|| "ffmpeg".to_string());

    let args = if eval.decision == DecisionKind::Fail {
        Vec::new()
    } else {
        assemble_args_strings(media, settings, &eval, &preview_output_path, false)
    };

    let command_preview = if args.is_empty() {
        String::new()
    } else {
        format_command_preview(&ffmpeg_program, &args)
    };

    Ok(ConversionPlan {
        decision: eval.decision,
        quality_impact: eval.quality_impact,
        summary: eval.summary,
        quality_explanation: eval.quality_explanation,
        warnings: eval.warnings,
        errors: eval.errors,
        resolved_output_path: preview_output_path.display().to_string(),
        resolved_muxer: eval.muxer,
        resolved_video_encoder: if eval.audio_only {
            None
        } else if normalize_codec_family(&settings.video_codec) == "copy" {
            Some("copy".to_string())
        } else {
            eval.selected_video_encoder
                .as_ref()
                .map(|e| e.encoder_name.clone())
        },
        resolved_audio_encoder: eval.resolved_audio_encoder,
        using_hardware: eval
            .selected_video_encoder
            .as_ref()
            .is_some_and(|e| e.is_hardware),
        ffmpeg_program,
        ffmpeg_args: args,
        command_preview,
    })
}

/// Builds typed `OsString` execution arguments targeting `staged_temp_output` with `-progress pipe:1`.
///
/// # Errors
/// Propagates any validation or planning errors from [`build_conversion_plan`].
pub fn build_execution_args(
    media: &MediaFile,
    settings: &ConversionSettings,
    registry: &CapabilityRegistry,
    staged_temp_output: &Path,
) -> Result<(ConversionPlan, Vec<OsString>)> {
    let plan = build_conversion_plan(media, settings, registry)?;
    let eval = evaluate_conversion_settings(media, settings, registry);

    let raw_args = assemble_args_strings(media, settings, &eval, staged_temp_output, true);
    let os_args = raw_args.into_iter().map(OsString::from).collect();
    Ok((plan, os_args))
}

fn assemble_args_strings(
    media: &MediaFile,
    settings: &ConversionSettings,
    eval: &crate::conversion::validation::ValidatedEvaluation,
    output_path: &Path,
    include_progress: bool,
) -> Vec<String> {
    let mut args = vec![
        "-hide_banner".to_string(),
        "-y".to_string(),
        "-protocol_whitelist".to_string(),
        "file,pipe".to_string(),
    ];

    if include_progress {
        args.extend([
            "-progress".to_string(),
            "pipe:1".to_string(),
            "-stats_period".to_string(),
            "0.25".to_string(),
        ]);
    }

    args.push("-i".to_string());
    args.push(media.path.clone());

    if eval.audio_only {
        args.push("-vn".to_string());
        args.push("-sn".to_string());
        for a_idx in &eval.selected_audio_indices {
            args.push("-map".to_string());
            args.push(format!("0:{a_idx}"));
        }
        append_audio_args(&mut args, settings, eval);
    } else {
        if let Some(v_idx) = eval.selected_video_stream_index {
            args.push("-map".to_string());
            args.push(format!("0:{v_idx}"));
        }

        if settings.audio_mode != AudioMode::Strip && !eval.selected_audio_indices.is_empty() {
            for a_idx in &eval.selected_audio_indices {
                args.push("-map".to_string());
                args.push(format!("0:{a_idx}"));
            }
        }

        // Subtitles
        if settings.subtitle_mode != SubtitleMode::Strip
            && !media.subtitle_streams.is_empty()
            && !matches!(eval.muxer.as_str(), "avi" | "webm")
        {
            for sub in &media.subtitle_streams {
                let is_bitmap = matches!(
                    sub.codec_name.as_str(),
                    "hdmv_pgs_subtitle" | "dvd_subtitle" | "dvb_subtitle"
                );
                if matches!(eval.muxer.as_str(), "mp4" | "mov") {
                    if is_bitmap {
                        continue;
                    }
                    if settings.subtitle_mode == SubtitleMode::CopyCompatible
                        && !matches!(sub.codec_name.as_str(), "mov_text" | "tx3g")
                    {
                        continue;
                    }
                }
                args.push("-map".to_string());
                args.push(format!("0:{}", sub.index));
            }
        }

        append_video_args(&mut args, settings, eval);
        append_audio_args(&mut args, settings, eval);
        append_subtitle_args(&mut args, settings, eval);
    }

    if settings.copy_metadata {
        args.extend(["-map_metadata".to_string(), "0".to_string()]);
    } else {
        args.extend(["-map_metadata".to_string(), "-1".to_string()]);
    }

    if eval.supports_faststart && settings.faststart {
        args.extend(["-movflags".to_string(), "+faststart".to_string()]);
    }

    args.extend(["-f".to_string(), eval.muxer.clone()]);
    args.push(output_path.display().to_string());
    args
}

fn append_video_args(
    args: &mut Vec<String>,
    settings: &ConversionSettings,
    eval: &crate::conversion::validation::ValidatedEvaluation,
) {
    let codec_norm = normalize_codec_family(&settings.video_codec);
    if codec_norm == "copy" {
        args.extend(["-c:v".to_string(), "copy".to_string()]);
        return;
    }

    let enc = eval
        .selected_video_encoder
        .as_ref()
        .map_or("libx264", |e| e.encoder_name.as_str());

    args.extend(["-c:v".to_string(), enc.to_string()]);

    if let Some(scale_expr) = resolution_filter_expr(settings) {
        args.extend(["-vf".to_string(), scale_expr]);
    }

    if let Some(fps) = settings
        .target_fps
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        args.extend(["-r".to_string(), fps.to_string()]);
    }

    if let Some(pf) = settings
        .pixel_format
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        args.extend(["-pix_fmt".to_string(), pf.to_string()]);
    } else if enc == "prores_ks" {
        args.extend(["-pix_fmt".to_string(), "yuv422p10le".to_string()]);
    }

    append_rate_control_args(args, enc, settings);
    append_video_preset_and_codec_defaults(args, enc, settings);
    append_expert_video_args(args, enc, settings);
}

fn append_video_preset_and_codec_defaults(
    args: &mut Vec<String>,
    enc: &str,
    settings: &ConversionSettings,
) {
    if let Some(preset) = settings
        .encoder_preset
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if enc == "libsvtav1" {
            args.extend(["-preset".to_string(), map_preset_to_svtav1(preset)]);
        } else if is_nvenc(enc) {
            args.extend(["-preset".to_string(), map_preset_to_nvenc(preset)]);
        } else if matches!(
            enc,
            "libx264" | "libx265" | "h264_qsv" | "hevc_qsv" | "av1_qsv"
        ) {
            args.extend(["-preset".to_string(), preset.to_string()]);
        }
    }

    if enc == "prores_ks" && settings.profile.is_none() {
        args.extend(["-profile:v".to_string(), "2".to_string()]);
    }
    if enc == "dnxhd" && settings.profile.is_none() {
        args.extend(["-profile:v".to_string(), "dnxhr_hq".to_string()]);
    }
    if enc == "ffv1" {
        args.extend([
            "-level".to_string(),
            "3".to_string(),
            "-coder".to_string(),
            "1".to_string(),
            "-context".to_string(),
            "1".to_string(),
        ]);
    }
}

fn append_expert_video_args(args: &mut Vec<String>, enc: &str, settings: &ConversionSettings) {
    if let Some(profile) = settings
        .profile
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        args.extend(["-profile:v".to_string(), profile.to_string()]);
    }
    if let Some(level) = settings
        .level
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        args.extend(["-level:v".to_string(), level.to_string()]);
    }
    if let Some(tune) = settings
        .tune
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        && matches!(enc, "libx264" | "libx265" | "h264_nvenc" | "hevc_nvenc")
    {
        args.extend(["-tune".to_string(), tune.to_string()]);
    }
    if let Some(gop) = settings.gop_size {
        args.extend(["-g".to_string(), gop.to_string()]);
    }
    if let Some(bf) = settings.b_frames {
        args.extend(["-bf".to_string(), bf.to_string()]);
    }
    if let Some(range) = settings
        .color_range
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        args.extend(["-color_range".to_string(), range.to_string()]);
    }
}

fn append_rate_control_args(args: &mut Vec<String>, enc: &str, settings: &ConversionSettings) {
    if matches!(enc, "prores_ks" | "dnxhd" | "ffv1") {
        return;
    }

    match settings.rate_control {
        RateControlMode::Lossless => {
            if enc == "libx264" {
                args.extend(["-crf".to_string(), "0".to_string()]);
            } else if enc == "libx265" {
                args.extend(["-x265-params".to_string(), "lossless=1".to_string()]);
            } else if enc == "libvpx-vp9" {
                args.extend(["-lossless".to_string(), "1".to_string()]);
            } else if is_nvenc(enc) {
                args.extend(["-tune".to_string(), "lossless".to_string()]);
            }
        }
        RateControlMode::Bitrate => {
            let kbps = settings.video_bitrate_kbps.unwrap_or(6000).max(100);
            args.extend(["-b:v".to_string(), format!("{kbps}k")]);
        }
        RateControlMode::Crf | RateControlMode::Cq => {
            let q = settings.quality_value;
            if is_nvenc(enc) {
                args.extend([
                    "-rc".to_string(),
                    "vbr".to_string(),
                    "-cq".to_string(),
                    q.to_string(),
                    "-b:v".to_string(),
                    "0".to_string(),
                ]);
            } else if is_qsv(enc) {
                args.extend(["-global_quality".to_string(), q.to_string()]);
            } else if is_amf(enc) {
                args.extend([
                    "-rc".to_string(),
                    "cqp".to_string(),
                    "-qp_i".to_string(),
                    q.to_string(),
                    "-qp_p".to_string(),
                    q.to_string(),
                ]);
            } else if enc == "libvpx-vp9" || enc == "libvpx" {
                args.extend([
                    "-crf".to_string(),
                    q.to_string(),
                    "-b:v".to_string(),
                    "0".to_string(),
                ]);
            } else if encoder_supports_crf(enc) {
                args.extend(["-crf".to_string(), q.to_string()]);
            } else {
                let fallback_kbps = settings.video_bitrate_kbps.unwrap_or(8000);
                args.extend(["-b:v".to_string(), format!("{fallback_kbps}k")]);
            }
        }
    }
}

fn append_audio_args(
    args: &mut Vec<String>,
    settings: &ConversionSettings,
    eval: &crate::conversion::validation::ValidatedEvaluation,
) {
    if settings.audio_mode == AudioMode::Strip || eval.selected_audio_indices.is_empty() {
        args.push("-an".to_string());
        return;
    }

    let enc = eval.resolved_audio_encoder.as_deref().unwrap_or("copy");
    args.extend(["-c:a".to_string(), enc.to_string()]);

    if enc != "copy" {
        if !matches!(enc, "flac" | "alac" | "pcm_s16le" | "pcm_s24le")
            && let Some(kbps) = settings.audio_bitrate_kbps
        {
            args.extend(["-b:a".to_string(), format!("{}k", kbps.max(32))]);
        }
        if let Some(ch) = settings.audio_channels {
            args.extend(["-ac".to_string(), ch.to_string()]);
        }
        if let Some(sr) = settings.audio_sample_rate {
            args.extend(["-ar".to_string(), sr.to_string()]);
        }
    }
}

fn append_subtitle_args(
    args: &mut Vec<String>,
    settings: &ConversionSettings,
    eval: &crate::conversion::validation::ValidatedEvaluation,
) {
    if settings.subtitle_mode == SubtitleMode::Strip {
        args.push("-sn".to_string());
        return;
    }
    if matches!(eval.muxer.as_str(), "mp4" | "mov")
        && settings.subtitle_mode == SubtitleMode::ConvertMovText
    {
        args.extend(["-c:s".to_string(), "mov_text".to_string()]);
    } else {
        args.extend(["-c:s".to_string(), "copy".to_string()]);
    }
}

fn resolution_filter_expr(settings: &ConversionSettings) -> Option<String> {
    match settings.resolution {
        ResolutionPreset::Original => None,
        ResolutionPreset::Uhd4k => Some("scale=3840:-2".to_string()),
        ResolutionPreset::Qhd1440p => Some("scale=2560:-2".to_string()),
        ResolutionPreset::Fhd1080p => Some("scale=1920:-2".to_string()),
        ResolutionPreset::Hd720p => Some("scale=1280:-2".to_string()),
        ResolutionPreset::Custom => match (settings.custom_width, settings.custom_height) {
            (Some(w), Some(h)) => Some(format!("scale={w}:{h}")),
            _ => None,
        },
    }
}

fn map_preset_to_svtav1(preset: &str) -> String {
    if preset.parse::<u8>().is_ok() {
        return preset.to_string();
    }
    match preset {
        "veryslow" => "3",
        "slower" => "4",
        "slow" => "5",
        "fast" => "8",
        "faster" => "10",
        "veryfast" => "11",
        "ultrafast" => "12",
        _ => "6",
    }
    .to_string()
}

fn map_preset_to_nvenc(preset: &str) -> String {
    if preset.starts_with('p') && preset.len() == 2 {
        return preset.to_string();
    }
    match preset {
        "ultrafast" => "p1",
        "veryfast" => "p2",
        "faster" => "p3",
        "fast" => "p4",
        "slow" => "p6",
        "slower" | "veryslow" => "p7",
        _ => "p5",
    }
    .to_string()
}

#[must_use]
pub fn format_command_preview(program: &str, args: &[String]) -> String {
    let mut tokens = Vec::with_capacity(args.len() + 1);
    tokens.push(quote_display_token(program));
    for arg in args {
        tokens.push(quote_display_token(arg));
    }
    tokens.join(" ")
}

fn quote_display_token(token: &str) -> String {
    if token.is_empty()
        || token
            .chars()
            .any(|c| c.is_whitespace() || c == '"' || c == '\'')
    {
        format!("\"{}\"", token.replace('"', "\\\""))
    } else {
        token.to_string()
    }
}
