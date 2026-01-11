mod bind;
mod common;
mod config_path;
mod force_key_authentication;
mod forced_hosts;
mod forwarding_mode;
mod forwarding_secret_file;
mod motd;
mod online_mode;
mod servers;
mod show_max_players;
mod summary;
mod try_order;

use std::error::Error;
use std::fs;

use toml_edit::DocumentMut;

use crate::prompts::prompt_yes_no;

pub(crate) fn run_setup() -> Result<(), Box<dyn Error>> {
    println!("--setup: velocity.toml を対話式で変更します。");
    let config_path = config_path::prompt_config_path()?;
    if !config_path.exists() {
        return Err(format!("設定ファイルが見つかりません: {}", config_path.display()).into());
    }

    let contents = fs::read_to_string(&config_path)?;
    let use_crlf = contents.contains("\r\n");
    let original_doc: DocumentMut = contents.parse()?;
    let mut doc: DocumentMut = contents.parse()?;

    bind::apply(&mut doc)?;
    motd::apply(&mut doc)?;
    show_max_players::apply(&mut doc)?;
    online_mode::apply(&mut doc)?;
    force_key_authentication::apply(&mut doc)?;
    forwarding_mode::apply(&mut doc)?;
    forwarding_secret_file::apply(&mut doc)?;
    servers::apply(&mut doc)?;
    try_order::apply(&mut doc)?;
    forced_hosts::cleanup_invalid_references(&mut doc)?;

    let mut new_contents = doc.to_string();
    if normalize_line_endings(&new_contents) == normalize_line_endings(&contents) {
        println!("変更がないため保存しません。");
        return Ok(());
    }

    let changes = summary::summarize_changes(&original_doc, &doc);
    println!();
    println!("変更内容:");
    if changes.is_empty() {
        println!("(変更点の詳細を取得できませんでした)");
    } else {
        for change in changes {
            println!("- {change}");
        }
    }

    if !prompt_yes_no("この内容で保存しますか？", true)? {
        println!("保存を中断しました。");
        return Ok(());
    }

    if use_crlf {
        new_contents = new_contents.replace('\n', "\r\n");
    }
    fs::write(&config_path, new_contents)?;
    println!("保存しました: {}", config_path.display());
    Ok(())
}

fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n")
}
