use std::path::Path;
use std::process::{Command, Stdio};

use crate::Result;
use crate::media::{MediaKind, encoding};

pub fn render_time_ranges(
    input_path: &Path,
    output_path: &Path,
    ranges: &[(f64, f64)],
) -> Result<()> {
    let mut filter = String::new();

    for (index, (start, end)) in ranges.iter().enumerate() {
        filter.push_str(&format!(
            "[0:a]atrim=start={start:.6}:end={end:.6},asetpts=PTS-STARTPTS[a{index}];"
        ));
    }

    for index in 0..ranges.len() {
        filter.push_str(&format!("[a{index}]"));
    }
    filter.push_str(&format!("concat=n={}:v=0:a=1[outa]", ranges.len()));

    let mut command = Command::new("ffmpeg");
    command
        .args(["-y", "-hide_banner", "-nostats", "-i"])
        .arg(input_path)
        .args(["-filter_complex", &filter, "-map", "[outa]"]);

    encoding::add_delivery_encoding_args(&mut command, output_path, MediaKind::Audio);

    let status = command
        .arg(output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err("ffmpeg failed while rendering the audio clip".into());
    }

    Ok(())
}
