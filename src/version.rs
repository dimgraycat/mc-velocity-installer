use std::collections::BTreeMap;
use std::error::Error;

use reqwest::blocking::Client;
use serde::Deserialize;

pub const VERSION_INDEX_URL: &str = "https://minedeck.github.io/jars/velocity.json";

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
pub struct VersionInfo {
    pub version: String,
    pub kind: String,
    pub url: String,
    pub sha256: String,
}

pub fn fetch_versions(client: &Client, url: &str) -> Result<Vec<VersionInfo>, Box<dyn Error>> {
    let text = client.get(url).send()?.error_for_status()?.text()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;

    #[test]
    fn fetch_versions_rejects_status_error() {
        let server = MockServer::start();
        let body = r#"{
  "status": "error",
  "data": {}
}"#;
        server.mock(|when, then| {
            when.method(GET).path("/velocity.json");
            then.status(200).body(body);
        });

        let client = Client::builder().build().expect("client");
        let result = fetch_versions(&client, &server.url("/velocity.json"));
        assert!(result.is_err());
        let message = result.err().expect("error").to_string();
        assert!(message.contains("status=error"));
    }

    #[test]
    fn fetch_versions_requires_sha256() {
        let server = MockServer::start();
        let body = r#"{
  "status": "ok",
  "data": {
    "1.2.3": {
      "url": "http://example.invalid/velocity.jar",
      "checksum": {
        "sha1": null,
        "sha256": null
      },
      "type": "stable"
    }
  }
}"#;
        server.mock(|when, then| {
            when.method(GET).path("/velocity.json");
            then.status(200).body(body);
        });

        let client = Client::builder().build().expect("client");
        let result = fetch_versions(&client, &server.url("/velocity.json"));
        assert!(result.is_err());
        let message = result.err().expect("error").to_string();
        assert!(message.contains("sha256"));
    }

    #[test]
    fn fetch_versions_rejects_empty_list() {
        let server = MockServer::start();
        let body = r#"{
  "status": "ok",
  "data": {}
}"#;
        server.mock(|when, then| {
            when.method(GET).path("/velocity.json");
            then.status(200).body(body);
        });

        let client = Client::builder().build().expect("client");
        let result = fetch_versions(&client, &server.url("/velocity.json"));
        assert!(result.is_err());
        let message = result.err().expect("error").to_string();
        assert!(message.contains("バージョン一覧が空です"));
    }
}
