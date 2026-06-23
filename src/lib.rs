pub mod env;
pub mod media;
pub mod tools;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run() -> Result<()> {
    tools::Tool::from_env()?.run()
}
