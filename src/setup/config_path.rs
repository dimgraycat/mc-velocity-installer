use std::io;
use std::path::PathBuf;

use crate::prompts::input::prompt_with_default;

pub(crate) fn prompt_config_path() -> io::Result<PathBuf> {
    let default = default_config_path();
    let default_display = default.to_string_lossy();
    let input = prompt_with_default("velocity.toml のパス", &default_display)?;
    Ok(PathBuf::from(input))
}

fn default_config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("velocity")
        .join("velocity.toml")
}
