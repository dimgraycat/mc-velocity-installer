use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

mod prompts;
mod setup;
mod version;

use prompts::{
    confirm_existing_install, prompt_install_dir, prompt_memory, prompt_version, prompt_yes_no,
};
use setup::run_setup;
use version::{VERSION_INDEX_URL, VersionInfo, fetch_versions};

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
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--setup") {
        run_setup()?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--update") {
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
    let index_url =
        std::env::var("MC_VELOCITY_INDEX_URL").unwrap_or_else(|_| VERSION_INDEX_URL.to_string());
    let versions = fetch_versions(&client, &index_url)?;
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
        "#!/usr/bin/env sh\nset -e\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\ncd \"$DIR\"\nexec java -Xms{} -Xmx{} -jar \"velocity.jar\"\n",
        settings.xms, settings.xmx
    );
    fs::write(&sh_path, sh_contents)?;

    let bat_contents = format!(
        "@echo off\r\nset \"DIR=%~dp0\"\r\ncd /d \"%DIR%\"\r\njava -Xms{} -Xmx{} -jar \"velocity.jar\"\r\n",
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
