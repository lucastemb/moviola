use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Result;
use crate::env;
use crate::media::video;

const DEFAULT_SILENCE_THRESHOLD_DB: f32 = -20.0;
const DEFAULT_MIN_SILENCE_MS: f32 = 250.0;

#[derive(Debug, Clone)]
pub struct SilenceTrimConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub threshold_db: f32,
    pub min_silence_duration_seconds: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct SilenceRange {
    start: f64,
    end: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct KeepRange {
    start: f64,
    end: f64,
}

pub fn run() -> Result<()> {
    let input_path = env::required_path("INPUT").map_err(|_| usage())?;
    video::validate_video_path(&input_path)?;

    let output_path =
        env::optional_path("OUTPUT").unwrap_or_else(|| default_output_path(&input_path));
    let threshold_db =
        env::optional_f32("SILENCE_THRESHOLD")?.unwrap_or(DEFAULT_SILENCE_THRESHOLD_DB);
    let min_silence_ms = env::optional_f32("MIN_SILENCE_MS")?
        .or(env::optional_f32("MIN_SILENCE")?)
        .unwrap_or(DEFAULT_MIN_SILENCE_MS);

    let config = SilenceTrimConfig {
        input_path,
        output_path,
        threshold_db,
        min_silence_duration_seconds: min_silence_ms / 1000.0,
    };

    eprintln!("Moviola: trimming silence");
    eprintln!("  input: {}", config.input_path.display());
    eprintln!("  output: {}", config.output_path.display());
    eprintln!("  silence threshold: {} dB", config.threshold_db);
    eprintln!("  minimum silence: {} ms", min_silence_ms);

    trim_silence(&config)?;

    eprintln!("Done: {}", config.output_path.display());
    Ok(())
}

fn default_output_path(input_path: &Path) -> PathBuf {
    video::default_mp4_output_path(input_path, "silence_trimmed")
}

fn trim_silence(config: &SilenceTrimConfig) -> Result<()> {
    video::validate_video_path(&config.input_path)?;

    if config.threshold_db >= 0.0 {
        return Err("Silence threshold must be below 0 dB, for example -30".into());
    }

    if config.min_silence_duration_seconds <= 0.0 {
        return Err("Minimum silence duration must be greater than 0 seconds".into());
    }

    video::ensure_ffmpeg_tools_available()?;
    video::ensure_output_parent_exists(&config.output_path)?;

    let duration = video::duration_seconds(&config.input_path)?;
    let silences = detect_silences(config)?;
    let keep_ranges = build_keep_ranges(duration, &silences);

    if keep_ranges.is_empty() {
        return Err(
            "The whole clip is below the configured silence threshold; nothing to keep".into(),
        );
    }

    if keep_ranges.len() == 1
        && keep_ranges[0].start <= 0.001
        && keep_ranges[0].end >= duration - 0.001
    {
        video::transcode_for_delivery(&config.input_path, &config.output_path)?;
        return Ok(());
    }

    let ranges = keep_ranges
        .iter()
        .map(|range| (range.start, range.end))
        .collect::<Vec<_>>();
    video::render_time_ranges(&config.input_path, &config.output_path, &ranges)
}

fn detect_silences(config: &SilenceTrimConfig) -> Result<Vec<SilenceRange>> {
    let filter = format!(
        "silencedetect=noise={}dB:d={}",
        config.threshold_db, config.min_silence_duration_seconds
    );

    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-nostats", "-i"])
        .arg(&config.input_path)
        .args(["-af", &filter, "-f", "null", "-"])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg silence detection failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(parse_silencedetect_output(&String::from_utf8_lossy(
        &output.stderr,
    )))
}

fn parse_silencedetect_output(output: &str) -> Vec<SilenceRange> {
    let mut ranges = Vec::new();
    let mut current_start = None;

    for line in output.lines() {
        if let Some(value) = value_after_marker(line, "silence_start:") {
            current_start = value.parse::<f64>().ok();
            continue;
        }

        if let Some(value) = value_after_marker(line, "silence_end:")
            && let (Some(start), Ok(end)) = (current_start.take(), value.parse::<f64>())
            && end > start
        {
            ranges.push(SilenceRange { start, end });
        }
    }

    ranges
}

fn value_after_marker<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let (_, value) = line.split_once(marker)?;
    value.split_whitespace().next()
}

fn build_keep_ranges(duration: f64, silences: &[SilenceRange]) -> Vec<KeepRange> {
    let mut keep_ranges = Vec::new();
    let mut cursor = 0.0_f64;

    for silence in silences {
        let silence_start = silence.start.clamp(0.0, duration);
        let silence_end = silence.end.clamp(0.0, duration);

        if silence_start > cursor {
            keep_ranges.push(KeepRange {
                start: cursor,
                end: silence_start,
            });
        }

        cursor = cursor.max(silence_end);
    }

    if cursor < duration {
        keep_ranges.push(KeepRange {
            start: cursor,
            end: duration,
        });
    }

    keep_ranges
        .into_iter()
        .filter(|range| range.end - range.start > 0.01)
        .collect()
}

fn usage() -> Box<dyn std::error::Error> {
    "Missing required INPUT. Usage: make trim-silence INPUT=/path/to/video.mp4 [OUTPUT=/path/out.mp4] [SILENCE_THRESHOLD=-20] [MIN_SILENCE_MS=250]".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_silencedetect_ranges() {
        let output = "[silencedetect @ 0x] silence_start: 1.25\n[silencedetect @ 0x] silence_end: 2.5 | silence_duration: 1.25";
        assert_eq!(
            parse_silencedetect_output(output),
            vec![SilenceRange {
                start: 1.25,
                end: 2.5
            }]
        );
    }

    #[test]
    fn builds_keep_ranges_around_silence() {
        let ranges = build_keep_ranges(
            10.0,
            &[
                SilenceRange {
                    start: 2.0,
                    end: 3.0,
                },
                SilenceRange {
                    start: 7.0,
                    end: 8.0,
                },
            ],
        );

        assert_eq!(
            ranges,
            vec![
                KeepRange {
                    start: 0.0,
                    end: 2.0
                },
                KeepRange {
                    start: 3.0,
                    end: 7.0
                },
                KeepRange {
                    start: 8.0,
                    end: 10.0
                }
            ]
        );
    }
}
