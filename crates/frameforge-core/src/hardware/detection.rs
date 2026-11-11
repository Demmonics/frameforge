use std::path::Path;
use tokio::process::Command;
use tokio::task::JoinSet;

use crate::core::models::{CodecCapability, HardwareEncoderProbe, MediaKind};
use crate::platform::configure_background_command;

const CANDIDATE_HW_ENCODERS: &[(&str, &str, &str, bool)] = &[
    ("h264_nvenc", "NVIDIA NVENC", "h264", false),
    ("hevc_nvenc", "NVIDIA NVENC", "hevc", true),
    ("av1_nvenc", "NVIDIA NVENC", "av1", true),
    ("h264_qsv", "Intel Quick Sync", "h264", false),
    ("hevc_qsv", "Intel Quick Sync", "hevc", true),
    ("av1_qsv", "Intel Quick Sync", "av1", true),
    ("h264_amf", "AMD AMF", "h264", false),
    ("hevc_amf", "AMD AMF", "hevc", true),
    ("av1_amf", "AMD AMF", "av1", true),
    ("h264_mf", "Windows Media Foundation", "h264", false),
    ("hevc_mf", "Windows Media Foundation", "hevc", true),
];

pub async fn probe_hardware_encoders(
    ffmpeg_path: &Path,
    encoders: &[CodecCapability],
) -> Vec<HardwareEncoderProbe> {
    let mut set = JoinSet::new();

    for (idx, &(enc_name, backend, codec_family, supports_10bit)) in
        CANDIDATE_HW_ENCODERS.iter().enumerate()
    {
        let present = encoders
            .iter()
            .any(|e| e.media_type == MediaKind::Video && e.name == enc_name);
        if !present {
            continue;
        }
        let ff = ffmpeg_path.to_path_buf();
        set.spawn(async move {
            let probe =
                probe_single_hw_encoder(&ff, enc_name, backend, codec_family, supports_10bit).await;
            (idx, probe)
        });
    }

    let mut results = Vec::new();
    while let Some(joined) = set.join_next().await {
        if let Ok(item) = joined {
            results.push(item);
        }
    }
    results.sort_by_key(|(idx, _)| *idx);
    results.into_iter().map(|(_, p)| p).collect()
}

async fn probe_single_hw_encoder(
    ffmpeg_path: &Path,
    enc_name: &str,
    backend: &str,
    codec_family: &str,
    supports_10bit: bool,
) -> HardwareEncoderProbe {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=256x256:d=0.04",
        "-frames:v",
        "1",
        "-c:v",
        enc_name,
        "-f",
        "null",
        "-",
    ]);
    configure_background_command(&mut cmd);

    let (verified_working, diagnostic) =
        match tokio::time::timeout(std::time::Duration::from_secs(4), cmd.output()).await {
            Ok(Ok(out)) if out.status.success() => (
                true,
                format!("Verified working via {backend} device initialization"),
            ),
            Ok(Ok(out)) => {
                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let first_line = err
                    .lines()
                    .next()
                    .unwrap_or("Hardware device initialization failed")
                    .to_string();
                (false, first_line)
            }
            Ok(Err(e)) => (false, format!("Probe execution failed: {e}")),
            Err(_) => (false, "Hardware probe timed out".to_string()),
        };

    let supported_pixel_formats = match codec_family {
        "h264" => vec!["yuv420p".to_string(), "nv12".to_string()],
        _ if supports_10bit => vec![
            "yuv420p".to_string(),
            "nv12".to_string(),
            "p010le".to_string(),
        ],
        _ => vec!["yuv420p".to_string(), "nv12".to_string()],
    };

    HardwareEncoderProbe {
        encoder_name: enc_name.to_string(),
        backend: backend.to_string(),
        codec_family: codec_family.to_string(),
        available_in_ffmpeg: true,
        verified_working,
        supported_pixel_formats,
        supports_10bit,
        diagnostic,
    }
}
