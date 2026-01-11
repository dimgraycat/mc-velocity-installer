use std::io;

use toml_edit::{DocumentMut, Item};

use super::common::{Action, prompt_action, prompt_u32_with_default};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    let current = doc
        .get("show-max-players")
        .and_then(Item::as_integer)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(500);
    println!();
    println!("show-max-players: {current}");
    match prompt_action("show-max-players", false)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("show-max-players");
            Ok(())
        }
        Action::Edit => {
            let new_value = i64::from(prompt_u32_with_default("show-max-players", current)?);
            doc["show-max-players"] = Item::Value(new_value.into());
            Ok(())
        }
    }
}
