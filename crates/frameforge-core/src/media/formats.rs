use crate::core::models::{CodecCapability, ContainerExportOption, FormatCapability, MediaKind};

#[derive(Debug, Clone)]
pub struct ContainerDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub extension: &'static str,
    pub muxer: &'static str,
    pub description: &'static str,
    pub audio_only: bool,
    pub compatible_video_codecs: &'static [&'static str],
    pub compatible_audio_codecs: &'static [&'static str],
    pub compatible_subtitle_codecs: &'static [&'static str],
    pub supports_faststart: bool,
}

pub const CONTAINER_DEFINITIONS: &[ContainerDefinition] = &[
    ContainerDefinition {
        id: "mp4",
        label: "MP4 (MPEG-4 Part 14)",
        extension: "mp4",
        muxer: "mp4",
        description: "Universal delivery container for web, editors, mobile, and broadcast.",
        audio_only: false,
        compatible_video_codecs: &["copy", "h264", "hevc", "av1", "mpeg4", "mpeg2video"],
        compatible_audio_codecs: &["copy", "aac", "mp3", "ac3", "eac3", "opus", "alac", "flac"],
        compatible_subtitle_codecs: &["mov_text", "tx3g"],
        supports_faststart: true,
    },
    ContainerDefinition {
        id: "mkv",
        label: "MKV (Matroska)",
        extension: "mkv",
        muxer: "matroska",
        description: "Flexible lossless/multi-track archival & mastering container.",
        audio_only: false,
        compatible_video_codecs: &[
            "copy",
            "h264",
            "hevc",
            "av1",
            "vp9",
            "vp8",
            "prores",
            "dnxhd",
            "ffv1",
            "mpeg4",
            "mpeg2video",
            "mjpeg",
        ],
        compatible_audio_codecs: &[
            "copy",
            "aac",
            "opus",
            "flac",
            "mp3",
            "ac3",
            "eac3",
            "vorbis",
            "pcm_s16le",
            "pcm_s24le",
            "alac",
        ],
        compatible_subtitle_codecs: &[
            "subrip",
            "srt",
            "ass",
            "ssa",
            "webvtt",
            "mov_text",
            "hdmv_pgs_subtitle",
            "dvd_subtitle",
        ],
        supports_faststart: false,
    },
    ContainerDefinition {
        id: "mov",
        label: "MOV (QuickTime)",
        extension: "mov",
        muxer: "mov",
        description: "Post-production intermediate container for ProRes, DNxHR, and H.264/HEVC.",
        audio_only: false,
        compatible_video_codecs: &["copy", "h264", "hevc", "prores", "dnxhd", "mjpeg"],
        compatible_audio_codecs: &["copy", "aac", "alac", "pcm_s16le", "pcm_s24le", "ac3"],
        compatible_subtitle_codecs: &["mov_text", "tx3g"],
        supports_faststart: true,
    },
    ContainerDefinition {
        id: "webm",
        label: "WebM",
        extension: "webm",
        muxer: "webm",
        description: "Open web container restricted to VP8/VP9/AV1 video and Opus/Vorbis audio.",
        audio_only: false,
        compatible_video_codecs: &["copy", "vp9", "vp8", "av1"],
        compatible_audio_codecs: &["copy", "opus", "vorbis"],
        compatible_subtitle_codecs: &["webvtt"],
        supports_faststart: false,
    },
    ContainerDefinition {
        id: "avi",
        label: "AVI",
        extension: "avi",
        muxer: "avi",
        description: "Legacy Windows container for MPEG-4, MJPEG, and uncompressed PCM.",
        audio_only: false,
        compatible_video_codecs: &["copy", "mpeg4", "mjpeg", "h264"],
        compatible_audio_codecs: &["copy", "mp3", "pcm_s16le", "ac3"],
        compatible_subtitle_codecs: &[],
        supports_faststart: false,
    },
    ContainerDefinition {
        id: "ts",
        label: "MPEG-TS (.ts)",
        extension: "ts",
        muxer: "mpegts",
        description: "Broadcast transport stream resilient to interruption.",
        audio_only: false,
        compatible_video_codecs: &["copy", "h264", "hevc", "mpeg2video"],
        compatible_audio_codecs: &["copy", "aac", "mp3", "ac3", "eac3"],
        compatible_subtitle_codecs: &["dvb_subtitle"],
        supports_faststart: false,
    },
    ContainerDefinition {
        id: "m4a",
        label: "M4A (Audio Only)",
        extension: "m4a",
        muxer: "ipod",
        description: "Audio-only MPEG-4 extraction for AAC and ALAC.",
        audio_only: true,
        compatible_video_codecs: &[],
        compatible_audio_codecs: &["copy", "aac", "alac"],
        compatible_subtitle_codecs: &[],
        supports_faststart: true,
    },
    ContainerDefinition {
        id: "flac",
        label: "FLAC (Lossless Audio)",
        extension: "flac",
        muxer: "flac",
        description: "Audio-only lossless FLAC extraction.",
        audio_only: true,
        compatible_video_codecs: &[],
        compatible_audio_codecs: &["copy", "flac"],
        compatible_subtitle_codecs: &[],
        supports_faststart: false,
    },
    ContainerDefinition {
        id: "wav",
        label: "WAV (Uncompressed PCM)",
        extension: "wav",
        muxer: "wav",
        description: "Audio-only uncompressed PCM waveform extraction.",
        audio_only: true,
        compatible_video_codecs: &[],
        compatible_audio_codecs: &["pcm_s16le", "pcm_s24le"],
        compatible_subtitle_codecs: &[],
        supports_faststart: false,
    },
    ContainerDefinition {
        id: "mp3",
        label: "MP3 (Audio Only)",
        extension: "mp3",
        muxer: "mp3",
        description: "Audio-only MP3 extraction.",
        audio_only: true,
        compatible_video_codecs: &[],
        compatible_audio_codecs: &["copy", "mp3"],
        compatible_subtitle_codecs: &[],
        supports_faststart: false,
    },
];

#[must_use]
pub fn find_container(id: &str) -> Option<&'static ContainerDefinition> {
    let norm = id.trim().to_ascii_lowercase();
    CONTAINER_DEFINITIONS
        .iter()
        .find(|c| c.id == norm || c.extension == norm || c.muxer == norm)
}

#[must_use]
pub fn normalize_codec_family(codec: &str) -> String {
    match codec.trim().to_ascii_lowercase().as_str() {
        "h264" | "avc" | "avc1" | "libx264" | "libx264rgb" | "h264_nvenc" | "h264_qsv"
        | "h264_amf" | "h264_mf" => "h264".to_string(),
        "hevc" | "h265" | "hvc1" | "hev1" | "libx265" | "hevc_nvenc" | "hevc_qsv" | "hevc_amf"
        | "hevc_mf" => "hevc".to_string(),
        "av1" | "libsvtav1" | "libaom-av1" | "librav1e" | "av1_nvenc" | "av1_qsv" | "av1_amf" => {
            "av1".to_string()
        }
        "vp9" | "libvpx-vp9" | "vp9_qsv" => "vp9".to_string(),
        "vp8" | "libvpx" => "vp8".to_string(),
        "prores" | "prores_ks" | "prores_aw" => "prores".to_string(),
        "dnxhd" | "dnxhr" => "dnxhd".to_string(),
        "ffv1" => "ffv1".to_string(),
        "mpeg4" | "xvid" | "libxvid" => "mpeg4".to_string(),
        "mpeg2video" | "mpeg2" => "mpeg2video".to_string(),
        "mjpeg" => "mjpeg".to_string(),
        "mp3" | "libmp3lame" => "mp3".to_string(),
        "opus" | "libopus" => "opus".to_string(),
        "vorbis" | "libvorbis" => "vorbis".to_string(),
        "aac" | "libfdk_aac" => "aac".to_string(),
        "flac" => "flac".to_string(),
        "alac" => "alac".to_string(),
        "ac3" => "ac3".to_string(),
        "eac3" => "eac3".to_string(),
        "pcm_s16le" => "pcm_s16le".to_string(),
        "pcm_s24le" => "pcm_s24le".to_string(),
        other => other.to_string(),
    }
}

#[must_use]
pub fn candidate_video_encoders(codec_family: &str) -> &'static [&'static str] {
    match codec_family {
        "h264" => &["libx264", "h264_nvenc", "h264_qsv", "h264_amf", "h264_mf"],
        "hevc" => &["libx265", "hevc_nvenc", "hevc_qsv", "hevc_amf", "hevc_mf"],
        "av1" => &[
            "libsvtav1",
            "libaom-av1",
            "librav1e",
            "av1_nvenc",
            "av1_qsv",
            "av1_amf",
        ],
        "vp9" => &["libvpx-vp9", "vp9_qsv"],
        "vp8" => &["libvpx"],
        "prores" => &["prores_ks", "prores_aw", "prores"],
        "dnxhd" => &["dnxhd"],
        "ffv1" => &["ffv1"],
        "mpeg4" => &["mpeg4", "libxvid"],
        "mpeg2video" => &["mpeg2video"],
        "mjpeg" => &["mjpeg"],
        _ => &[],
    }
}

#[must_use]
pub fn preferred_audio_encoder(codec_family: &str, encoders: &[CodecCapability]) -> Option<String> {
    let candidates: &[&str] = match codec_family {
        "aac" => &["aac", "libfdk_aac"],
        "opus" => &["libopus", "opus"],
        "mp3" => &["libmp3lame", "libshine"],
        "vorbis" => &["libvorbis", "vorbis"],
        "flac" => &["flac"],
        "alac" => &["alac"],
        "ac3" => &["ac3"],
        "eac3" => &["eac3"],
        "pcm_s16le" => &["pcm_s16le"],
        "pcm_s24le" => &["pcm_s24le"],
        other => {
            return encoders
                .iter()
                .find(|e| e.media_type == MediaKind::Audio && e.name == other)
                .map(|e| e.name.clone());
        }
    };

    for candidate in candidates {
        if encoders
            .iter()
            .any(|e| e.media_type == MediaKind::Audio && e.name == *candidate)
        {
            return Some((*candidate).to_string());
        }
    }
    None
}

#[must_use]
pub fn build_supported_containers(
    muxers: &[FormatCapability],
    encoders: &[CodecCapability],
) -> Vec<ContainerExportOption> {
    CONTAINER_DEFINITIONS
        .iter()
        .map(|def| {
            let has_muxer = muxers.iter().any(|m| m.name == def.muxer);
            let available_video: Vec<String> = def
                .compatible_video_codecs
                .iter()
                .filter(|codec| {
                    **codec == "copy"
                        || candidate_video_encoders(codec).iter().any(|enc_name| {
                            encoders
                                .iter()
                                .any(|e| e.media_type == MediaKind::Video && e.name == *enc_name)
                        })
                })
                .map(|s| (*s).to_string())
                .collect();

            let available_audio: Vec<String> = def
                .compatible_audio_codecs
                .iter()
                .filter(|codec| {
                    **codec == "copy" || preferred_audio_encoder(codec, encoders).is_some()
                })
                .map(|s| (*s).to_string())
                .collect();

            let available = has_muxer
                && if def.audio_only {
                    available_audio.iter().any(|a| a != "copy") || !available_audio.is_empty()
                } else {
                    available_video.iter().any(|v| v != "copy")
                };

            ContainerExportOption {
                id: def.id.to_string(),
                label: def.label.to_string(),
                extension: def.extension.to_string(),
                muxer: def.muxer.to_string(),
                description: def.description.to_string(),
                available,
                video_codecs: available_video,
                audio_codecs: available_audio,
                audio_only: def.audio_only,
            }
        })
        .collect()
}

#[must_use]
pub fn is_audio_stream_copy_compatible_with_container(
    source_audio_codec: &str,
    container_id: &str,
) -> bool {
    let Some(def) = find_container(container_id) else {
        return false;
    };
    let norm = normalize_codec_family(source_audio_codec);
    def.compatible_audio_codecs
        .iter()
        .any(|c| *c != "copy" && *c == norm)
}

#[must_use]
pub fn is_video_stream_copy_compatible_with_container(
    source_video_codec: &str,
    container_id: &str,
) -> bool {
    let Some(def) = find_container(container_id) else {
        return false;
    };
    let norm = normalize_codec_family(source_video_codec);
    def.compatible_video_codecs
        .iter()
        .any(|c| *c != "copy" && *c == norm)
}
