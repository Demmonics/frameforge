use crate::core::models::{AudioStreamInfo, SubtitleStreamInfo, VideoStreamInfo};

/// Selects the primary video stream without ever assuming stream index 0 is video.
#[must_use]
pub fn select_primary_video_stream(streams: &[VideoStreamInfo]) -> Option<&VideoStreamInfo> {
    streams
        .iter()
        .find(|s| !s.is_attached_pic && s.is_default)
        .or_else(|| streams.iter().find(|s| !s.is_attached_pic))
        .or_else(|| streams.first())
}

#[must_use]
pub fn select_primary_audio_stream(streams: &[AudioStreamInfo]) -> Option<&AudioStreamInfo> {
    streams
        .iter()
        .find(|s| s.is_default)
        .or_else(|| streams.first())
}

#[must_use]
pub fn find_video_stream_by_index(
    streams: &[VideoStreamInfo],
    stream_index: Option<usize>,
) -> Option<&VideoStreamInfo> {
    match stream_index {
        Some(idx) => streams.iter().find(|s| s.index == idx),
        None => select_primary_video_stream(streams),
    }
}

#[must_use]
pub fn find_subtitle_streams(streams: &[SubtitleStreamInfo]) -> &[SubtitleStreamInfo] {
    streams
}
