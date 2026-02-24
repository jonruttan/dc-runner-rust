use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecSourceMode {
    Bundled,
    Workspace,
    Auto,
}

impl SpecSourceMode {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "bundled" => Some(Self::Bundled),
            "workspace" => Some(Self::Workspace),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

mod embedded_data_contracts {
    include!(concat!(env!("OUT_DIR"), "/embedded_data_contracts.rs"));
}

fn bundled_index() -> &'static HashMap<&'static str, &'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| embedded_data_contracts::FILES.iter().copied().collect())
}

pub fn bundled_snapshot_sha256() -> &'static str {
    let _ = embedded_data_contracts::SNAPSHOT_SOURCE_ROOT;
    embedded_data_contracts::SNAPSHOT_SHA256
}

pub fn effective_mode() -> Result<SpecSourceMode, String> {
    match std::env::var("DC_RUNNER_SPEC_SOURCE") {
        Ok(raw) => SpecSourceMode::parse(&raw).ok_or_else(|| {
            format!(
                "invalid DC_RUNNER_SPEC_SOURCE value `{raw}` (expected: bundled|workspace|auto)"
            )
        }),
        Err(_) => {
            if cfg!(test) {
                Ok(SpecSourceMode::Auto)
            } else {
                Ok(SpecSourceMode::Bundled)
            }
        }
    }
}

fn normalize_repo_path(raw: &str) -> String {
    raw.trim().trim_start_matches('/').replace('\\', "/")
}

fn bundled_candidate_paths(raw: &str) -> Vec<String> {
    let normalized = normalize_repo_path(raw);
    if normalized.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::<String>::new();
    if normalized.starts_with("specs/upstream/data-contracts/") {
        out.push(normalized.clone());
    }
    if normalized.starts_with("specs/") {
        out.push(format!("specs/upstream/data-contracts/{normalized}"));
    }
    out.push(normalized);
    out.sort();
    out.dedup();
    out
}

fn read_bundled(raw: &str) -> Result<Option<String>, String> {
    let index = bundled_index();
    for candidate in bundled_candidate_paths(raw) {
        if let Some(text) = index.get(candidate.as_str()) {
            return Ok(Some((*text).to_string()));
        }
    }
    Ok(None)
}

fn read_workspace(root: &Path, raw: &str) -> Result<Option<String>, String> {
    let path = root.join(normalize_repo_path(raw));
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("failed reading workspace spec {}: {e}", path.display()))?;
    Ok(Some(text))
}

pub fn read_spec_text(root: &Path, raw: &str) -> Result<String, String> {
    let mode = effective_mode()?;
    match mode {
        SpecSourceMode::Bundled => match read_bundled(raw)? {
            Some(text) => Ok(text),
            None => Err(format!(
                "spec not found in bundled snapshot: {raw} (snapshot_sha256={})",
                bundled_snapshot_sha256()
            )),
        },
        SpecSourceMode::Workspace => match read_workspace(root, raw)? {
            Some(text) => Ok(text),
            None => Err(format!(
                "spec not found in workspace: {} (mode=workspace; hint: use --spec-source bundled|auto)",
                root.join(normalize_repo_path(raw)).display()
            )),
        },
        SpecSourceMode::Auto => {
            if let Some(text) = read_workspace(root, raw)? {
                return Ok(text);
            }
            match read_bundled(raw)? {
                Some(text) => Ok(text),
                None => Err(format!(
                    "spec not found in workspace or bundled snapshot: {raw} (snapshot_sha256={})",
                    bundled_snapshot_sha256()
                )),
            }
        }
    }
}

pub fn spec_exists(root: &Path, raw: &str) -> Result<bool, String> {
    let mode = effective_mode()?;
    match mode {
        SpecSourceMode::Bundled => Ok(read_bundled(raw)?.is_some()),
        SpecSourceMode::Workspace => Ok(root.join(normalize_repo_path(raw)).exists()),
        SpecSourceMode::Auto => {
            let workspace_exists = root.join(normalize_repo_path(raw)).exists();
            if workspace_exists {
                return Ok(true);
            }
            Ok(read_bundled(raw)?.is_some())
        }
    }
}
