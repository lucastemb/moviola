use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::Result;
use crate::media::MediaKind;

pub fn transcode_for_delivery(
    input_path: &Path,
    output_path: &Path,
    media_kind: MediaKind,
) -> Result<()> {
    let mut command = Command::new("ffmpeg");
    command
        .args(["-y", "-hide_banner", "-nostats", "-i"])
        .arg(input_path);

    match media_kind {
        MediaKind::Video => {
            command.args(["-map", "0:v:0", "-map", "0:a:0?", "-shortest"]);
        }
        MediaKind::Audio => {
            command.args(["-map", "0:a:0"]);
        }
    }

    add_delivery_encoding_args(&mut command, output_path, media_kind);

    let status = command
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err("ffmpeg failed while exporting the media".into());
    }

    Ok(())
}

pub fn add_delivery_encoding_args(
    command: &mut Command,
    output_path: &Path,
    media_kind: MediaKind,
) {
    match media_kind {
        MediaKind::Video => add_video_delivery_encoding_args(command, output_path),
        MediaKind::Audio => add_audio_delivery_encoding_args(command, output_path),
    }
}

fn add_video_delivery_encoding_args(command: &mut Command, output_path: &Path) {
    if is_quicktime_compatible_container(output_path) {
        let encoder =
            optional_env("VIDEO_ENCODER").unwrap_or_else(|| default_video_encoder().into());

        match encoder.as_str() {
            "h264_videotoolbox" => {
                let video_bitrate =
                    optional_env("VIDEO_BITRATE").unwrap_or_else(|| "30000k".into());
                command.args([
                    "-c:v",
                    "h264_videotoolbox",
                    "-b:v",
                    &video_bitrate,
                    "-allow_sw",
                    "1",
                    "-pix_fmt",
                    "yuv420p",
                ]);
            }
            _ => {
                let video_crf = optional_env("VIDEO_CRF").unwrap_or_else(|| "16".into());
                let video_preset = optional_env("VIDEO_PRESET").unwrap_or_else(|| "slow".into());
                command.args([
                    "-c:v",
                    "libx264",
                    "-preset",
                    &video_preset,
                    "-crf",
                    &video_crf,
                    "-pix_fmt",
                    "yuv420p",
                ]);
            }
        }

        command.args([
            "-c:a",
            "aac",
            "-b:a",
            "320k",
            "-ar",
            "48000",
            "-movflags",
            "+faststart",
            "-video_track_timescale",
            "600",
        ]);
    }
}

fn add_audio_delivery_encoding_args(command: &mut Command, output_path: &Path) {
    match output_path.extension().and_then(OsStr::to_str) {
        Some(extension) if extension.eq_ignore_ascii_case("mp3") => {
            command.args(["-c:a", "libmp3lame", "-b:a", "320k"]);
        }
        Some(extension) if extension.eq_ignore_ascii_case("wav") => {
            command.args(["-c:a", "pcm_s16le"]);
        }
        Some(extension) if extension.eq_ignore_ascii_case("flac") => {
            command.args(["-c:a", "flac"]);
        }
        _ => {
            command.args(["-c:a", "aac", "-b:a", "320k", "-ar", "48000"]);
        }
    }
}

fn default_video_encoder() -> &'static str {
    "libx264"
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn is_quicktime_compatible_container(output_path: &Path) -> bool {
    output_path
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            ["mp4", "mov", "m4v"]
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
}
