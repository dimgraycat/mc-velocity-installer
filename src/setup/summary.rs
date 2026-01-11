use std::collections::{BTreeMap, BTreeSet};

use toml_edit::{DocumentMut, Item, Value};

pub(crate) fn summarize_changes(original: &DocumentMut, updated: &DocumentMut) -> Vec<String> {
    let mut changes = Vec::new();

    changes.extend(diff_scalar("bind", original, updated));
    changes.extend(diff_scalar("motd", original, updated));
    changes.extend(diff_scalar("show-max-players", original, updated));
    changes.extend(diff_scalar("online-mode", original, updated));
    changes.extend(diff_scalar("force-key-authentication", original, updated));
    changes.extend(diff_scalar(
        "player-info-forwarding-mode",
        original,
        updated,
    ));
    changes.extend(diff_scalar("forwarding-secret-file", original, updated));

    changes.extend(diff_servers(original, updated));
    changes.extend(diff_try_order(original, updated));
    changes.extend(diff_forced_hosts(original, updated));

    changes
}

fn diff_scalar(label: &str, original: &DocumentMut, updated: &DocumentMut) -> Vec<String> {
    let before = display_item(original.get(label));
    let after = display_item(updated.get(label));
    if before == after {
        Vec::new()
    } else {
        vec![format!(
            "{label}: {} -> {}",
            before.unwrap_or_else(|| "(未設定)".to_string()),
            after.unwrap_or_else(|| "(削除)".to_string())
        )]
    }
}

fn diff_servers(original: &DocumentMut, updated: &DocumentMut) -> Vec<String> {
    let before = servers_map(original);
    let after = servers_map(updated);
    diff_map("servers", before, after)
}

fn diff_forced_hosts(original: &DocumentMut, updated: &DocumentMut) -> Vec<String> {
    let before = forced_hosts_map(original);
    let after = forced_hosts_map(updated);
    diff_map("forced-hosts", before, after)
}

fn diff_try_order(original: &DocumentMut, updated: &DocumentMut) -> Vec<String> {
    let before = try_list(original);
    let after = try_list(updated);
    if before == after {
        Vec::new()
    } else {
        vec![format!(
            "servers.try: {} -> {}",
            list_display(&before),
            list_display(&after)
        )]
    }
}

fn diff_map(
    label: &str,
    before: BTreeMap<String, String>,
    after: BTreeMap<String, String>,
) -> Vec<String> {
    let mut changes = Vec::new();
    let keys: BTreeSet<String> = before
        .keys()
        .cloned()
        .chain(after.keys().cloned())
        .collect();

    for key in keys {
        match (before.get(&key), after.get(&key)) {
            (Some(old), Some(new)) if old == new => {}
            (Some(old), Some(new)) => changes.push(format!("{label}: 変更 {key} = {old} -> {new}")),
            (None, Some(new)) => changes.push(format!("{label}: 追加 {key} = {new}")),
            (Some(old), None) => changes.push(format!("{label}: 削除 {key} = {old}")),
            (None, None) => {}
        }
    }
    changes
}

fn servers_map(doc: &DocumentMut) -> BTreeMap<String, String> {
    doc.get("servers")
        .and_then(Item::as_table)
        .map(|table| {
            table
                .iter()
                .filter_map(|(key, value)| {
                    if key == "try" {
                        None
                    } else {
                        value
                            .as_str()
                            .map(|addr| (key.to_string(), addr.to_string()))
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn forced_hosts_map(doc: &DocumentMut) -> BTreeMap<String, String> {
    doc.get("forced-hosts")
        .and_then(Item::as_table)
        .map(|table| {
            table
                .iter()
                .filter_map(|(key, value)| value.as_array().map(|array| (key, array)))
                .map(|(key, array)| (key.to_string(), list_display(&array_list(array))))
                .collect()
        })
        .unwrap_or_default()
}

fn try_list(doc: &DocumentMut) -> Vec<String> {
    doc.get("servers")
        .and_then(Item::as_table)
        .and_then(|table| table.get("try"))
        .and_then(Item::as_array)
        .map(array_list)
        .unwrap_or_default()
}

fn array_list(array: &toml_edit::Array) -> Vec<String> {
    array
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn list_display(list: &[String]) -> String {
    if list.is_empty() {
        "(未設定)".to_string()
    } else {
        format!("[{}]", list.join(", "))
    }
}

fn display_item(item: Option<&Item>) -> Option<String> {
    let value = item.and_then(Item::as_value)?;
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    if let Some(value) = value.as_bool() {
        return Some(value.to_string());
    }
    if let Some(value) = value.as_integer() {
        return Some(value.to_string());
    }
    if let Some(value) = value.as_float() {
        return Some(value.to_string());
    }
    if let Some(array) = value.as_array() {
        return Some(list_display(&array_list(array)));
    }
    Some(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(input: &str) -> DocumentMut {
        input.parse::<DocumentMut>().expect("parse toml")
    }

    #[test]
    fn summarize_changes_reports_key_updates() {
        let before = doc(r#"
bind = "0.0.0.0:25565"

[servers]
lobby = "127.0.0.1:30066"
try = ["lobby"]
"#);
        let after = doc(r#"
bind = "0.0.0.0:25566"

[servers]
lobby = "127.0.0.1:30066"
pvp = "127.0.0.1:30067"
try = ["lobby", "pvp"]
"#);

        let changes = summarize_changes(&before, &after);
        assert!(
            changes
                .iter()
                .any(|line| line.contains("bind: 0.0.0.0:25565 -> 0.0.0.0:25566"))
        );
        assert!(
            changes
                .iter()
                .any(|line| line.contains("servers: 追加 pvp = 127.0.0.1:30067"))
        );
        assert!(
            changes
                .iter()
                .any(|line| line.contains("servers.try: [lobby] -> [lobby, pvp]"))
        );
    }
}
