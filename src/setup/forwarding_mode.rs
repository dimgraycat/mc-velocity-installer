use std::io;

use toml_edit::{DocumentMut, Item, value};

use crate::prompts::input::prompt_usize_with_default;

use super::common::{Action, prompt_action};

const OPTIONS: [&str; 4] = ["none", "legacy", "bungeeguard", "modern"];

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    let current = doc
        .get("player-info-forwarding-mode")
        .and_then(Item::as_str)
        .unwrap_or("NONE")
        .to_ascii_lowercase();
    println!();
    println!("player-info-forwarding-mode: {current}");
    match prompt_action("player-info-forwarding-mode", false)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("player-info-forwarding-mode");
            Ok(())
        }
        Action::Edit => {
            println!("利用可能なモード:");
            for (idx, option) in OPTIONS.iter().enumerate() {
                println!("{:>3}. {}", idx + 1, option);
            }
            let default_index = OPTIONS
                .iter()
                .position(|option| *option == current)
                .map(|idx| idx + 1)
                .unwrap_or(1);
            let selection = prompt_usize_with_default(
                "番号で選択してください",
                default_index,
                1..=OPTIONS.len(),
            )?;
            let chosen = OPTIONS[selection - 1].to_ascii_uppercase();
            doc["player-info-forwarding-mode"] = value(chosen);
            Ok(())
        }
    }
}
