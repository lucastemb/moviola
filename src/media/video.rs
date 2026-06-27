use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::Result;

const SUPPORTED_VIDEO_EXTENSIONS: &[&str] =
    &["mp4", "mov", "m4v", "mkv", "webm", "avi", "mpeg", "mpg"];

pub fn validate_video_path(path: &Path) -> Result<()> {
    let Some(extension) = path.extension().and_then(OsStr::to_str) else {
        return Err("Input file must have a video extension".into());
    };

    if !SUPPORTED_VIDEO_EXTENSIONS
        .iter()
        .any(|supported| extension.eq_ignore_ascii_case(supported))
    {
        return Err(format!(
            "Unsupported file extension '.{extension}'. Supported video extensions: {}",
            SUPPORTED_VIDEO_EXTENSIONS.join(", ")
        )
        .into());
    }

    if !path.exists() {
        return Err(format!("Input file does not exist: {}", path.display()).into());
    }

    if !path.is_file() {
        return Err(format!("Input path is not a file: {}", path.display()).into());
    }

    Ok(())
}

pub fn ensure_ffmpeg_tools_available() -> Result<()> {
    for binary in ["ffmpeg", "ffprobe"] {
        let status = Command::new(binary)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if !matches!(status, Ok(status) if status.success()) {
            return Err(
                format!("Required binary '{binary}' was not found. Install ffmpeg first.").into(),
            );
        }
    }

    Ok(())
}

pub fn duration_seconds(input_path: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(input_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let duration: f64 = String::from_utf8_lossy(&output.stdout).trim().parse()?;
    if duration <= 0.0 {
        return Err("Could not determine a positive video duration".into());
    }

    Ok(duration)
}

pub fn ensure_output_parent_exists(output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

pub fn transcode_for_delivery(input_path: &Path, output_path: &Path) -> Result<()> {
    let mut command = Command::new("ffmpeg");
    command
        .args(["-y", "-hide_banner", "-nostats", "-i"])
        .arg(input_path)
        .args(["-map", "0:v:0", "-map", "0:a:0?", "-shortest"]);

    add_delivery_encoding_args(&mut command, output_path);

    let status = command
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err("ffmpeg failed while exporting the clip".into());
    }

    Ok(())
}

pub fn render_time_ranges(
    input_path: &Path,
    output_path: &Path,
    ranges: &[(f64, f64)],
) -> Result<()> {
    let mut filter = String::new();

    for (index, (start, end)) in ranges.iter().enumerate() {
        filter.push_str(&format!(
            "[0:v]trim=start={start:.6}:end={end:.6},setpts=PTS-STARTPTS[v{index}];"
        ));
        filter.push_str(&format!(
            "[0:a]atrim=start={start:.6}:end={end:.6},asetpts=PTS-STARTPTS[a{index}];"
        ));
    }

    for index in 0..ranges.len() {
        filter.push_str(&format!("[v{index}][a{index}]"));
    }
    filter.push_str(&format!("concat=n={}:v=1:a=1[outv][outa]", ranges.len()));

    let mut command = Command::new("ffmpeg");
    command
        .args(["-y", "-hide_banner", "-nostats", "-i"])
        .arg(input_path)
        .args([
            "-filter_complex",
            &filter,
            "-map",
            "[outv]",
            "-map",
            "[outa]",
        ]);

    add_delivery_encoding_args(&mut command, output_path);

    let status = command
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err("ffmpeg failed while rendering the clip".into());
    }

    Ok(())
}

fn add_delivery_encoding_args(command: &mut Command, output_path: &Path) {
    if is_quicktime_compatible_container(output_path) {
        let encoder = env::var("VIDEO_ENCODER").unwrap_or_else(|_| default_video_encoder().into());

        match encoder.as_str() {
            "h264_videotoolbox" => {
                let video_bitrate = env::var("VIDEO_BITRATE").unwrap_or_else(|_| "6000k".into());
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
                command.args([
                    "-c:v",
                    "libx264",
                    "-preset",
                    "veryfast",
                    "-crf",
                    "23",
                    "-pix_fmt",
                    "yuv420p",
                    "-profile:v",
                    "high",
                    "-level",
                    "4.1",
                ]);
            }
        }

        command.args([
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-ar",
            "48000",
            "-movflags",
            "+faststart",
            "-video_track_timescale",
            "600",
        ]);
    }
}

fn default_video_encoder() -> &'static str {
    if cfg!(target_os = "macos") {
        "h264_videotoolbox"
    } else {
        "libx264"
    }
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

pub fn default_mp4_output_path(input_path: &Path, suffix: &str) -> PathBuf {
    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = input_path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("output");
    parent.join(format!("{stem}_{suffix}.mp4"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn rejects_non_video_extensions() {
        let path = Path::new("notes.txt");
        let error = validate_video_path(path).unwrap_err().to_string();
        assert!(error.contains("Unsupported"));
    }
}
