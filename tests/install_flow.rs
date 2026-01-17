use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use httpmock::Method::GET;
use httpmock::MockServer;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_mc-velocity-installer")
}

fn assert_systemd_service(install_dir: &Path) {
    let service_path = install_dir.join("velocity.service");
    assert!(service_path.exists());

    let contents = std::fs::read_to_string(&service_path).expect("read service");
    let user = std::env::var("USER").unwrap_or_else(|_| "velocity".to_string());
    let group = std::env::var("USER").unwrap_or_else(|_| "velocity".to_string());
    assert!(contents.contains("Description=Velocity Minecraft Proxy"));
    assert!(contents.contains("StartLimitIntervalSec=600"));
    assert!(contents.contains("StartLimitBurst=6"));
    assert!(contents.contains(&format!("WorkingDirectory={}", install_dir.display())));
    assert!(contents.contains(&format!(
        "ExecStart={}",
        install_dir.join("start.sh").display()
    )));
    assert!(contents.contains(&format!("User={user}")));
    assert!(contents.contains(&format!("Group={group}")));
    assert!(!contents.contains("StandardOutput="));
    assert!(!contents.contains("StandardError="));
}

#[test]
fn install_flow_downloads_jar_and_writes_scripts() {
    let temp_dir = TempDir::new().expect("temp dir");
    let server = MockServer::start();

    let jar_bytes = b"velocity-jar";
    let sha256 = format!("{:x}", Sha256::digest(jar_bytes));
    let jar_name = "velocity-proxy-1.0.0.jar";
    let jar_path = format!("/{}", jar_name);
    server.mock(|when, then| {
        when.method(GET).path(jar_path.as_str());
        then.status(200).body(jar_bytes.as_slice());
    });

    let index_body = format!(
        r#"{{
  "status": "ok",
  "platform": "velocity",
  "type": "proxy",
  "data": {{
    "1.0.0": {{
      "url": "{}",
      "checksum": {{
        "sha1": null,
        "sha256": "{}"
      }},
      "build": 1,
      "type": "stable"
    }}
  }}
}}"#,
        server.url(jar_path.as_str()),
        sha256
    );
    server.mock(|when, then| {
        when.method(GET).path("/velocity.json");
        then.status(200).body(index_body);
    });

    let mut child = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .env("MC_VELOCITY_INDEX_URL", server.url("/velocity.json"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let input = "\n\n\n\n\n\n\n\n";
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let install_dir = temp_dir.path().join("velocity");
    let jar_file = install_dir.join(jar_name);
    assert!(jar_file.exists());
    assert!(install_dir.join("start.sh").exists());
    assert!(install_dir.join("start.bat").exists());

    let sh = std::fs::read_to_string(install_dir.join("start.sh")).expect("read start.sh");
    assert!(sh.contains(jar_name));
    let bat = std::fs::read_to_string(install_dir.join("start.bat")).expect("read start.bat");
    assert!(bat.contains(jar_name));
    assert_systemd_service(&install_dir);
}

#[test]
fn install_flow_retries_inputs_and_overwrites_existing_dir() {
    let temp_dir = TempDir::new().expect("temp dir");
    let velocity_dir = temp_dir.path().join("velocity");
    std::fs::create_dir_all(&velocity_dir).expect("create velocity dir");
    std::fs::write(velocity_dir.join("existing.txt"), "data").expect("write file");

    let server = MockServer::start();

    let jar_bytes = b"velocity-jar";
    let sha256 = format!("{:x}", Sha256::digest(jar_bytes));
    let jar_name = "velocity-proxy-1.0.0.jar";
    let jar_path = format!("/{}", jar_name);
    server.mock(|when, then| {
        when.method(GET).path(jar_path.as_str());
        then.status(200).body(jar_bytes.as_slice());
    });

    let index_body = format!(
        r#"{{
  "status": "ok",
  "platform": "velocity",
  "type": "proxy",
  "data": {{
    "1.0.0": {{
      "url": "{}",
      "checksum": {{
        "sha1": null,
        "sha256": "{}"
      }},
      "build": 1,
      "type": "stable"
    }}
  }}
}}"#,
        server.url(jar_path.as_str()),
        sha256
    );
    server.mock(|when, then| {
        when.method(GET).path("/velocity.json");
        then.status(200).body(index_body);
    });

    let mut child = Command::new(bin_path())
        .current_dir(temp_dir.path())
        .env("MC_VELOCITY_INDEX_URL", server.url("/velocity.json"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    let inputs = [
        "custom", "n", "", "y", "maybe", "y", "x", "2", "", "n", "", "y", "128M", "256M", "n", "",
        "", "y", "",
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

    assert!(velocity_dir.join(jar_name).exists());
    assert!(velocity_dir.join("start.sh").exists());
    assert!(velocity_dir.join("start.bat").exists());
    assert_systemd_service(&velocity_dir);
}
