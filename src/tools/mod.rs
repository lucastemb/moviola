pub mod trim_silence;

use crate::Result;
use crate::env;

pub enum Tool {
    TrimSilence,
}

impl Tool {
    pub const ENV_KEY: &'static str = "MOVIOLA_TOOL";

    pub fn from_env() -> Result<Self> {
        let tool_name = env::required_string(Self::ENV_KEY)?;
        Self::from_name(&tool_name)
    }

    pub fn from_name(name: &str) -> Result<Self> {
        match name {
            "trim-silence" => Ok(Self::TrimSilence),
            unknown => Err(format!(
                "Unknown tool '{unknown}'. Available tools: {}",
                Self::available_tools().join(", ")
            )
            .into()),
        }
    }

    pub fn available_tools() -> &'static [&'static str] {
        &["trim-silence"]
    }

    pub fn run(self) -> Result<()> {
        match self {
            Self::TrimSilence => trim_silence::run(),
        }
    }
}
