use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::Result;
use crate::media::MediaKind;

// Validates whether path belongs to supported media type (audio or video)
pub fn validate_path(path: &Path, media_type: MediaKind) -> Result<()> {
    let Some(extension) = path.extension().and_then(OsStr::to_str) else {
        return Err(format!("Input file must have a {} extension", media_type.label()).into());
    };

    if !is_supported_extension(media_type, extension) {
        return Err(format!(
            "Unsupported file extension '.{extension}'. Supported {} extensions: {}",
            media_type.label(),
            media_type.supported_extensions().join(", ")
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

pub fn ensure_output_parent_exists(output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

// Checks whether or not the current extension is supported
fn is_supported_extension(media_type: MediaKind, extension: &str) -> bool {
    let extension = extension.to_ascii_lowercase();
    media_type
        .supported_extensions()
        .contains(&extension.as_str())
}
