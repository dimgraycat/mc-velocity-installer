use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use reqwest::{Url, blocking::Client};
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
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        print_version();
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--setup") {
        run_setup()?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--redownload-jar") {
        run_redownload_jar()?;
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--update") {
        println!("アップデート機能は未対応です。新規インストールのみ対応しています。");
        return Ok(());
    }

    println!("{} (新規インストール)", binary_name());
    println!("Java はインストール済みであることを前提に進めます。");
    println!();

    let install_dir = prompt_install_dir()?;
    if !confirm_existing_install(&install_dir)? {
        println!("中断しました。");
        return Ok(());
    }

    let client = build_client()?;

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

fn print_help() {
    let name = binary_name();
    println!(
        "{name} {}\n\n使い方:\n  {name} [OPTIONS]\n\nOPTIONS:\n  --setup            velocity.toml を対話式に編集します\n  --redownload-jar   jar を再取得します（必要ならスクリプト置き換え）\n  --update           未対応（新規インストールのみ対応）\n  -h, --help         ヘルプを表示します\n  -V, --version      バージョンを表示します\n\n詳細・更新情報:\n  ドキュメントや最新の変更点は以下で確認できます。\n  https://github.com/dimgraycat/mc-velocity-installer\n",
        build_version(),
    );
}

fn print_version() {
    println!("{} {}", binary_name(), build_version());
}

fn binary_name() -> String {
    std::env::args()
        .next()
        .and_then(|arg0| {
            Path::new(&arg0)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "mc-velocity-installer".to_string())
}

fn build_version() -> String {
    option_env!("MC_VELOCITY_BUILD_VERSION")
        .filter(|version| !version.trim().is_empty())
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string()
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

fn print_redownload_summary(install_dir: &Path, version: &VersionInfo, jar_name: &str) {
    println!();
    println!("設定サマリ:");
    println!("- インストール先: {}", install_dir.display());
    println!("- バージョン: {} ({})", version.version, version.kind);
    println!("- 再取得する jar: {jar_name}");
    println!("- 既存スクリプトの置き換え可否は後で確認します");
}

fn perform_install(client: &Client, settings: &InstallSettings) -> Result<(), Box<dyn Error>> {
    if !settings.install_dir.exists() {
        fs::create_dir_all(&settings.install_dir)?;
    }

    let jar_name = jar_filename_from_url(&settings.version.url, &settings.version.version);
    let jar_path = settings.install_dir.join(&jar_name);
    println!("ダウンロード中: {}", settings.version.url);
    download_with_sha256(client, &settings.version, &jar_path)?;

    write_start_scripts(
        &settings.install_dir,
        &settings.xms,
        &settings.xmx,
        &jar_name,
    )?;
    write_systemd_service(settings)?;
    Ok(())
}

fn run_redownload_jar() -> Result<(), Box<dyn Error>> {
    println!("{} (jar再取得)", binary_name());
    println!("Java はインストール済みであることを前提に進めます。");
    println!();

    let install_dir = prompt_install_dir()?;
    if !confirm_existing_install(&install_dir)? {
        println!("中断しました。");
        return Ok(());
    }

    let client = build_client()?;
    println!("Velocity のバージョン一覧を取得しています...");
    let index_url =
        std::env::var("MC_VELOCITY_INDEX_URL").unwrap_or_else(|_| VERSION_INDEX_URL.to_string());
    let versions = fetch_versions(&client, &index_url)?;
    let version = prompt_version(&versions)?;

    let jar_name = jar_filename_from_url(&version.url, &version.version);
    print_redownload_summary(&install_dir, &version, &jar_name);
    if !prompt_yes_no("この内容で再取得しますか？", true)? {
        println!("中断しました。");
        return Ok(());
    }

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir)?;
    }

    let jar_path = install_dir.join(&jar_name);
    println!("ダウンロード中: {}", version.url);
    download_with_sha256(&client, &version, &jar_path)?;
    let replace_scripts = prompt_yes_no("start.sh / start.bat を置き換えますか？", false)?;
    if replace_scripts {
        let (xms, xmx) = match detect_existing_memory(&install_dir)? {
            Some(values) => values,
            None => prompt_memory()?,
        };
        write_start_scripts(&install_dir, &xms, &xmx, &jar_name)?;
        println!("start.sh / start.bat を更新しました。");
    }
    println!();
    println!("完了しました。");
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

fn build_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder()
        .user_agent("mc-velocity-installer")
        .build()?)
}

fn write_start_scripts(
    install_dir: &Path,
    xms: &str,
    xmx: &str,
    jar_name: &str,
) -> Result<(), Box<dyn Error>> {
    let sh_path = install_dir.join("start.sh");
    let bat_path = install_dir.join("start.bat");

    let sh_contents = format!(
        "#!/usr/bin/env sh\nset -e\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\ncd \"$DIR\"\nexec java -Xms{} -Xmx{} -jar \"{}\"\n",
        xms, xmx, jar_name
    );
    fs::write(&sh_path, sh_contents)?;

    let bat_contents = format!(
        "@echo off\r\nset \"DIR=%~dp0\"\r\ncd /d \"%DIR%\"\r\njava -Xms{} -Xmx{} -jar \"{}\"\r\n",
        xms, xmx, jar_name
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

fn jar_filename_from_url(url: &str, version: &str) -> String {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(name) = parsed.path_segments().and_then(|segments| segments.last()) {
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    format!("velocity-{}.jar", version)
}

fn detect_existing_memory(install_dir: &Path) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let sh_path = install_dir.join("start.sh");
    if let Some(values) = read_memory_from_script(&sh_path)? {
        return Ok(Some(values));
    }
    let bat_path = install_dir.join("start.bat");
    if let Some(values) = read_memory_from_script(&bat_path)? {
        return Ok(Some(values));
    }
    Ok(None)
}

fn read_memory_from_script(path: &Path) -> Result<Option<(String, String)>, Box<dyn Error>> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
    Ok(extract_memory_flags(&contents))
}

fn extract_memory_flags(contents: &str) -> Option<(String, String)> {
    let mut xms: Option<String> = None;
    let mut xmx: Option<String> = None;
    let mut iter = contents.split_whitespace().peekable();
    while let Some(token) = iter.next() {
        if token == "-Xms" {
            if let Some(value) = iter.next() {
                xms = Some(value.trim_matches('"').to_string());
            }
            continue;
        }
        if token == "-Xmx" {
            if let Some(value) = iter.next() {
                xmx = Some(value.trim_matches('"').to_string());
            }
            continue;
        }
        if let Some(value) = token.strip_prefix("-Xms") {
            if !value.is_empty() {
                xms = Some(value.trim_matches('"').to_string());
            }
            continue;
        }
        if let Some(value) = token.strip_prefix("-Xmx") {
            if !value.is_empty() {
                xmx = Some(value.trim_matches('"').to_string());
            }
        }
    }
    match (xms, xmx) {
        (Some(xms), Some(xmx)) => Some((xms, xmx)),
        _ => None,
    }
}

fn write_systemd_service(settings: &InstallSettings) -> Result<(), Box<dyn Error>> {
    let service_path = settings.install_dir.join("velocity.service");
    let install_dir = absolute_path(&settings.install_dir)?;

    let exec_start = install_dir.join("start.sh");
    let user = std::env::var("USER").unwrap_or_else(|_| "velocity".to_string());
    let group = std::env::var("USER").unwrap_or_else(|_| "velocity".to_string());

    let contents = format!(
        "[Unit]\nDescription=Velocity Minecraft Proxy\nAfter=network.target\nStartLimitIntervalSec=600\nStartLimitBurst=6\n\n[Service]\nType=simple\nWorkingDirectory={}\nExecStart={}\nRestart=on-failure\nRestartSec=5s\nUser={}\nGroup={}\n\n[Install]\nWantedBy=multi-user.target\n",
        install_dir.display(),
        exec_start.display(),
        user,
        group
    );
    fs::write(service_path, contents)?;
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
