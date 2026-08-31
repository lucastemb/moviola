use std::path::{Path, PathBuf};

use crate::Result;
use crate::env;
use crate::media::{self, MediaKind, video};

const DEFAULT_SEGMENT_COUNT: f64 = 10.0;

#[derive(Debug, Clone)]
pub struct ChopVideoConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub max_duration_seconds: f64,
    pub cut_frequency_seconds: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct KeepRange {
    start: f64,
    end: f64,
}

pub fn run() -> Result<()> {
    let input_path = env::required_path("INPUT").map_err(|_| usage())?;
    media::validate_path(&input_path, MediaKind::Video)?;

    let max_duration_seconds = env::required_f64("MAX_DURATION_SECONDS").map_err(|_| usage())?;
    let cut_frequency_seconds = env::optional_f64("CUT_FREQUENCY_SECONDS")?
        .unwrap_or_else(|| default_cut_frequency_seconds(max_duration_seconds));
    let output_path =
        env::optional_path("OUTPUT").unwrap_or_else(|| default_output_path(&input_path));

    let config = ChopVideoConfig {
        input_path,
        output_path,
        max_duration_seconds,
        cut_frequency_seconds,
    };

    eprintln!("Moviola: chopping video");
    eprintln!("  input: {}", config.input_path.display());
    eprintln!("  output: {}", config.output_path.display());
    eprintln!("  output duration: {} seconds", config.max_duration_seconds);
    eprintln!("  cut frequency: {} seconds", config.cut_frequency_seconds);

    chop_video(&config)?;

    eprintln!("Done: {}", config.output_path.display());
    Ok(())
}

fn default_output_path(input_path: &Path) -> PathBuf {
    media::default_output_path(input_path, "chopped", MediaKind::Video)
}

fn default_cut_frequency_seconds(max_duration_seconds: f64) -> f64 {
    max_duration_seconds / DEFAULT_SEGMENT_COUNT
}

pub fn run_with_config(config: &ChopVideoConfig) -> Result<()> {
    chop_video(config)
}

fn chop_video(config: &ChopVideoConfig) -> Result<()> {
    validate_config(config)?;
    media::ensure_ffmpeg_tools_available()?;
    media::ensure_output_parent_exists(&config.output_path)?;

    let source_duration = media::duration_seconds(&config.input_path)?;

    if source_duration + 0.001 < config.max_duration_seconds {
        return Err(format!(
            "Input video is too short. Need at least {:.3}s of source video to produce exactly {:.3}s, but input is only {source_duration:.3}s.",
            config.max_duration_seconds, config.max_duration_seconds
        )
        .into());
    }

    let keep_ranges = build_keep_ranges(
        source_duration,
        config.max_duration_seconds,
        config.cut_frequency_seconds,
    )?;

    let ranges = keep_ranges
        .iter()
        .map(|range| (range.start, range.end))
        .collect::<Vec<_>>();

    video::render_time_ranges(&config.input_path, &config.output_path, &ranges)
}

fn validate_config(config: &ChopVideoConfig) -> Result<()> {
    media::validate_path(&config.input_path, MediaKind::Video)?;

    if config.max_duration_seconds <= 0.0 {
        return Err("MAX_DURATION_SECONDS must be greater than 0".into());
    }

    if config.cut_frequency_seconds <= 0.0 {
        return Err("CUT_FREQUENCY_SECONDS must be greater than 0".into());
    }

    Ok(())
}

fn build_keep_ranges(
    source_duration_seconds: f64,
    max_duration_seconds: f64,
    cut_frequency_seconds: f64,
) -> Result<Vec<KeepRange>> {
    if source_duration_seconds + 0.001 < max_duration_seconds {
        return Err(format!(
            "Input video is too short. Need at least {max_duration_seconds:.3}s of source video, but input is only {source_duration_seconds:.3}s."
        )
        .into());
    }

    let segment_durations = build_segment_durations(max_duration_seconds, cut_frequency_seconds);
    if segment_durations.is_empty() {
        return Err("No keep ranges were generated".into());
    }

    let last_segment_duration = *segment_durations
        .last()
        .ok_or("No keep ranges were generated")?;
    let max_start = source_duration_seconds - last_segment_duration;
    let start_step = if segment_durations.len() == 1 {
        0.0
    } else {
        max_start / (segment_durations.len() - 1) as f64
    };

    Ok(segment_durations
        .iter()
        .enumerate()
        .map(|(index, duration)| {
            let start = start_step * index as f64;
            KeepRange {
                start,
                end: start + duration,
            }
        })
        .collect())
}

fn build_segment_durations(max_duration_seconds: f64, cut_frequency_seconds: f64) -> Vec<f64> {
    let mut remaining = max_duration_seconds;
    let mut durations = Vec::new();

    while remaining > 0.001 {
        let duration = remaining.min(cut_frequency_seconds);
        durations.push(duration);
        remaining -= duration;
    }

    durations
}

fn usage() -> Box<dyn std::error::Error> {
    "Missing required INPUT or MAX_DURATION_SECONDS. Usage: make chop-video INPUT=/path/to/video.mp4 MAX_DURATION_SECONDS=30 [OUTPUT=/path/out.mp4] [CUT_FREQUENCY_SECONDS=3]".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cut_frequency_splits_output_into_ten_keep_segments() {
        assert_eq!(default_cut_frequency_seconds(30.0), 3.0);
    }

    #[test]
    fn builds_keep_ranges_spaced_across_entire_source_video() {
        let ranges = build_keep_ranges(180.0, 30.0, 2.0).unwrap();

        assert_eq!(ranges.len(), 15);
        assert_eq!(
            ranges[0],
            KeepRange {
                start: 0.0,
                end: 2.0
            }
        );
        assert_eq!(
            ranges[14],
            KeepRange {
                start: 178.0,
                end: 180.0
            }
        );
    }

    #[test]
    fn last_keep_range_can_be_partial_to_hit_exact_output_duration() {
        let ranges = build_keep_ranges(100.0, 25.0, 6.0).unwrap();

        let output_duration: f64 = ranges.iter().map(|range| range.end - range.start).sum();
        assert!((output_duration - 25.0).abs() < 0.001);
        assert_eq!(
            ranges.last().unwrap().end - ranges.last().unwrap().start,
            1.0
        );
    }

    #[test]
    fn errors_when_source_is_shorter_than_output_duration() {
        let error = build_keep_ranges(5.0, 10.0, 1.0).unwrap_err().to_string();
        assert!(error.contains("too short"));
    }
}
