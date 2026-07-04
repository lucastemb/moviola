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
    optional_number(name)
}

pub fn required_f64(name: &str) -> Result<f64> {
    let value = required_string(name)?;
    parse_number(name, &value)
}

pub fn optional_f64(name: &str) -> Result<Option<f64>> {
    optional_number(name)
}

fn optional_number<T>(name: &str) -> Result<Option<T>>
where
    T: std::str::FromStr,
{
    let Some(value) = env::var(name).ok().filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };

    Ok(Some(parse_number(name, &value)?))
}

fn parse_number<T>(name: &str, value: &str) -> Result<T>
where
    T: std::str::FromStr,
{
    value
        .parse::<T>()
        .map_err(|_| format!("{name} must be a number, got '{value}'").into())
}
