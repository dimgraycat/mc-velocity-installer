use std::process::Command;

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
fn help_shows_usage() {
    let output = Command::new(bin_path())
        .arg("--help")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("使い方"));
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
