use std::io;

use toml_edit::{DocumentMut, Item, Table, value};

use crate::prompts::input::prompt_line;

use super::common::{Action, prompt_action};

pub(crate) fn apply(doc: &mut DocumentMut) -> io::Result<()> {
    println!();
    println!("[servers]");
    println!("バックエンドサーバ一覧: プロキシが接続するサーバ定義です。");
    println!("例: lobby = \"127.0.0.1:30066\"");
    match prompt_action("バックエンドサーバ一覧(servers)", true)? {
        Action::Skip => Ok(()),
        Action::Delete => {
            doc.remove("servers");
            Ok(())
        }
        Action::Edit => edit_servers(doc),
    }
}

fn edit_servers(doc: &mut DocumentMut) -> io::Result<()> {
    let table = ensure_servers_table(doc);
    loop {
        print_servers(table);
        let input =
            prompt_line("操作を選択してください [a]追加 [e]編集 [r]削除 [f]完了 (default: f): ")?;
        let trimmed = input.trim().to_ascii_lowercase();
        if trimmed.is_empty() || trimmed == "f" {
            break;
        }
        match trimmed.as_str() {
            "a" => add_server(table)?,
            "e" => edit_server(table)?,
            "r" => remove_server(table)?,
            _ => println!("入力が無効です。"),
        }
    }
    Ok(())
}

fn ensure_servers_table(doc: &mut DocumentMut) -> &mut Table {
    if !doc.as_table().contains_key("servers") || !doc["servers"].is_table() {
        doc["servers"] = Item::Table(Table::new());
    }
    doc["servers"].as_table_mut().expect("servers table")
}

fn print_servers(table: &Table) {
    let mut entries = Vec::new();
    for (key, value) in table.iter() {
        if key == "try" {
            continue;
        }
        let addr = value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| "<非対応の値>".to_string());
        entries.push(format!("{key} = {addr}"));
    }
    if entries.is_empty() {
        println!("サーバは未設定です。");
    } else {
        println!("現在のサーバ一覧:");
        for entry in entries {
            println!("- {entry}");
        }
    }
}

fn add_server(table: &mut Table) -> io::Result<()> {
    let name = prompt_line("サーバ名: ")?;
    let name = name.trim();
    if name.is_empty() {
        println!("サーバ名は空にできません。");
        return Ok(());
    }
    if name == "try" {
        println!("\"try\" は予約語のため使用できません。");
        return Ok(());
    }
    if table.contains_key(name) {
        println!("同名のサーバが既に存在します。");
        return Ok(());
    }
    let addr = prompt_line("サーバアドレス: ")?;
    let addr = addr.trim();
    if addr.is_empty() {
        println!("サーバアドレスは空にできません。");
        return Ok(());
    }
    table.insert(name, value(addr.to_string()));
    Ok(())
}

fn edit_server(table: &mut Table) -> io::Result<()> {
    let name = prompt_line("編集するサーバ名: ")?;
    let name = name.trim();
    if name.is_empty() {
        println!("サーバ名は空にできません。");
        return Ok(());
    }
    if !table.contains_key(name) {
        println!("指定したサーバが見つかりません。");
        return Ok(());
    }
    let addr = prompt_line("新しいサーバアドレス: ")?;
    let addr = addr.trim();
    if addr.is_empty() {
        println!("サーバアドレスは空にできません。");
        return Ok(());
    }
    table.insert(name, value(addr.to_string()));
    Ok(())
}

fn remove_server(table: &mut Table) -> io::Result<()> {
    let name = prompt_line("削除するサーバ名: ")?;
    let name = name.trim();
    if name.is_empty() {
        println!("サーバ名は空にできません。");
        return Ok(());
    }
    if table.remove(name).is_none() {
        println!("指定したサーバが見つかりません。");
    }
    Ok(())
}
