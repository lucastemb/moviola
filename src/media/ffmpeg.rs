use std::path::Path;
use std::process::{Command, Stdio};

use crate::Result;

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
        return Err("Could not determine a positive media duration".into());
    }

    Ok(duration)
}
