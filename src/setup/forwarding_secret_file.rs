use std::io;

use toml_edit::{DocumentMut, Item, value};

use crate::prompts::input::prompt_with_default;

use super::common::{Action, prompt_action};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    let current = doc
        .get("forwarding-secret-file")
        .and_then(Item::as_str)
        .unwrap_or("forwarding.secret");
    println!();
    println!("forwarding-secret-file: {current}");
    match prompt_action("forwarding-secret-file", false)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("forwarding-secret-file");
            Ok(())
        }
        Action::Edit => {
            let new_value = prompt_with_default("forwarding-secret-file", current)?;
            doc["forwarding-secret-file"] = value(new_value);
            Ok(())
        }
    }
}
