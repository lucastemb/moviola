use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Audio,
}

const SUPPORTED_VIDEO_EXTENSIONS: &[&str] =
    &["mp4", "mov", "m4v", "mkv", "webm", "avi", "mpeg", "mpg"];

const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "m4a", "aac", "flac", "ogg", "opus", "aiff", "aif",
];

impl MediaKind {
    pub fn supported_extensions(self) -> &'static [&'static str] {
        match self {
            Self::Video => SUPPORTED_VIDEO_EXTENSIONS,
            Self::Audio => SUPPORTED_AUDIO_EXTENSIONS,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
        }
    }

    pub fn default_output_extension(self) -> &'static str {
        match self {
            Self::Video => "mp4",
            Self::Audio => "m4a",
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        Self::from_extension(extension)
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        let extension = extension.to_ascii_lowercase();

        [Self::Video, Self::Audio]
            .into_iter()
            .find(|kind| kind.supported_extensions().contains(&extension.as_str()))
    }
}

pub fn default_output_path(input_path: &Path, suffix: &str, media_kind: MediaKind) -> PathBuf {
    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = input_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("output");
    let extension = media_kind.default_output_extension();

    parent.join(format!("{stem}_{suffix}.{extension}"))
}
