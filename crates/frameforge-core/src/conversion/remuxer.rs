use crate::core::models::{AudioMode, ConversionSettings, MediaFile};
use crate::media::formats::{
    is_audio_stream_copy_compatible_with_container, is_video_stream_copy_compatible_with_container,
};

#[must_use]
pub fn container_matches_source(media: &MediaFile, target_container_id: &str) -> bool {
    let names: Vec<&str> = media
        .container
        .format_name
        .split(',')
        .map(str::trim)
        .collect();
    match target_container_id {
        "mp4" => names.contains(&"mp4") && media.path.to_ascii_lowercase().ends_with(".mp4"),
        "mov" => names.contains(&"mov") && media.path.to_ascii_lowercase().ends_with(".mov"),
        "mkv" => names.contains(&"matroska") && media.path.to_ascii_lowercase().ends_with(".mkv"),
        "webm" => names.contains(&"webm") || media.path.to_ascii_lowercase().ends_with(".webm"),
        "avi" => names.contains(&"avi"),
        "ts" => names.contains(&"mpegts"),
        "m4a" => media.path.to_ascii_lowercase().ends_with(".m4a"),
        "flac" => names.contains(&"flac"),
        "wav" => names.contains(&"wav"),
        "mp3" => names.contains(&"mp3"),
        _ => false,
    }
}

#[must_use]
pub fn can_losslessly_remux(media: &MediaFile, settings: &ConversionSettings) -> bool {
    if settings.video_codec != "copy" {
        return false;
    }
    if let Some(primary_v) = &media.primary_video
        && !is_video_stream_copy_compatible_with_container(
            &primary_v.codec_name,
            &settings.container,
        )
    {
        return false;
    }
    if settings.audio_mode == AudioMode::Copy {
        for a in &media.audio_streams {
            if !is_audio_stream_copy_compatible_with_container(&a.codec_name, &settings.container) {
                return false;
            }
        }
    }
    true
}
