use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const VERSION_INDEX_URL: &str = "https://minedeck.github.io/jars/velocity.json";
const DEFAULT_INSTALL_DIR: &str = "/opt/";
const DEFAULT_BIND: &str = "0.0.0.0:25565";
const DEFAULT_MOTD: &str = "<#09add3>A Velocity Server";
const DEFAULT_SHOW_MAX_PLAYERS: u32 = 500;
const DEFAULT_XMS: &str = "256M";
const DEFAULT_XMX: &str = "512M";
const DEFAULT_FORWARDING_SECRET_FILE: &str = "forwarding.secret";
const CONFIG_VERSION: &str = "2.7";

#[derive(Debug, Deserialize)]
struct VelocityIndex {
    status: Option<String>,
    data: BTreeMap<String, VelocityEntry>,
}

#[derive(Debug, Deserialize)]
struct VelocityEntry {
    url: String,
    checksum: Checksum,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
struct Checksum {
    sha256: Option<String>,
}

#[derive(Debug, Clone)]
struct VersionInfo {
    version: String,
    kind: String,
    url: String,
    sha256: String,
}

#[derive(Debug, Clone)]
struct BackendServer {
    name: String,
    address: String,
}

#[derive(Debug)]
struct ServerConfig {
    servers: Vec<BackendServer>,
    try_order: Vec<String>,
}

#[derive(Debug)]
struct InstallSettings {
    install_dir: PathBuf,
    version: VersionInfo,
    bind: String,
    motd: String,
    show_max_players: u32,
    online_mode: bool,
    force_key_authentication: bool,
    forwarding_mode: String,
    forwarding_secret: Option<String>,
    servers: Vec<BackendServer>,
    try_order: Vec<String>,
    xms: String,
    xmx: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("エラー: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    if std::env::args().skip(1).any(|arg| arg == "--update") {
        println!("アップデート機能は未対応です。新規インストールのみ対応しています。");
        return Ok(());
    }

    println!("mc-velocity-installer (新規インストール)");
    println!("Java はインストール済みであることを前提に進めます。");
    println!();

    let install_dir = prompt_install_dir()?;
    if !confirm_existing_install(&install_dir)? {
        println!("中断しました。");
        return Ok(());
    }

    let client = Client::builder()
        .user_agent("mc-velocity-installer")
        .build()?;

    println!("Velocity のバージョン一覧を取得しています...");
    let versions = fetch_versions(&client)?;
    let version = prompt_version(&versions)?;

    let server_config = prompt_servers()?;
    let forwarding_mode = prompt_forwarding_mode()?;
    let forwarding_secret = if matches!(forwarding_mode.as_str(), "BUNGEGUARD" | "MODERN") {
        Some(prompt_non_empty(
            "共有シークレット (forwarding.secret に保存)",
        )?)
    } else {
        None
    };

    let bind = prompt_with_default("リッスンアドレス/ポート", DEFAULT_BIND)?;
    let motd = prompt_with_default("MOTD", DEFAULT_MOTD)?;
    let show_max_players = prompt_u32_with_default("プレイヤー数表示", DEFAULT_SHOW_MAX_PLAYERS)?;
    let online_mode = prompt_yes_no("オンラインモードを有効にしますか？", true)?;
    let force_key_authentication = prompt_yes_no("鍵認証を強制しますか？", true)?;

    let (xms, xmx) = prompt_memory()?;

    let settings = InstallSettings {
        install_dir,
        version,
        bind,
        motd,
        show_max_players,
        online_mode,
        force_key_authentication,
        forwarding_mode,
        forwarding_secret,
        servers: server_config.servers,
        try_order: server_config.try_order,
        xms,
        xmx,
    };

    print_summary(&settings);
    if !prompt_yes_no("この内容で実行しますか？", true)? {
        println!("中断しました。");
        return Ok(());
    }

    perform_install(&client, &settings)?;
    println!();
    println!("完了しました。");
    println!(
        "起動するには {} を実行してください。",
        settings.install_dir.join("start.sh").display()
    );
    println!(
        "設定ファイルは {} です。",
        settings.install_dir.join("velocity.toml").display()
    );

    Ok(())
}

fn prompt_install_dir() -> io::Result<PathBuf> {
    loop {
        let input = prompt_with_default("インストール先ディレクトリ", DEFAULT_INSTALL_DIR)?;
        let path = PathBuf::from(input);
        if path.as_os_str().is_empty() {
            println!("空のパスは指定できません。");
            continue;
        }
        let confirm = prompt_yes_no(
            &format!("インストール先は {} でよいですか？", path.display()),
            true,
        )?;
        if confirm {
            return Ok(path);
        }
    }
}

fn confirm_existing_install(path: &Path) -> Result<bool, Box<dyn Error>> {
    if path.exists() {
        if !path.is_dir() {
            return Err("インストール先がディレクトリではありません。".into());
        }
        let mut entries = fs::read_dir(path)?;
        if entries.next().is_some() {
            let confirm =
                prompt_yes_no("既存ファイルが存在します。上書きして続行しますか？", false)?;
            if !confirm {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn fetch_versions(client: &Client) -> Result<Vec<VersionInfo>, Box<dyn Error>> {
    let text = client
        .get(VERSION_INDEX_URL)
        .send()?
        .error_for_status()?
        .text()?;
    let index: VelocityIndex = serde_json::from_str(&text)?;
    if let Some(status) = index.status.as_deref()
        && status != "ok"
    {
        return Err(format!("バージョン一覧の取得に失敗しました: status={status}").into());
    }

    let mut versions = Vec::new();
    for (version, entry) in index.data {
        let sha256 = entry
            .checksum
            .sha256
            .ok_or_else(|| format!("sha256 が見つかりません: {version}"))?;
        versions.push(VersionInfo {
            version,
            kind: entry.kind,
            url: entry.url,
            sha256,
        });
    }

    versions.sort_by(|a, b| b.version.cmp(&a.version));
    if versions.is_empty() {
        return Err("バージョン一覧が空です。".into());
    }
    Ok(versions)
}

fn prompt_version(versions: &[VersionInfo]) -> io::Result<VersionInfo> {
    loop {
        println!();
        println!("利用可能なバージョン一覧:");
        for (idx, version) in versions.iter().enumerate() {
            println!("{:>3}. {} ({})", idx + 1, version.version, version.kind);
        }
        let selection = prompt_usize_with_default("番号で選択してください", 1, 1..=versions.len())?;
        let chosen = versions[selection - 1].clone();
        let confirm = prompt_yes_no(
            &format!("{} ({}) を選択しますか？", chosen.version, chosen.kind),
            true,
        )?;
        if confirm {
            return Ok(chosen);
        }
    }
}

fn prompt_servers() -> io::Result<ServerConfig> {
    let mut servers: Vec<BackendServer> = Vec::new();
    loop {
        let name_default = if servers.is_empty() { "lobby" } else { "" };
        let addr_default = if servers.is_empty() {
            "127.0.0.1:30066"
        } else {
            ""
        };
        let name = prompt_with_default("バックエンドサーバ名", name_default)?;
        if name.trim().is_empty() {
            println!("サーバ名は空にできません。");
            continue;
        }
        if name.contains(',') {
            println!("サーバ名にカンマは使用できません。");
            continue;
        }
        if servers.iter().any(|existing| existing.name == name) {
            println!("同じサーバ名が既に存在します。");
            continue;
        }
        let addr = prompt_with_default("バックエンドサーバアドレス", addr_default)?;
        if addr.trim().is_empty() {
            println!("サーバアドレスは空にできません。");
            continue;
        }
        servers.push(BackendServer {
            name,
            address: addr,
        });

        let add_more = prompt_yes_no("他のバックエンドサーバを追加しますか？", false)?;
        if !add_more {
            break;
        }
    }

    let default_try = servers
        .iter()
        .map(|server| server.name.clone())
        .collect::<Vec<_>>()
        .join(",");

    let try_order = loop {
        let input = prompt_with_default("接続順序(カンマ区切り)", &default_try)?;
        let list = input
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();
        if list.is_empty() {
            println!("接続順序は空にできません。");
            continue;
        }
        let mut seen = std::collections::BTreeSet::new();
        if list.iter().any(|item| !seen.insert(item.clone())) {
            println!("接続順序に重複があります。");
            continue;
        }
        let unknown = list
            .iter()
            .filter(|name| !servers.iter().any(|server| server.name == **name))
            .cloned()
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            println!("未定義のサーバが含まれています: {}", unknown.join(", "));
            continue;
        }
        break list;
    };

    Ok(ServerConfig { servers, try_order })
}

fn prompt_forwarding_mode() -> io::Result<String> {
    let options = ["none", "legacy", "bungeeguard", "modern"];
    loop {
        println!();
        println!("プレイヤー情報転送モード:");
        for (idx, option) in options.iter().enumerate() {
            println!("{:>3}. {}", idx + 1, option);
        }
        let selection = prompt_usize_with_default("番号で選択してください", 1, 1..=options.len())?;
        let chosen = options[selection - 1];
        let confirm = prompt_yes_no(&format!("{} を選択しますか？", chosen), true)?;
        if confirm {
            return Ok(chosen.to_ascii_uppercase());
        }
    }
}

fn prompt_memory() -> io::Result<(String, String)> {
    loop {
        let xms = prompt_with_default("起動メモリ Xms", DEFAULT_XMS)?;
        let xmx = prompt_with_default("最大メモリ Xmx", DEFAULT_XMX)?;
        let confirm = prompt_yes_no(&format!("Xms={} / Xmx={} でよいですか？", xms, xmx), true)?;
        if confirm {
            return Ok((xms, xmx));
        }
    }
}

fn print_summary(settings: &InstallSettings) {
    println!();
    println!("設定サマリ:");
    println!("- インストール先: {}", settings.install_dir.display());
    println!(
        "- バージョン: {} ({})",
        settings.version.version, settings.version.kind
    );
    println!("- bind: {}", settings.bind);
    println!("- MOTD: {}", settings.motd);
    println!("- プレイヤー数表示: {}", settings.show_max_players);
    println!(
        "- オンラインモード: {}",
        if settings.online_mode {
            "有効"
        } else {
            "無効"
        }
    );
    println!(
        "- 鍵認証強制: {}",
        if settings.force_key_authentication {
            "有効"
        } else {
            "無効"
        }
    );
    println!(
        "- 転送モード: {}",
        settings.forwarding_mode.to_ascii_lowercase()
    );
    println!(
        "- 共有シークレット: {}",
        if settings.forwarding_secret.is_some() {
            "設定済み"
        } else {
            "なし"
        }
    );
    println!("- バックエンドサーバ:");
    for server in &settings.servers {
        println!("  - {} = {}", server.name, server.address);
    }
    println!("- 接続順序: {}", settings.try_order.join(", "));
    println!("- 起動メモリ: Xms={} / Xmx={}", settings.xms, settings.xmx);
}

fn perform_install(client: &Client, settings: &InstallSettings) -> Result<(), Box<dyn Error>> {
    if !settings.install_dir.exists() {
        fs::create_dir_all(&settings.install_dir)?;
    }

    let jar_path = settings.install_dir.join("velocity.jar");
    println!("ダウンロード中: {}", settings.version.url);
    download_with_sha256(client, &settings.version, &jar_path)?;

    let config_path = settings.install_dir.join("velocity.toml");
    let config_contents = build_velocity_config(settings);
    fs::write(&config_path, config_contents)?;

    if let Some(secret) = &settings.forwarding_secret {
        let secret_path = settings.install_dir.join(DEFAULT_FORWARDING_SECRET_FILE);
        fs::write(secret_path, secret)?;
    }

    write_start_scripts(settings)?;
    Ok(())
}

fn download_with_sha256(
    client: &Client,
    version: &VersionInfo,
    dest_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut response = client.get(&version.url).send()?.error_for_status()?;
    let mut file = File::create(dest_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        file.write_all(&buffer[..bytes_read])?;
    }
    let actual = format!("{:x}", hasher.finalize());
    let expected = version.sha256.to_ascii_lowercase();
    if actual != expected {
        let _ = fs::remove_file(dest_path);
        return Err(format!(
            "チェックサム不一致: expected={}, actual={}",
            expected, actual
        )
        .into());
    }
    Ok(())
}

fn build_velocity_config(settings: &InstallSettings) -> String {
    let mut out = String::new();
    out.push_str("# Config version. Do not change this\n");
    out.push_str(&format!("config-version = \"{}\"\n\n", CONFIG_VERSION));
    out.push_str(&format!("bind = \"{}\"\n", escape_toml(&settings.bind)));
    out.push_str(&format!("motd = \"{}\"\n", escape_toml(&settings.motd)));
    out.push_str(&format!(
        "show-max-players = {}\n",
        settings.show_max_players
    ));
    out.push_str(&format!("online-mode = {}\n", settings.online_mode));
    out.push_str(&format!(
        "force-key-authentication = {}\n",
        settings.force_key_authentication
    ));
    out.push_str("prevent-client-proxy-connections = false\n");
    out.push_str(&format!(
        "player-info-forwarding-mode = \"{}\"\n",
        settings.forwarding_mode
    ));
    out.push_str(&format!(
        "forwarding-secret-file = \"{}\"\n",
        DEFAULT_FORWARDING_SECRET_FILE
    ));
    out.push_str("announce-forge = false\n");
    out.push_str("kick-existing-players = false\n");
    out.push_str("ping-passthrough = \"DISABLED\"\n");
    out.push_str("sample-players-in-ping = false\n");
    out.push_str("enable-player-address-logging = true\n\n");

    out.push_str("[servers]\n");
    for server in &settings.servers {
        out.push_str(&format!(
            "\"{}\" = \"{}\"\n",
            escape_toml(&server.name),
            escape_toml(&server.address)
        ));
    }
    out.push_str("try = [\n");
    for name in &settings.try_order {
        out.push_str(&format!("    \"{}\",\n", escape_toml(name)));
    }
    out.push_str("]\n\n");

    out.push_str("[advanced]\n");
    out.push_str("compression-threshold = 256\n");
    out.push_str("compression-level = -1\n");
    out.push_str("login-ratelimit = 3000\n");
    out.push_str("connection-timeout = 5000\n");
    out.push_str("read-timeout = 30000\n");
    out.push_str("haproxy-protocol = false\n");
    out.push_str("tcp-fast-open = false\n");
    out.push_str("bungee-plugin-message-channel = true\n");
    out.push_str("show-ping-requests = false\n");
    out.push_str("failover-on-unexpected-server-disconnect = true\n");
    out.push_str("announce-proxy-commands = true\n");
    out.push_str("log-command-executions = false\n");
    out.push_str("log-player-connections = true\n");
    out.push_str("accepts-transfers = false\n");
    out.push_str("enable-reuse-port = false\n");
    out.push_str("command-rate-limit = 50\n");
    out.push_str("forward-commands-if-rate-limited = true\n");
    out.push_str("kick-after-rate-limited-commands = 0\n");
    out.push_str("tab-complete-rate-limit = 10\n");
    out.push_str("kick-after-rate-limited-tab-completes = 0\n\n");

    out.push_str("[query]\n");
    out.push_str("enabled = false\n");
    out.push_str("port = 25565\n");
    out.push_str("map = \"Velocity\"\n");
    out.push_str("show-plugins = false\n");

    out
}

fn write_start_scripts(settings: &InstallSettings) -> Result<(), Box<dyn Error>> {
    let sh_path = settings.install_dir.join("start.sh");
    let bat_path = settings.install_dir.join("start.bat");

    let sh_contents = format!(
        "#!/usr/bin/env sh\nset -e\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec java -Xms{} -Xmx{} -jar \"$DIR/velocity.jar\"\n",
        settings.xms, settings.xmx
    );
    fs::write(&sh_path, sh_contents)?;

    let bat_contents = format!(
        "@echo off\r\nset DIR=%~dp0\r\njava -Xms{} -Xmx{} -jar \"%DIR%velocity.jar\"\r\n",
        settings.xms, settings.xmx
    );
    fs::write(&bat_path, bat_contents)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&sh_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&sh_path, permissions)?;
    }

    Ok(())
}

fn prompt_with_default(message: &str, default: &str) -> io::Result<String> {
    let input = prompt(&format!("{message} [{default}]: "))?;
    if input.trim().is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

fn prompt_u32_with_default(message: &str, default: u32) -> io::Result<u32> {
    loop {
        let input = prompt(&format!("{message} [{default}]: "))?;
        if input.trim().is_empty() {
            return Ok(default);
        }
        match input.trim().parse::<u32>() {
            Ok(value) => return Ok(value),
            Err(_) => println!("数値を入力してください。"),
        }
    }
}

fn prompt_usize_with_default(
    message: &str,
    default: usize,
    range: std::ops::RangeInclusive<usize>,
) -> io::Result<usize> {
    loop {
        let input = prompt(&format!("{message} [{default}]: "))?;
        let value = if input.trim().is_empty() {
            default
        } else {
            match input.trim().parse::<usize>() {
                Ok(value) => value,
                Err(_) => {
                    println!("数値を入力してください。");
                    continue;
                }
            }
        };
        if range.contains(&value) {
            return Ok(value);
        }
        println!("範囲内の番号を入力してください。");
    }
}

fn prompt_yes_no(message: &str, default: bool) -> io::Result<bool> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    loop {
        let input = prompt(&format!("{message} {suffix}: "))?;
        let trimmed = input.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Ok(default);
        }
        match trimmed.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("y または n を入力してください。"),
        }
    }
}

fn prompt_non_empty(message: &str) -> io::Result<String> {
    loop {
        let input = prompt(&format!("{message}: "))?;
        if input.trim().is_empty() {
            println!("空の値は指定できません。");
            continue;
        }
        return Ok(input);
    }
}

fn prompt(message: &str) -> io::Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn escape_toml(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
