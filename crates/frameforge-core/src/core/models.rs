use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::core::errors::SerializableError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInfo {
    pub format_name: String,
    pub format_long_name: String,
    pub probe_score: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HdrMetadata {
    pub mastering_display: Option<String>,
    pub max_cll: Option<u32>,
    pub max_fall: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoStreamInfo {
    pub index: usize,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: Option<String>,
    pub level: Option<i32>,
    pub width: u32,
    pub height: u32,
    pub coded_width: Option<u32>,
    pub coded_height: Option<u32>,
    pub sample_aspect_ratio: Option<String>,
    pub display_aspect_ratio: Option<String>,
    pub pix_fmt: Option<String>,
    pub bit_depth: Option<u8>,
    pub r_frame_rate: Option<String>,
    pub avg_frame_rate: Option<String>,
    pub fps_nominal: Option<f64>,
    pub fps_average: Option<f64>,
    pub is_vfr: bool,
    pub vfr_note: Option<String>,
    pub bit_rate: Option<u64>,
    pub nb_frames: Option<u64>,
    pub color_primaries: Option<String>,
    pub color_transfer: Option<String>,
    pub color_space: Option<String>,
    pub color_range: Option<String>,
    pub hdr_format: Option<String>,
    pub hdr_metadata: Option<HdrMetadata>,
    pub rotation_degrees: Option<i32>,
    pub field_order: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub encoder: Option<String>,
    pub is_default: bool,
    pub is_attached_pic: bool,
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamInfo {
    pub index: usize,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    pub sample_fmt: Option<String>,
    pub bit_rate: Option<u64>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub is_default: bool,
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleStreamInfo {
    pub index: usize,
    pub codec_name: String,
    pub codec_long_name: String,
    pub language: Option<String>,
    pub title: Option<String>,
    pub is_default: bool,
    pub is_forced: bool,
    pub tags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaFile {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub container: ContainerInfo,
    pub duration_secs: Option<f64>,
    pub bit_rate: Option<u64>,
    pub primary_video: Option<VideoStreamInfo>,
    pub primary_audio: Option<AudioStreamInfo>,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
    pub subtitle_streams: Vec<SubtitleStreamInfo>,
    pub chapters_count: usize,
    pub creation_time: Option<String>,
    pub encoder_tag: Option<String>,
    pub tags: BTreeMap<String, String>,
    pub thumbnail_data_url: Option<String>,
    pub thumbnail_cache_path: Option<String>,
    pub technical_report: String,
    pub raw_probe_json: String,
    pub probed_at_utc: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Video,
    Audio,
    Subtitle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FormatCapability {
    pub name: String,
    pub description: String,
}

// Mirrors four independent FFmpeg codec capability flags directly over IPC JSON.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodecCapability {
    pub name: String,
    pub codec_family: String,
    pub media_type: MediaKind,
    pub description: String,
    pub is_hardware: bool,
    pub experimental: bool,
    pub frame_mt: bool,
    pub slice_mt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FilterCapability {
    pub name: String,
    pub io_signature: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BinaryToolStatus {
    pub name: String,
    pub available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub configuration: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HardwareEncoderProbe {
    pub encoder_name: String,
    pub backend: String,
    pub codec_family: String,
    pub available_in_ffmpeg: bool,
    pub verified_working: bool,
    pub supported_pixel_formats: Vec<String>,
    pub supports_10bit: bool,
    pub diagnostic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContainerExportOption {
    pub id: String,
    pub label: String,
    pub extension: String,
    pub muxer: String,
    pub description: String,
    pub available: bool,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub audio_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityRegistry {
    pub ffmpeg: BinaryToolStatus,
    pub ffprobe: BinaryToolStatus,
    pub mpv: BinaryToolStatus,
    pub license_profile: String,
    pub demuxers: Vec<FormatCapability>,
    pub muxers: Vec<FormatCapability>,
    pub decoders: Vec<CodecCapability>,
    pub encoders: Vec<CodecCapability>,
    pub filters: Vec<FilterCapability>,
    pub hwaccels: Vec<String>,
    pub hardware_probes: Vec<HardwareEncoderProbe>,
    pub supported_containers: Vec<ContainerExportOption>,
    pub probed_at_utc: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConversionInterfaceLevel {
    Simple,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RateControlMode {
    Crf,
    Cq,
    Bitrate,
    Lossless,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AudioMode {
    Copy,
    Transcode,
    Strip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SubtitleMode {
    CopyCompatible,
    ConvertMovText,
    Strip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HardwareMode {
    Auto,
    Cpu,
    Gpu,
    QualityPriority,
    SpeedPriority,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OverwritePolicy {
    RenameIfExists,
    Overwrite,
    FailIfExists,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResolutionPreset {
    Original,
    Uhd4k,
    Qhd1440p,
    Fhd1080p,
    Hd720p,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConversionSettings {
    pub container: String,
    pub video_codec: String,
    pub video_encoder: Option<String>,
    pub rate_control: RateControlMode,
    pub quality_value: u8,
    pub video_bitrate_kbps: Option<u32>,
    pub encoder_preset: Option<String>,
    pub resolution: ResolutionPreset,
    pub custom_width: Option<u32>,
    pub custom_height: Option<u32>,
    pub target_fps: Option<String>,
    pub pixel_format: Option<String>,
    pub hardware_mode: HardwareMode,
    pub allow_cpu_fallback: bool,
    pub video_stream_index: Option<usize>,
    pub audio_mode: AudioMode,
    pub audio_codec: Option<String>,
    pub audio_bitrate_kbps: Option<u32>,
    pub audio_channels: Option<u32>,
    pub audio_sample_rate: Option<u32>,
    pub audio_stream_indices: Option<Vec<usize>>,
    pub subtitle_mode: SubtitleMode,
    pub copy_metadata: bool,
    pub faststart: bool,
    pub profile: Option<String>,
    pub level: Option<String>,
    pub tune: Option<String>,
    pub gop_size: Option<u32>,
    pub b_frames: Option<u8>,
    pub color_range: Option<String>,
    pub output_dir: Option<String>,
    pub filename_pattern: String,
    pub overwrite_policy: OverwritePolicy,
    pub preset_id: Option<String>,
    pub preset_name: Option<String>,
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self {
            container: "mp4".to_string(),
            video_codec: "h264".to_string(),
            video_encoder: None,
            rate_control: RateControlMode::Crf,
            quality_value: 20,
            video_bitrate_kbps: None,
            encoder_preset: Some("medium".to_string()),
            resolution: ResolutionPreset::Original,
            custom_width: None,
            custom_height: None,
            target_fps: None,
            pixel_format: None,
            hardware_mode: HardwareMode::Auto,
            allow_cpu_fallback: true,
            video_stream_index: None,
            audio_mode: AudioMode::Transcode,
            audio_codec: Some("aac".to_string()),
            audio_bitrate_kbps: Some(192),
            audio_channels: None,
            audio_sample_rate: None,
            audio_stream_indices: None,
            subtitle_mode: SubtitleMode::Strip,
            copy_metadata: true,
            faststart: true,
            profile: None,
            level: None,
            tune: None,
            gop_size: None,
            b_frames: None,
            color_range: None,
            output_dir: None,
            filename_pattern: "{name}_converted".to_string(),
            overwrite_policy: OverwritePolicy::RenameIfExists,
            preset_id: None,
            preset_name: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DecisionKind {
    Copy,
    Remux,
    Transcode,
    Fail,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum QualityImpact {
    LosslessStreamCopy,
    LosslessReencode,
    QualityAltered,
    AudioOnlyExtraction,
    Incompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversionPlan {
    pub decision: DecisionKind,
    pub quality_impact: QualityImpact,
    pub summary: String,
    pub quality_explanation: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub resolved_output_path: String,
    pub resolved_muxer: String,
    pub resolved_video_encoder: Option<String>,
    pub resolved_audio_encoder: Option<String>,
    pub using_hardware: bool,
    pub ffmpeg_program: String,
    pub ffmpeg_args: Vec<String>,
    pub command_preview: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JobState {
    Queued,
    Validating,
    Running,
    Paused,
    Cancelling,
    Cancelled,
    Failed,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub percent: f64,
    pub frame: Option<u64>,
    pub fps: Option<f64>,
    pub bitrate: Option<String>,
    pub total_size_bytes: Option<u64>,
    pub out_time_secs: f64,
    pub speed: Option<f64>,
    pub elapsed_ms: u64,
    pub eta_secs: Option<f64>,
}

impl Default for JobProgress {
    fn default() -> Self {
        Self {
            percent: 0.0,
            frame: None,
            fps: None,
            bitrate: None,
            total_size_bytes: None,
            out_time_secs: 0.0,
            speed: None,
            elapsed_ms: 0,
            eta_secs: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConversionJob {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub media_info: MediaFile,
    pub selected_preset: Option<String>,
    pub settings: ConversionSettings,
    pub hardware_mode: HardwareMode,
    pub decision: DecisionKind,
    pub state: JobState,
    pub progress: JobProgress,
    pub actual_encoder: Option<String>,
    pub logs: Vec<String>,
    pub error: Option<SerializableError>,
    pub output_size_bytes: Option<u64>,
    pub created_at_utc: String,
    pub started_at_utc: Option<String>,
    pub finished_at_utc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreset {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub is_builtin: bool,
    pub settings: ConversionSettings,
    pub available: bool,
    pub unavailability_reason: Option<String>,
}
