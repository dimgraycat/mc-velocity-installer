use std::collections::BTreeMap;
use std::error::Error;

use reqwest::blocking::Client;
use serde::Deserialize;

const VERSION_INDEX_URL: &str = "https://minedeck.github.io/jars/velocity.json";

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

pub fn fetch_versions(client: &Client) -> Result<Vec<VersionInfo>, Box<dyn Error>> {
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
