use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::input::{prompt_with_default, prompt_yes_no};

pub(crate) fn prompt_install_dir() -> io::Result<PathBuf> {
    let default_dir = default_install_dir();
    let default_display = default_dir.to_string_lossy();
    loop {
        let input = prompt_with_default("インストール先ディレクトリ", &default_display)?;
        let path = PathBuf::from(input);
        if path.as_os_str().is_empty() {
            println!("空のパスは指定できません。");
            continue;
        }
        let confirm = prompt_yes_no(
            &format!("インストール先は {} でよいですか？", path.display()),
            true,
        )?;
        if confirm {
            return Ok(path);
        }
    }
}

pub(crate) fn confirm_existing_install(path: &Path) -> Result<bool, Box<dyn Error>> {
    if path.exists() {
        if !path.is_dir() {
            return Err("インストール先がディレクトリではありません。".into());
        }
        let mut entries = fs::read_dir(path)?;
        if entries.next().is_some() {
            let confirm =
                prompt_yes_no("既存ファイルが存在します。上書きして続行しますか？", false)?;
            if !confirm {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn default_install_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("velocity")
}
