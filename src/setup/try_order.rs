use std::io;

use toml_edit::{Array, DocumentMut, Item, Table, Value};

use crate::prompts::input::prompt_with_default;
use crate::prompts::prompt_yes_no;

use super::common::{Action, prompt_action};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    println!();
    println!("[servers.try]");
    let current = current_try_list(doc);
    let current_display = if current.is_empty() {
        "<未設定>".to_string()
    } else {
        current.join(", ")
    };
    println!("接続順序(try): {current_display}");
    println!("プレイヤーがログイン/切断時に接続を試すサーバ順です。");

    match prompt_action("接続順序(try)", true)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            if let Some(table) = doc.get_mut("servers").and_then(Item::as_table_mut) {
                table.remove("try");
            }
            Ok(())
        }
        Action::Edit => edit_try_list(doc, &current),
    }
}

fn edit_try_list(doc: &mut DocumentMut, current: &[String]) -> io::Result<()> {
    let default_value = if current.is_empty() {
        String::new()
    } else {
        current.join(",")
    };

    loop {
        let input = prompt_with_default("try (カンマ区切り)", &default_value)?;
        let list = input
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();
        if list.is_empty() {
            println!("接続順序は空にできません。");
            continue;
        }

        let servers = server_names(doc);
        let unknown = list
            .iter()
            .filter(|name| !servers.contains(name))
            .cloned()
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            let confirm = prompt_yes_no(
                &format!(
                    "未定義のサーバが含まれています: {}。続行しますか？",
                    unknown.join(", ")
                ),
                false,
            )?;
            if !confirm {
                continue;
            }
        }

        let table = ensure_servers_table(doc);
        let mut array = Array::new();
        for name in list {
            array.push(name);
        }
        table["try"] = Item::Value(Value::Array(array));
        return Ok(());
    }
}

fn current_try_list(doc: &DocumentMut) -> Vec<String> {
    doc.get("servers")
        .and_then(Item::as_table)
        .and_then(|table| table.get("try"))
        .and_then(Item::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn server_names(doc: &DocumentMut) -> Vec<String> {
    doc.get("servers")
        .and_then(Item::as_table)
        .map(|table| {
            table
                .iter()
                .filter_map(|(key, value)| {
                    if key == "try" || !value.is_value() {
                        None
                    } else {
                        Some(key.to_string())
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn ensure_servers_table(doc: &mut DocumentMut) -> &mut Table {
    if !doc.as_table().contains_key("servers") || !doc["servers"].is_table() {
        doc["servers"] = Item::Table(Table::new());
    }
    doc["servers"].as_table_mut().expect("servers table")
}
