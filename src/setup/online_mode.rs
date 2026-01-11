use std::io;

use toml_edit::{DocumentMut, Item, value};

use crate::prompts::prompt_yes_no;

use super::common::{Action, prompt_action};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    let current = doc
        .get("online-mode")
        .and_then(Item::as_bool)
        .unwrap_or(true);
    println!();
    println!("online-mode: {current}");
    match prompt_action("online-mode", false)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("online-mode");
            Ok(())
        }
        Action::Edit => {
            let new_value = prompt_yes_no("online-mode を有効にしますか？", current)?;
            doc["online-mode"] = value(new_value);
            Ok(())
        }
    }
}
