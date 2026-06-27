use std::env;
use std::path::PathBuf;

use crate::Result;

pub fn required_string(name: &str) -> Result<String> {
    let value =
        env::var(name).map_err(|_| format!("Missing required environment variable {name}"))?;

    if value.trim().is_empty() {
        return Err(format!("{name} cannot be empty").into());
    }

    Ok(value)
}

pub fn required_path(name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(required_string(name)?))
}

pub fn optional_path(name: &str) -> Option<PathBuf> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

pub fn optional_f32(name: &str) -> Result<Option<f32>> {
    let Some(value) = env::var(name).ok().filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };

    Ok(Some(value.parse::<f32>().map_err(|_| {
        format!("{name} must be a number, got '{value}'")
    })?))
}
