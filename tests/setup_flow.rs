use std::io::Write;
use std::process::{Command, Stdio};

use tempfile::TempDir;
use toml_edit::DocumentMut;

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_mc-velocity-installer")
}

const BASE_CONFIG: &str = r#"
bind = "0.0.0.0:25565"
motd = "<#09add3>A Velocity Server"
show-max-players = 100
online-mode = true
force-key-authentication = true
player-info-forwarding-mode = "NONE"
forwarding-secret-file = "forwarding.secret"

[servers]
lobby = "127.0.0.1:30066"
pvp = "127.0.0.1:30067"
try = ["lobby", "pvp"]

[forced-hosts]
"lobby.example.com" = ["lobby"]
"pvp.example.com" = ["pvp"]
"#;

const SERVER_EDIT_CONFIG: &str = r#"
bind = "0.0.0.0:25565"
motd = "<#09add3>A Velocity Server"
show-max-players = 100
online-mode = true
force-key-authentication = true
player-info-forwarding-mode = "NONE"
forwarding-secret-file = "forwarding.secret"

[servers]
lobby = "127.0.0.1:30066"
try = ["lobby"]

[forced-hosts]
"lobby.example.com" = ["lobby"]
"#;

#[test]
fn setup_flow_updates_selected_fields() {
    let temp_dir = TempDir::new().expect("temp dir");
    let velocity_dir = temp_dir.path().join("velocity");
    std::fs::create_dir_all(&velocity_dir).expect("create velocity dir");
    let config_path = velocity_dir.join("velocity.toml");

    std::fs::write(&config_path, BASE_CONFIG).expect("write config");

    let mut child = Command::new(bin_path())
        .arg("--setup")
        .current_dir(temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let inputs = [
        "",           // config path (default)
        "s",          // bind skip
        "e",          // motd edit
        "New MOTD",   // motd value
        "e",          // show-max-players edit
        "200",        // show-max-players value
        "e",          // online-mode edit
        "n",          // online-mode false
        "s",          // force-key-authentication skip
        "e",          // forwarding-mode edit
        "4",          // forwarding-mode select modern
        "e",          // forwarding-secret-file edit
        "secret.txt", // forwarding-secret-file value
        "e",          // servers edit
        "r",          // remove server
        "pvp",        // server name
        "f",          // finish server edit
        "e",          // try edit
        "lobby",      // try list
        "y",          // remove invalid forced-hosts references
        "y",          // remove host if empty
        "y",          // save
    ];
    let input_blob = inputs.join("\n") + "\n";
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input_blob.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let updated = std::fs::read_to_string(&config_path).expect("read config");
    let doc: DocumentMut = updated.parse().expect("parse updated toml");
    assert_eq!(
        doc.get("motd").and_then(|item| item.as_str()),
        Some("New MOTD")
    );
    assert_eq!(
        doc.get("show-max-players")
            .and_then(|item| item.as_integer()),
        Some(200)
    );
    assert_eq!(
        doc.get("online-mode").and_then(|item| item.as_bool()),
        Some(false)
    );
    assert_eq!(
        doc.get("player-info-forwarding-mode")
            .and_then(|item| item.as_str()),
        Some("MODERN")
    );
    assert_eq!(
        doc.get("forwarding-secret-file")
            .and_then(|item| item.as_str()),
        Some("secret.txt")
    );

    let servers = doc
        .get("servers")
        .and_then(|item| item.as_table())
        .expect("servers table");
    assert!(servers.get("lobby").is_some());
    assert!(servers.get("pvp").is_none());
    let try_list = servers
        .get("try")
        .and_then(|item| item.as_array())
        .expect("try list");
    assert_eq!(try_list.len(), 1);
    assert_eq!(
        try_list.get(0).and_then(|item| item.as_str()),
        Some("lobby")
    );

    let forced_hosts = doc
        .get("forced-hosts")
        .and_then(|item| item.as_table())
        .expect("forced-hosts");
    assert!(forced_hosts.get("pvp.example.com").is_none());
}

#[test]
fn setup_flow_no_changes_skips_save() {
    let temp_dir = TempDir::new().expect("temp dir");
    let velocity_dir = temp_dir.path().join("velocity");
    std::fs::create_dir_all(&velocity_dir).expect("create velocity dir");
    let config_path = velocity_dir.join("velocity.toml");
    std::fs::write(&config_path, BASE_CONFIG).expect("write config");

    let mut child = Command::new(bin_path())
        .arg("--setup")
        .current_dir(temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let inputs = ["", "s", "s", "s", "s", "s", "s", "s", "s", "s", "s"];
    let input_blob = inputs.join("\n") + "\n";
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input_blob.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("変更がないため保存しません。"));
    let updated = std::fs::read_to_string(&config_path).expect("read config");
    assert_eq!(updated, BASE_CONFIG);
}

#[test]
fn setup_flow_server_edit_with_invalid_inputs() {
    let temp_dir = TempDir::new().expect("temp dir");
    let velocity_dir = temp_dir.path().join("velocity");
    std::fs::create_dir_all(&velocity_dir).expect("create velocity dir");
    let config_path = velocity_dir.join("velocity.toml");
    std::fs::write(&config_path, SERVER_EDIT_CONFIG).expect("write config");

    let mut child = Command::new(bin_path())
        .arg("--setup")
        .current_dir(temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let inputs = [
        "",
        "x",
        "e",
        "0.0.0.0:25566",
        "s",
        "e",
        "abc",
        "300",
        "s",
        "e",
        "maybe",
        "n",
        "s",
        "s",
        "e",
        "x",
        "a",
        "",
        "a",
        "try",
        "a",
        "lobby",
        "a",
        "hub",
        "",
        "a",
        "hub",
        "127.0.0.1:30070",
        "e",
        "",
        "e",
        "missing",
        "e",
        "hub",
        "",
        "e",
        "hub",
        "127.0.0.1:30071",
        "r",
        "",
        "r",
        "missing",
        "r",
        "hub",
        "f",
        "s",
        "n",
    ];
    let input_blob = inputs.join("\n") + "\n";
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input_blob.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("保存を中断しました。"));
    let updated = std::fs::read_to_string(&config_path).expect("read config");
    assert_eq!(updated, SERVER_EDIT_CONFIG);
}
