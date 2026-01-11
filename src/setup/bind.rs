use std::io;

use toml_edit::{DocumentMut, Item, value};

use crate::prompts::input::prompt_with_default;

use super::common::{Action, prompt_action};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    let current = doc
        .get("bind")
        .and_then(Item::as_str)
        .unwrap_or("0.0.0.0:25565");
    println!();
    println!("bind: {current}");
    match prompt_action("bind", false)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("bind");
            Ok(())
        }
        Action::Edit => {
            let new_value = prompt_with_default("bind", current)?;
            doc["bind"] = value(new_value);
            Ok(())
        }
    }
}
