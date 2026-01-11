use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const VERSION_INDEX_URL: &str = "https://minedeck.github.io/jars/velocity.json";
const DEFAULT_XMS: &str = "256M";
const DEFAULT_XMX: &str = "512M";

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

#[derive(Debug)]
struct InstallSettings {
    install_dir: PathBuf,
    version: VersionInfo,
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

    let (xms, xmx) = prompt_memory()?;

    let settings = InstallSettings {
        install_dir,
        version,
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
        "初回起動で {} が生成されます。",
        settings.install_dir.join("velocity.toml").display()
    );

    Ok(())
}

fn prompt_install_dir() -> io::Result<PathBuf> {
    let default_dir = default_install_dir();
    let default_display = default_dir.to_string_lossy();
    loop {
        let input = prompt_with_default("インストール先ディレクトリ", &default_display)?;
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

fn default_install_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("velocity")
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
    println!("- 起動メモリ: Xms={} / Xmx={}", settings.xms, settings.xmx);
    println!("- 設定ファイルは初回起動時に生成されます");
}

fn perform_install(client: &Client, settings: &InstallSettings) -> Result<(), Box<dyn Error>> {
    if !settings.install_dir.exists() {
        fs::create_dir_all(&settings.install_dir)?;
    }

    let jar_path = settings.install_dir.join("velocity.jar");
    println!("ダウンロード中: {}", settings.version.url);
    download_with_sha256(client, &settings.version, &jar_path)?;

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

fn prompt(message: &str) -> io::Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
