use crate::core::models::{
    AudioMode, CapabilityRegistry, ConversionSettings, ExportPreset, HardwareMode, MediaKind,
    OverwritePolicy, RateControlMode, ResolutionPreset, SubtitleMode,
};
use crate::media::formats::{candidate_video_encoders, find_container, preferred_audio_encoder};

type PresetSeed = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    ConversionSettings,
);

/// Builds the list of validated built-in export presets checked against `registry`.
#[must_use]
pub fn builtin_presets(registry: &CapabilityRegistry) -> Vec<ExportPreset> {
    let mut raw_defs = delivery_presets();
    raw_defs.extend(editing_and_archival_presets());

    raw_defs
        .into_iter()
        .map(|(id, name, category, description, settings)| {
            let (available, reason) = validate_preset_availability(&settings, registry);
            ExportPreset {
                id: id.to_string(),
                name: name.to_string(),
                category: category.to_string(),
                description: description.to_string(),
                is_builtin: true,
                settings,
                available,
                unavailability_reason: reason,
            }
        })
        .collect()
}

fn delivery_presets() -> Vec<PresetSeed> {
    vec![
        (
            "general-h264-mp4",
            "General H.264 MP4",
            "Delivery",
            "Balanced H.264 + AAC in MP4 with faststart for broad compatibility.",
            ConversionSettings {
                container: "mp4".into(),
                video_codec: "h264".into(),
                rate_control: RateControlMode::Crf,
                quality_value: 20,
                encoder_preset: Some("medium".into()),
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("aac".into()),
                audio_bitrate_kbps: Some(192),
                faststart: true,
                preset_id: Some("general-h264-mp4".into()),
                preset_name: Some("H264_MP4".into()),
                ..Default::default()
            },
        ),
        (
            "remux-mkv-lossless",
            "Stream Copy to MKV",
            "Lossless",
            "Lossless zero-reencode remux into Matroska preserving all audio & subtitle tracks.",
            ConversionSettings {
                container: "mkv".into(),
                video_codec: "copy".into(),
                audio_mode: AudioMode::Copy,
                subtitle_mode: SubtitleMode::CopyCompatible,
                faststart: false,
                preset_id: Some("remux-mkv-lossless".into()),
                preset_name: Some("MKV_Copy".into()),
                ..Default::default()
            },
        ),
        (
            "high-efficiency-hevc",
            "High-Efficiency HEVC",
            "Delivery",
            "Compact 10-bit capable H.265/HEVC MP4 for smaller high-detail archives.",
            ConversionSettings {
                container: "mp4".into(),
                video_codec: "hevc".into(),
                rate_control: RateControlMode::Crf,
                quality_value: 22,
                encoder_preset: Some("medium".into()),
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("aac".into()),
                audio_bitrate_kbps: Some(192),
                preset_id: Some("high-efficiency-hevc".into()),
                preset_name: Some("HEVC_MP4".into()),
                ..Default::default()
            },
        ),
        (
            "av1-compression",
            "AV1 Modern Compression",
            "Compression",
            "High-efficiency AV1 in MKV paired with Opus audio.",
            ConversionSettings {
                container: "mkv".into(),
                video_codec: "av1".into(),
                rate_control: RateControlMode::Crf,
                quality_value: 28,
                encoder_preset: Some("medium".into()),
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("opus".into()),
                audio_bitrate_kbps: Some(160),
                faststart: false,
                preset_id: Some("av1-compression".into()),
                preset_name: Some("AV1_MKV".into()),
                ..Default::default()
            },
        ),
        (
            "youtube-1080p-master",
            "Web / YouTube 1080p",
            "Delivery",
            "High-quality 1080p H.264 MP4 with 320 kbps AAC and web faststart.",
            ConversionSettings {
                container: "mp4".into(),
                video_codec: "h264".into(),
                rate_control: RateControlMode::Crf,
                quality_value: 17,
                encoder_preset: Some("slow".into()),
                resolution: ResolutionPreset::Fhd1080p,
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("aac".into()),
                audio_bitrate_kbps: Some(320),
                faststart: true,
                preset_id: Some("youtube-1080p-master".into()),
                preset_name: Some("Web_1080p".into()),
                ..Default::default()
            },
        ),
    ]
}

fn editing_and_archival_presets() -> Vec<PresetSeed> {
    vec![
        (
            "prores-422-edit",
            "ProRes 422 Editing Intermediate",
            "Editing",
            "Intra-frame Apple ProRes 422 + uncompressed PCM in MOV for smooth NLE scrubbing.",
            ConversionSettings {
                container: "mov".into(),
                video_codec: "prores".into(),
                video_encoder: Some("prores_ks".into()),
                hardware_mode: HardwareMode::Cpu,
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("pcm_s16le".into()),
                audio_bitrate_kbps: None,
                faststart: false,
                preset_id: Some("prores-422-edit".into()),
                preset_name: Some("ProRes422".into()),
                ..Default::default()
            },
        ),
        (
            "dnxhr-hq-edit",
            "DNxHR HQ Editing Intermediate",
            "Editing",
            "Avid DNxHR HQ + PCM audio in MOV for DaVinci Resolve / Premiere workflows.",
            ConversionSettings {
                container: "mov".into(),
                video_codec: "dnxhd".into(),
                video_encoder: Some("dnxhd".into()),
                hardware_mode: HardwareMode::Cpu,
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("pcm_s16le".into()),
                audio_bitrate_kbps: None,
                faststart: false,
                preset_id: Some("dnxhr-hq-edit".into()),
                preset_name: Some("DNxHR_HQ".into()),
                ..Default::default()
            },
        ),
        (
            "lossless-ffv1-archive",
            "Lossless FFV1 Archival",
            "Archival",
            "Mathematically lossless FFV1 video + FLAC audio in Matroska.",
            ConversionSettings {
                container: "mkv".into(),
                video_codec: "ffv1".into(),
                video_encoder: Some("ffv1".into()),
                rate_control: RateControlMode::Lossless,
                hardware_mode: HardwareMode::Cpu,
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("flac".into()),
                audio_bitrate_kbps: None,
                faststart: false,
                preset_id: Some("lossless-ffv1-archive".into()),
                preset_name: Some("FFV1_Archive".into()),
                ..Default::default()
            },
        ),
        (
            "editing-proxy-720p",
            "720p H.264 Editing Proxy",
            "Editing",
            "Fast lightweight 720p proxy for sluggish 4K/8K timelines.",
            ConversionSettings {
                container: "mov".into(),
                video_codec: "h264".into(),
                rate_control: RateControlMode::Crf,
                quality_value: 21,
                encoder_preset: Some("veryfast".into()),
                resolution: ResolutionPreset::Hd720p,
                gop_size: Some(15),
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("pcm_s16le".into()),
                audio_bitrate_kbps: None,
                overwrite_policy: OverwritePolicy::RenameIfExists,
                preset_id: Some("editing-proxy-720p".into()),
                preset_name: Some("Proxy_720p".into()),
                ..Default::default()
            },
        ),
        (
            "extract-audio-flac",
            "Extract Lossless FLAC Audio",
            "Audio",
            "Extracts audio stream directly to lossless FLAC.",
            ConversionSettings {
                container: "flac".into(),
                video_codec: "copy".into(),
                audio_mode: AudioMode::Transcode,
                audio_codec: Some("flac".into()),
                audio_bitrate_kbps: None,
                faststart: false,
                preset_id: Some("extract-audio-flac".into()),
                preset_name: Some("Audio_FLAC".into()),
                ..Default::default()
            },
        ),
    ]
}

fn validate_preset_availability(
    settings: &ConversionSettings,
    registry: &CapabilityRegistry,
) -> (bool, Option<String>) {
    let Some(container) = find_container(&settings.container) else {
        return (
            false,
            Some(format!("Unknown container '{}'", settings.container)),
        );
    };
    if !registry.muxers.iter().any(|m| m.name == container.muxer) {
        return (
            false,
            Some(format!("Muxer '{}' not present in FFmpeg", container.muxer)),
        );
    }
    if !container.audio_only && settings.video_codec != "copy" {
        let has_video_enc = match &settings.video_encoder {
            Some(explicit) => registry
                .encoders
                .iter()
                .any(|e| e.media_type == MediaKind::Video && e.name == *explicit),
            None => candidate_video_encoders(&settings.video_codec)
                .iter()
                .any(|name| {
                    registry
                        .encoders
                        .iter()
                        .any(|e| e.media_type == MediaKind::Video && e.name == *name)
                }),
        };
        if !has_video_enc {
            return (
                false,
                Some(format!(
                    "No compatible encoder installed for '{}'",
                    settings.video_codec
                )),
            );
        }
    }
    if settings.audio_mode == AudioMode::Transcode
        && let Some(ac) = &settings.audio_codec
        && preferred_audio_encoder(ac, &registry.encoders).is_none()
    {
        return (
            false,
            Some(format!("Audio encoder for '{ac}' not installed")),
        );
    }
    (true, None)
}
