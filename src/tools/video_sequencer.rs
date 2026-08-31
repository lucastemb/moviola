use crate::Result;
use crate::media;
use std::path::PathBuf;

//This will run the logic to alternate between clips
pub fn run() -> Result<()> {
    return Ok(());
}

//Here is the math: if we are given x clips and a max duration, we want to alternate every
//MAX_DURATION/x videos. Until the clip is complete.

// Ensure that x[i].video_length + x[i+1].video_length + ... >= MAX_VIDEO_DURATION

#[allow(dead_code)]
fn valid_aggregate_video_duration(videos: &[PathBuf], max_duration: f64) -> bool {
    let mut running_duration = 0.0;
    for video in videos.iter() {
        running_duration += media::duration_seconds(video).unwrap_or_else(|error| {
            eprintln!("Error determining duration of video: {error}");
            0.0
        })
    }
    return running_duration < max_duration;
}
