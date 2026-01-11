use std::io;

use crate::prompts::input::prompt_line;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Skip,
    Edit,
    Delete,
}

pub(crate) fn prompt_action(label: &str, allow_delete: bool) -> io::Result<Action> {
    let options = if allow_delete {
        "[s]スキップ [e]変更 [d]削除"
    } else {
        "[s]スキップ [e]変更"
    };
    loop {
        let input = prompt_line(&format!(
            "{label} を変更しますか？ {options} (default: s): "
        ))?;
        let trimmed = input.trim().to_ascii_lowercase();
        if trimmed.is_empty() || trimmed == "s" {
            return Ok(Action::Skip);
        }
        if trimmed == "e" {
            return Ok(Action::Edit);
        }
        if allow_delete && trimmed == "d" {
            return Ok(Action::Delete);
        }
        println!("入力が無効です。");
    }
}

pub(crate) fn prompt_u32_with_default(label: &str, default: u32) -> io::Result<u32> {
    loop {
        let input = prompt_line(&format!("{label} [{default}]: "))?;
        if input.trim().is_empty() {
            return Ok(default);
        }
        match input.trim().parse::<u32>() {
            Ok(value) => return Ok(value),
            Err(_) => println!("数値を入力してください。"),
        }
    }
}
