use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use uuid::Uuid;

use crate::core::models::MediaFile;
use crate::media::streams::select_primary_video_stream;
use crate::platform::configure_background_command;

pub async fn generate_thumbnail(
    ffmpeg_path: &Path,
    media: &MediaFile,
    thumbnails_dir: &Path,
) -> Option<(String, String)> {
    let video = select_primary_video_stream(&media.video_streams)?;
    if video.width == 0 || video.height == 0 {
        return None;
    }

    if tokio::fs::create_dir_all(thumbnails_dir).await.is_err() {
        return None;
    }

    let thumb_file = thumbnails_dir.join(format!("{}.jpg", Uuid::new_v4().simple()));
    let seek_secs = media
        .duration_secs
        .map_or(0.0, |d| if d > 2.0 { (d * 0.12).min(15.0) } else { 0.0 });

    let ok = run_thumb_extract(ffmpeg_path, Path::new(&media.path), &thumb_file, seek_secs).await;
    let produced = if ok && tokio::fs::try_exists(&thumb_file).await.unwrap_or(false) {
        true
    } else if seek_secs > 0.0 {
        run_thumb_extract(ffmpeg_path, Path::new(&media.path), &thumb_file, 0.0).await
            && tokio::fs::try_exists(&thumb_file).await.unwrap_or(false)
    } else {
        false
    };

    if !produced {
        let _ = tokio::fs::remove_file(&thumb_file).await;
        return None;
    }

    let bytes = tokio::fs::read(&thumb_file).await.ok()?;
    if bytes.is_empty() {
        let _ = tokio::fs::remove_file(&thumb_file).await;
        return None;
    }

    let data_url = format!("data:image/jpeg;base64,{}", STANDARD.encode(&bytes));
    Some((data_url, PathBuf::from(&thumb_file).display().to_string()))
}

async fn run_thumb_extract(
    ffmpeg_path: &Path,
    input_path: &Path,
    out_path: &Path,
    seek_secs: f64,
) -> bool {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-protocol_whitelist",
        "file,pipe",
    ]);
    if seek_secs > 0.05 {
        cmd.args(["-ss", &format!("{seek_secs:.2}")]);
    }
    cmd.arg("-i").arg(input_path.as_os_str());
    cmd.args([
        "-frames:v",
        "1",
        "-vf",
        "scale='min(480,iw)':-2",
        "-q:v",
        "4",
        "-f",
        "image2",
    ])
    .arg(out_path.as_os_str());
    configure_background_command(&mut cmd);

    match tokio::time::timeout(std::time::Duration::from_secs(6), cmd.output()).await {
        Ok(Ok(out)) => out.status.success(),
        _ => false,
    }
}
