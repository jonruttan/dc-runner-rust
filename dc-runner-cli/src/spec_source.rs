use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSpecBundle {
    pub version: String,
    pub installed_at: i64,
    pub source_ref: String,
    pub checksum: Option<String>,
    pub verified: bool,
    pub checksum_checked_at: Option<i64>,
    pub published_at: Option<String>,
    pub signature_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecSourceState {
    pub version: u8,
    pub active: String,
    pub installed: Vec<CachedSpecBundle>,
    pub last_refresh_at: Option<i64>,
    pub last_check_at: Option<i64>,
    pub last_error: Option<String>,
}

impl Default for SpecSourceState {
    fn default() -> Self {
        Self {
            version: 1,
            active: "bundled".to_string(),
            installed: Vec::new(),
            last_refresh_at: None,
            last_check_at: None,
            last_error: None,
        }
    }
}

impl SpecSourceState {
    fn set_active_version(&mut self, version: Option<&str>) {
        self.active = match version {
            Some(v) => format!("cache:{v}"),
            None => "bundled".to_string(),
        };
    }
    fn active_version(&self) -> Option<&str> {
        parse_cache_active_version(&self.active)
    }
    fn find_version(&self, version: &str) -> Option<CachedSpecBundle> {
        self.installed
            .iter()
            .find(|entry| entry.version == version)
            .cloned()
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeSpecSource {
    Bundled,
    Workspace,
    Auto,
    Cache { version: String, path: PathBuf },
}

mod embedded_data_contracts {
    include!(concat!(env!("OUT_DIR"), "/embedded_data_contracts.rs"));
}
#[cfg(feature = "bundler")]
mod embedded_data_contracts_library {
    include!(concat!(
        env!("OUT_DIR"),
        "/embedded_data_contracts_library.rs"
    ));
}

fn bundled_index_core() -> &'static HashMap<&'static str, &'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| embedded_data_contracts::FILES.iter().copied().collect())
}

#[cfg(feature = "bundler")]
fn bundled_index_bundler() -> &'static HashMap<&'static str, &'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let _ = embedded_data_contracts_library::SNAPSHOT_SOURCE_ROOT;
    let _ = embedded_data_contracts_library::SNAPSHOT_SHA256;
    INDEX.get_or_init(|| {
        embedded_data_contracts_library::FILES
            .iter()
            .copied()
            .collect()
    })
}

fn to_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn user_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn default_cache_root() -> PathBuf {
    if let Ok(value) = std::env::var("DC_RUNNER_SPEC_CACHE_DIR") {
        return PathBuf::from(value);
    }
    if let Ok(value) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(value).join("dc-runner").join("specs");
    }
    user_home().join(".cache").join("dc-runner").join("specs")
}

fn state_path() -> PathBuf {
    default_cache_root().join("state.yaml")
}

fn bundle_root(version: &str) -> PathBuf {
    default_cache_root().join("bundles").join(version)
}

fn parse_cache_active_version(raw: &str) -> Option<&str> {
    raw.strip_prefix("cache:")
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

pub fn load_state() -> Result<SpecSourceState, String> {
    let path = state_path();
    if !path.exists() {
        return Ok(SpecSourceState::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("failed reading spec state {}: {e}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(SpecSourceState::default());
    }
    serde_yaml::from_str::<SpecSourceState>(&raw)
        .or_else(|e| {
            Err(format!(
                "failed parsing spec state {}: {e}",
                path.display()
            ))
        })
}

pub fn save_state(state: &SpecSourceState) -> Result<(), String> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create spec state dir {}: {e}", parent.display()))?;
    }
    let raw = serde_yaml::to_string(state)
        .map_err(|e| format!("failed to serialize spec state: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("failed writing spec state {}: {e}", path.display()))
}

fn normalize_active_cache_target(state: &SpecSourceState) -> Option<RuntimeSpecSource> {
    match state.active.as_str() {
        "bundled" => Some(RuntimeSpecSource::Bundled),
        "workspace" => Some(RuntimeSpecSource::Workspace),
        raw => {
            let version = parse_cache_active_version(raw)?;
            let entry = state.find_version(version)?;
            if !entry.verified {
                return None;
            }
            let path = bundle_root(version);
            if !path.exists() {
                return None;
            }
            Some(RuntimeSpecSource::Cache {
                version: version.to_string(),
                path,
            })
        }
    }
}

pub fn resolve_runtime_mode(_root: &Path) -> Result<RuntimeSpecSource, String> {
    let cli_mode = effective_mode()?;
    if matches!(cli_mode, SpecSourceMode::Bundled | SpecSourceMode::Workspace) {
        return Ok(match cli_mode {
            SpecSourceMode::Bundled => RuntimeSpecSource::Bundled,
            SpecSourceMode::Workspace => RuntimeSpecSource::Workspace,
            SpecSourceMode::Auto => RuntimeSpecSource::Auto,
        });
    }

    let state = match load_state() {
        Ok(v) => v,
        Err(_) => {
            return Ok(match cli_mode {
                SpecSourceMode::Bundled => RuntimeSpecSource::Bundled,
                SpecSourceMode::Workspace => RuntimeSpecSource::Workspace,
                SpecSourceMode::Auto => RuntimeSpecSource::Auto,
            })
        }
    };

    if let Some(active) = normalize_active_cache_target(&state) {
        return Ok(active);
    }

    if let SpecSourceMode::Auto = cli_mode {
        let active_cache = state
            .active_version()
            .and_then(|version| state.find_version(version))
            .filter(|entry| entry.verified)
            .and_then(|entry| {
                let path = bundle_root(&entry.version);
                if path.exists() {
                    Some(RuntimeSpecSource::Cache {
                        version: entry.version,
                        path,
                    })
                } else {
                    None
                }
            });
        if let Some(active_cache) = active_cache {
            return Ok(active_cache);
        }
    }

    Ok(match cli_mode {
        SpecSourceMode::Bundled => RuntimeSpecSource::Bundled,
        SpecSourceMode::Workspace => RuntimeSpecSource::Workspace,
        SpecSourceMode::Auto => RuntimeSpecSource::Auto,
    })
}

pub fn specs_root(root: &Path) -> Result<PathBuf, String> {
    Ok(match resolve_runtime_mode(root)? {
        RuntimeSpecSource::Cache { path, .. } => path,
        _ => root.to_path_buf(),
    })
}

pub fn upsert_bundle_entry(
    state: &mut SpecSourceState,
    version: String,
    source_ref: String,
    checksum: Option<String>,
    verified: bool,
    published_at: Option<String>,
) {
    let installed_at = to_timestamp();
    state.installed.retain(|entry| entry.version != version);
    state.installed.push(CachedSpecBundle {
        version,
        installed_at,
        source_ref,
        checksum,
        verified,
        checksum_checked_at: Some(installed_at),
        published_at,
        signature_available: false,
    });
}

pub fn use_cache_version(state: &mut SpecSourceState, version: Option<&str>) -> Result<(), String> {
    if let Some(version) = version {
        let entry = state.find_version(version).ok_or_else(|| {
            format!("cache version {version} is not installed")
        })?;
        if !entry.verified {
            return Err(format!("cache version {version} is not verified"));
        }
    }
    state.set_active_version(version);
    state.last_error = None;
    Ok(())
}

pub fn rollback_version(
    state: &mut SpecSourceState,
    target: Option<&str>,
) -> Result<String, String> {
    if target.is_some() {
        let version = target.unwrap_or("");
        let entry = state
            .installed
            .iter()
            .find(|entry| entry.version == version)
            .ok_or_else(|| format!("cache version {version} not installed"))?;
        if !entry.verified {
            return Err(format!("cache version {version} is not verified"));
        }
        state.set_active_version(Some(version));
        state.last_error = None;
        return Ok(version.to_string());
    }

    let active = state.active_version().map(|v| v.to_string());
    let mut ordered = state.installed.clone();
    ordered.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
    if let Some(active_version) = active {
        if let Some(entry) = ordered.iter().find(|entry| entry.version != active_version) {
            state.set_active_version(Some(&entry.version));
            state.last_error = None;
            return Ok(entry.version.clone());
        }
    } else if let Some(entry) = ordered.first() {
        state.set_active_version(Some(&entry.version));
        state.last_error = None;
        return Ok(entry.version.clone());
    }

    state.set_active_version(None);
    state.last_error = None;
    Ok("bundled".to_string())
}

pub fn mark_refresh(state: &mut SpecSourceState) {
    state.last_refresh_at = Some(to_timestamp());
    state.last_error = None;
}

pub fn mark_check(state: &mut SpecSourceState, ok: bool) {
    state.last_check_at = Some(to_timestamp());
    state.last_error = if ok {
        None
    } else {
        Some("verification_failed".to_string())
    };
}


pub fn bundle_path_for_version(version: &str) -> PathBuf {
    bundle_root(version)
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
    for candidate in bundled_candidate_paths(raw) {
        if let Some(text) = bundled_index_core().get(candidate.as_str()) {
            return Ok(Some((*text).to_string()));
        }
        #[cfg(feature = "bundler")]
        if let Some(text) = bundled_index_bundler().get(candidate.as_str()) {
            return Ok(Some((*text).to_string()));
        }
    }
    Ok(None)
}

fn read_workspace(root: &Path, raw: &str) -> Result<Option<String>, String> {
    let normalized = normalize_repo_path(raw);
    let mut candidates = vec![root.join(&normalized)];
    if let Some(stripped) = normalized.strip_prefix("specs/upstream/data-contracts/") {
        candidates.push(root.join(stripped));
    }
    if let Some(stripped) = normalized.strip_prefix("specs/upstream/data-contracts-library/") {
        candidates.push(root.join(stripped));
    }

    let Some(path) = candidates.into_iter().find(|p| p.exists()) else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("failed reading workspace spec {}: {e}", path.display()))?;
    Ok(Some(text))
}

fn workspace_path_for_error(root: &Path, raw: &str) -> String {
    let normalized = normalize_repo_path(raw);
    if let Some(stripped) = normalized.strip_prefix("specs/upstream/data-contracts/") {
        return root.join(stripped).display().to_string();
    }
    if let Some(stripped) = normalized.strip_prefix("specs/upstream/data-contracts-library/") {
        return root.join(stripped).display().to_string();
    }
    root.join(normalized).display().to_string()
}

fn read_cache(root: &Path, raw: &str) -> Result<Option<String>, String> {
    read_workspace(root, raw)
}

pub fn read_spec_text(root: &Path, raw: &str) -> Result<String, String> {
    let mode = resolve_runtime_mode(root)?;
    match mode {
        RuntimeSpecSource::Bundled => match read_bundled(raw)? {
            Some(text) => Ok(text),
            None => Err(format!(
                "spec not found in bundled snapshot: {raw} (snapshot_sha256={})",
                bundled_snapshot_sha256()
            )),
        },
        RuntimeSpecSource::Workspace => match read_workspace(root, raw)? {
            Some(text) => Ok(text),
            None => Err(format!(
                "spec not found in workspace: {} (mode=workspace; hint: use --spec-source bundled|auto)",
                workspace_path_for_error(root, raw)
            )),
        },
        RuntimeSpecSource::Auto => {
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
        RuntimeSpecSource::Cache { path, .. } => match read_cache(&path, raw)? {
            Some(text) => Ok(text),
            None => Err(format!(
                "spec not found in active cached source: {raw}"
            )),
        },
    }
}

pub fn spec_exists(root: &Path, raw: &str) -> Result<bool, String> {
    let mode = resolve_runtime_mode(root)?;
    match mode {
        RuntimeSpecSource::Bundled => Ok(read_bundled(raw)?.is_some()),
        RuntimeSpecSource::Workspace => read_workspace(root, raw).map(|v| v.is_some()),
        RuntimeSpecSource::Auto => {
            let workspace_exists = root.join(normalize_repo_path(raw)).exists();
            if workspace_exists {
                return Ok(true);
            }
            Ok(read_bundled(raw)?.is_some())
        }
        RuntimeSpecSource::Cache { path, .. } => {
            Ok(path.join(normalize_repo_path(raw)).exists())
        }
    }
}
