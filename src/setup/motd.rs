use std::io;

use toml_edit::{DocumentMut, Item, value};

use crate::prompts::input::prompt_with_default;

use super::common::{Action, prompt_action};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    let current = doc
        .get("motd")
        .and_then(Item::as_str)
        .unwrap_or("<#09add3>A Velocity Server");
    println!();
    println!("motd: {current}");
    match prompt_action("motd", false)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("motd");
            Ok(())
        }
        Action::Edit => {
            let new_value = prompt_with_default("motd", current)?;
            doc["motd"] = value(new_value);
            Ok(())
        }
    }
}
