use std::io;

use toml_edit::{Array, DocumentMut, Item, Table, Value};

use crate::prompts::prompt_yes_no;

pub(crate) fn cleanup_invalid_references(doc: &mut DocumentMut) -> io::Result<()> {
    let server_names = server_names(doc);
    let Some(forced_hosts) = doc.get_mut("forced-hosts").and_then(Item::as_table_mut) else {
        return Ok(());
    };

    let invalid = collect_invalid(forced_hosts, &server_names);
    if invalid.is_empty() {
        return Ok(());
    }

    println!();
    println!("[forced-hosts] 未定義サーバ参照があります。");
    println!("forced-hosts は ホスト名 -> servers の紐付けです。");
    println!("servers に存在しないサーバ名だけを削除できます。");
    for entry in &invalid {
        println!("- {}:", entry.host);
        println!("  現在: {}", entry.all.join(", "));
        println!("  未定義: {}", entry.invalid.join(", "));
    }

    if !prompt_yes_no("servers に存在しないサーバ名のみ削除しますか？", false)? {
        return Ok(());
    }

    for entry in invalid {
        if let Some(item) = forced_hosts.get_mut(&entry.host)
            && let Some(array) = item.as_array_mut()
        {
            let mut new_array = Array::new();
            for value in array.iter() {
                match value.as_str() {
                    Some(name) if entry.invalid.contains(&name.to_string()) => {}
                    _ => new_array.push(value.clone()),
                }
            }
            *array = new_array;
            if array.is_empty() {
                let remove_host = prompt_yes_no(
                    &format!(
                        "\"{}\" の forced-hosts が空になります。ホスト自体も削除しますか？",
                        entry.host
                    ),
                    true,
                )?;
                if remove_host {
                    forced_hosts.remove(&entry.host);
                }
            }
        }
    }

    Ok(())
}

fn collect_invalid(table: &Table, servers: &[String]) -> Vec<InvalidEntry> {
    let mut result = Vec::new();
    for (host, item) in table.iter() {
        let Some(array) = item.as_array() else {
            continue;
        };
        let all = array
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let invalid = array
            .iter()
            .filter_map(Value::as_str)
            .filter(|name| !servers.contains(&name.to_string()))
            .map(str::to_string)
            .collect::<Vec<_>>();
        if !invalid.is_empty() {
            result.push(InvalidEntry {
                host: host.to_string(),
                all,
                invalid,
            });
        }
    }
    result
}

struct InvalidEntry {
    host: String,
    all: Vec<String>,
    invalid: Vec<String>,
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
