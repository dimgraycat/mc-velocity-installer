use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_mc-velocity-installer")
}

#[test]
fn update_flag_shows_message() {
    let output = Command::new(bin_path())
        .arg("--update")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("未対応"));
}

#[test]
fn setup_requires_existing_config() {
    let mut temp = std::env::temp_dir();
    let unique = format!(
        "mc-velocity-installer-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    );
    temp.push(unique);
    fs::create_dir_all(&temp).expect("create temp dir");

    let mut child = Command::new(bin_path())
        .arg("--setup")
        .current_dir(&temp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn binary");

    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"\n")
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("設定ファイルが見つかりません"));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn help_shows_usage() {
    let output = Command::new(bin_path())
        .arg("--help")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("使い方"));
    assert!(stdout.contains("--setup"));
    assert!(stdout.contains("--redownload-jar"));
}

#[test]
fn version_shows_version() {
    let output = Command::new(bin_path())
        .arg("--version")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}
