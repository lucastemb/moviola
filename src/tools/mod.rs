pub mod chop_video;
pub mod trim_silence;
pub mod video_sequencer;

use crate::Result;
use crate::env;

pub enum Tool {
    ChopVideo,
    TrimSilence,
    VideoSequencer,
}

impl Tool {
    pub const ENV_KEY: &'static str = "MOVIOLA_TOOL";

    pub fn from_env() -> Result<Self> {
        let tool_name = env::required_string(Self::ENV_KEY)?;
        Self::from_name(&tool_name)
    }

    pub fn from_name(name: &str) -> Result<Self> {
        match name {
            "chop-video" => Ok(Self::ChopVideo),
            "trim-silence" => Ok(Self::TrimSilence),
            "video-sequencer" => Ok(Self::VideoSequencer),
            unknown => Err(format!(
                "Unknown tool '{unknown}'. Available tools: {}",
                Self::available_tools().join(", ")
            )
            .into()),
        }
    }

    pub fn available_tools() -> &'static [&'static str] {
        &["chop-video", "trim-silence", "video-sequencer"]
    }

    pub fn run(self) -> Result<()> {
        match self {
            Self::ChopVideo => chop_video::run(),
            Self::TrimSilence => trim_silence::run(),
            Self::VideoSequencer => video_sequencer::run(),
        }
    }
}
