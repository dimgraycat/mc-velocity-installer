use std::io;

use toml_edit::{DocumentMut, Item, value};

use crate::prompts::prompt_yes_no;

use super::common::{Action, prompt_action};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    let current = doc
        .get("force-key-authentication")
        .and_then(Item::as_bool)
        .unwrap_or(true);
    println!();
    println!("force-key-authentication: {current}");
    match prompt_action("force-key-authentication", false)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("force-key-authentication");
            Ok(())
        }
        Action::Edit => {
            let new_value = prompt_yes_no("force-key-authentication を有効にしますか？", current)?;
            doc["force-key-authentication"] = value(new_value);
            Ok(())
        }
    }
}
