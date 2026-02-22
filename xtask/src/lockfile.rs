use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockV1 {
    pub version: u32,
    pub upstream: Upstream,
    pub snapshot: Snapshot,
    pub integrity: Integrity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upstream {
    pub repo: String,
    pub tag: String,
    pub commit: String,
    pub synced_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub root: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Integrity {
    pub file_count: u64,
    pub sha256_manifest: String,
}

impl Snapshot {
    pub fn curated(root: &str, include: &[&str], exclude: &[&str]) -> Self {
        Self {
            root: root.to_string(),
            include: include.iter().map(|v| (*v).to_string()).collect(),
            exclude: exclude.iter().map(|v| (*v).to_string()).collect(),
        }
    }
}

pub fn load_lock(lock_file: &Path) -> Result<LockV1> {
    let txt = fs::read_to_string(lock_file)
        .with_context(|| format!("failed reading {}", lock_file.display()))?;
    let lock: LockV1 = serde_yaml::from_str(&txt).context("failed parsing lock YAML")?;
    validate_lock(&lock, None)?;
    Ok(lock)
}

pub fn write_lock(lock_file: &Path, lock: &LockV1) -> Result<()> {
    if let Some(parent) = lock_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }
    validate_lock(lock, None)?;
    let txt = serde_yaml::to_string(lock).context("failed serializing lock YAML")?;
    fs::write(lock_file, txt).with_context(|| format!("failed writing {}", lock_file.display()))
}

pub fn validate_lock(lock: &LockV1, expected_root: Option<&str>) -> Result<()> {
    if lock.version != 1 {
        bail!("lock version must be 1");
    }
    if lock.upstream.commit.len() != 40
        || !lock.upstream.commit.chars().all(|c| c.is_ascii_hexdigit())
    {
        bail!("lock upstream.commit must be 40-char sha");
    }
    if let Some(root) = expected_root {
        if lock.snapshot.root != root {
            bail!("lock snapshot.root unexpected: {}", lock.snapshot.root);
        }
    } else if !lock.snapshot.root.starts_with("specs/upstream/") {
        bail!("lock snapshot.root must be under specs/upstream/");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_unknown_fields_in_lock_yaml() {
        let yaml = r#"
version: 1
upstream:
  repo: https://github.com/jonruttan/data-contracts.git
  tag: v0.1.0
  commit: 0000000000000000000000000000000000000000
  synced_at_utc: "2026-01-01T00:00:00Z"
snapshot:
  root: specs/upstream/data-contracts
  include: [specs/index.md]
  exclude: []
  extra: nope
integrity:
  file_count: 1
  sha256_manifest: deadbeef
"#;
        let parsed = serde_yaml::from_str::<LockV1>(yaml);
        assert!(parsed.is_err());
    }
}
