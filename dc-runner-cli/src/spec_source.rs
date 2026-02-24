use std::cmp::Ordering;
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

#[derive(Debug, Clone, PartialEq)]
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
        .or_else(|e| Err(format!("failed parsing spec state {}: {e}", path.display())))
}

pub fn save_state(state: &SpecSourceState) -> Result<(), String> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create spec state dir {}: {e}", parent.display()))?;
    }
    let raw =
        serde_yaml::to_string(state).map_err(|e| format!("failed to serialize spec state: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("failed writing spec state {}: {e}", path.display()))
}

fn has_core_bundle_markers(bundle_root: &Path) -> bool {
    bundle_root
        .join("specs/04_governance/runner_entrypoints_v1.yaml")
        .exists()
        && bundle_root
            .join("specs/00_core/runner_version_contract_v1.yaml")
            .exists()
}

fn parse_version_parts(raw: &str) -> Option<Vec<u64>> {
    let mut parts = Vec::new();
    let mut seen = 0usize;
    for part in raw.split('.') {
        seen += 1;
        if seen > 3 {
            return None;
        }
        parts.push(part.parse::<u64>().ok()?);
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts)
}

fn compare_versions(a: &str, b: &str) -> Ordering {
    let (parsed_a, parsed_b) = (parse_version_parts(a), parse_version_parts(b));
    match (&parsed_a, &parsed_b) {
        (Some(a_parts), Some(b_parts)) => {
            let max_len = a_parts.len().max(b_parts.len());
            for idx in 0..max_len {
                let left = a_parts.get(idx).copied().unwrap_or(0);
                let right = b_parts.get(idx).copied().unwrap_or(0);
                match left.cmp(&right) {
                    Ordering::Equal => {}
                    non => return non,
                }
            }
            Ordering::Equal
        }
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => a.cmp(b),
    }
}

fn prefer_bundled_core_cache(state: &SpecSourceState) -> Option<CachedSpecBundle> {
    let mut candidates = state
        .installed
        .iter()
        .filter(|entry| entry.verified)
        .filter_map(|entry| {
            let path = bundle_root(&entry.version);
            if !path.exists() {
                return None;
            }
            if !has_core_bundle_markers(&path) {
                return None;
            }
            Some(entry.clone())
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|a, b| {
        let version_cmp = compare_versions(&a.version, &b.version);
        if version_cmp == Ordering::Equal {
            b.installed_at.cmp(&a.installed_at)
        } else {
            version_cmp
        }
    });
    candidates.pop()
}

fn resolve_bundled_runtime_mode() -> Result<(RuntimeSpecSource, String), String> {
    let mut state = load_state()?;
    let mut fallback_reason = String::new();

    if let Some(active_version) = state.active_version() {
        let active_path = bundle_root(active_version);
        if let Some(entry) = state.find_version(active_version) {
            if entry.verified && active_path.exists() && has_core_bundle_markers(&active_path) {
                return Ok((
                    RuntimeSpecSource::Cache {
                        version: active_version.to_string(),
                        path: active_path,
                    },
                    String::new(),
                ));
            }
            if !entry.verified {
                fallback_reason = format!("active cache version {active_version} is unverified");
            } else if !active_path.exists() {
                fallback_reason = format!("active cache version {active_version} is missing");
            } else {
                fallback_reason =
                    format!("active cache version {active_version} is not a valid core bundle");
            }
        }
    }

    if let Some(entry) = prefer_bundled_core_cache(&state) {
        if let Some(active_version) = state.active_version() {
            if active_version != entry.version {
                fallback_reason = format!(
                    "falling back to latest verified core cache: {}",
                    entry.version
                );
            } else {
                fallback_reason.clear();
            }
        }
        return Ok((
            RuntimeSpecSource::Cache {
                version: entry.version.clone(),
                path: bundle_root(&entry.version),
            },
            fallback_reason,
        ));
    }

    if fallback_reason.is_empty() {
        fallback_reason = "no verified core cache available".to_string();
    }
    fallback_reason.push_str(
        "; using bundled embedded snapshot. Run `dc-runner specs refresh --source remote --version latest --check-only` to repair.",
    );
    state.last_error = Some(fallback_reason.clone());
    let _ = save_state(&state);
    Ok((RuntimeSpecSource::Bundled, fallback_reason))
}

pub fn resolve_runtime_mode(_root: &Path) -> Result<RuntimeSpecSource, String> {
    let cli_mode = effective_mode()?;
    match cli_mode {
        SpecSourceMode::Bundled => {
            let (mode, reason) = resolve_bundled_runtime_mode()?;
            if let Ok(mut state) = load_state() {
                if mode == RuntimeSpecSource::Bundled {
                    state.last_error = Some(reason);
                } else {
                    if reason.is_empty() {
                        state.last_error = None;
                    } else {
                        state.last_error = Some(reason);
                    }
                }
                let _ = save_state(&state);
            }
            return Ok(mode);
        }
        SpecSourceMode::Workspace => return Ok(RuntimeSpecSource::Workspace),
        SpecSourceMode::Auto => {}
    }
    let state = match load_state() {
        Ok(v) => v,
        Err(_) => {
            return Ok(match cli_mode {
                SpecSourceMode::Auto => RuntimeSpecSource::Auto,
                SpecSourceMode::Bundled => RuntimeSpecSource::Bundled,
                SpecSourceMode::Workspace => RuntimeSpecSource::Workspace,
            })
        }
    };

    if let SpecSourceMode::Auto = cli_mode {
        if let Some(active_cache) = state.active_version().and_then(|version| {
            state
                .find_version(version)
                .filter(|entry| entry.verified)
                .and_then(|entry| {
                    let path = bundle_root(&entry.version);
                    if path.exists() {
                        Some(RuntimeSpecSource::Cache {
                            version: entry.version.clone(),
                            path,
                        })
                    } else {
                        None
                    }
                })
        }) {
            return Ok(active_cache);
        }
        if state.active == "workspace" {
            return Ok(RuntimeSpecSource::Workspace);
        }
        return Ok(RuntimeSpecSource::Auto);
    }

    if let Some(active) = state
        .active_version()
        .and_then(|version| state.find_version(version))
        .filter(|entry| entry.verified)
        .and_then(|entry| {
            let path = bundle_root(&entry.version);
            if path.exists() {
                Some(RuntimeSpecSource::Cache {
                    version: entry.version.clone(),
                    path,
                })
            } else {
                None
            }
        })
    {
        return Ok(active);
    }

    if let Some(fallback) = prefer_bundled_core_cache(&state) {
        return Ok(RuntimeSpecSource::Cache {
            version: fallback.version.clone(),
            path: bundle_root(&fallback.version),
        });
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
        let entry = state
            .find_version(version)
            .ok_or_else(|| format!("cache version {version} is not installed"))?;
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
            if let Ok(state) = load_state() {
                if let Some(entry) = prefer_bundled_core_cache(&state) {
                    let cache_root = bundle_root(&entry.version);
                    if let Some(text) = read_cache(&cache_root, raw)? {
                        return Ok(text);
                    }
                }
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
            if let Ok(state) = load_state() {
                if let Some(entry) = prefer_bundled_core_cache(&state) {
                    let cache_root = bundle_root(&entry.version);
                    if cache_root.join(normalize_repo_path(raw)).exists() {
                        return Ok(true);
                    }
                }
            }
            Ok(read_bundled(raw)?.is_some())
        }
        RuntimeSpecSource::Cache { path, .. } => Ok(path.join(normalize_repo_path(raw)).exists()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SPEC_SOURCE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn with_isolated_spec_cache<T>(test: impl FnOnce(&Path) -> T) -> T {
        let _guard = SPEC_SOURCE_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        let cache_root = std::env::temp_dir().join(format!(
            "dc-runner-spec-source-tests-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|v| v.as_nanos())
                .unwrap_or_default()
        ));
        let _ = fs::remove_dir_all(&cache_root);
        fs::create_dir_all(&cache_root).expect("create cache root");
        let previous_dir = std::env::var_os("DC_RUNNER_SPEC_CACHE_DIR");
        let previous_mode = std::env::var_os("DC_RUNNER_SPEC_SOURCE");
        std::env::set_var("DC_RUNNER_SPEC_CACHE_DIR", &cache_root);
        std::env::set_var("DC_RUNNER_SPEC_SOURCE", "bundled");

        let result = test(&cache_root);

        if let Some(previous) = previous_dir {
            std::env::set_var("DC_RUNNER_SPEC_CACHE_DIR", previous);
        } else {
            std::env::remove_var("DC_RUNNER_SPEC_CACHE_DIR");
        }
        if let Some(previous) = previous_mode {
            std::env::set_var("DC_RUNNER_SPEC_SOURCE", previous);
        } else {
            std::env::remove_var("DC_RUNNER_SPEC_SOURCE");
        }
        let _ = fs::remove_dir_all(&cache_root);
        result
    }

    fn write_core_bundle(cache_root: &Path, version: &str) {
        let root = cache_root.join("bundles").join(version).join("specs");
        fs::create_dir_all(root.join("04_governance")).expect("create governance marker directory");
        fs::create_dir_all(root.join("00_core")).expect("create core marker directory");
        fs::write(
            root.join("04_governance/runner_entrypoints_v1.yaml"),
            "title: test\n",
        )
        .expect("write required bundled marker");
        fs::write(
            root.join("00_core/runner_version_contract_v1.yaml"),
            "title: test\n",
        )
        .expect("write required core marker");
    }

    fn make_entry(version: &str, verified: bool) -> CachedSpecBundle {
        CachedSpecBundle {
            version: version.to_string(),
            installed_at: 0,
            source_ref: format!("test:{version}"),
            checksum: Some("ok".to_string()),
            verified,
            checksum_checked_at: Some(0),
            published_at: None,
            signature_available: false,
        }
    }

    #[test]
    fn resolve_runtime_mode_bundled_prefers_highest_verified_core_bundle() {
        with_isolated_spec_cache(|cache_root| {
            write_core_bundle(cache_root, "1.2.0");
            write_core_bundle(cache_root, "2.0.0");
            let state = SpecSourceState {
                version: 1,
                active: "bundled".to_string(),
                installed: vec![make_entry("1.2.0", true), make_entry("2.0.0", true)],
                last_refresh_at: None,
                last_check_at: None,
                last_error: None,
            };
            save_state(&state).expect("seed state");

            let mode = resolve_runtime_mode(cache_root).expect("resolve");
            match mode {
                RuntimeSpecSource::Cache { version, .. } => assert_eq!(version, "2.0.0"),
                _ => panic!("expected cached source mode"),
            }
        });
    }

    #[test]
    fn resolve_runtime_mode_auto_prefers_workspace_then_cached_core() {
        with_isolated_spec_cache(|cache_root| {
            let workspace = cache_root.join("workspace");
            fs::create_dir_all(workspace.join("specs")).expect("create workspace specs");
            fs::write(
                workspace.join("specs/workspace-marker.md"),
                "workspace-hit\n",
            )
            .expect("write workspace-only marker");

            write_core_bundle(cache_root, "2.0.0");
            fs::create_dir_all(cache_root.join("bundles/2.0.0/specs/cache-only"))
                .expect("cache dir");
            fs::write(
                cache_root.join("bundles/2.0.0/specs/cache-only/marker.md"),
                "cache-hit\n",
            )
            .expect("write cache marker");

            let state = SpecSourceState {
                version: 1,
                active: "bundled".to_string(),
                installed: vec![make_entry("2.0.0", true)],
                last_refresh_at: None,
                last_check_at: None,
                last_error: None,
            };
            save_state(&state).expect("seed state");
            std::env::set_var("DC_RUNNER_SPEC_SOURCE", "auto");

            let workspace_text =
                read_spec_text(&workspace, "specs/workspace-marker.md").expect("workspace-first");
            assert_eq!(workspace_text, "workspace-hit\n");

            let cache_text = read_spec_text(&workspace, "specs/cache-only/marker.md")
                .expect("cached-core fallback");
            assert_eq!(cache_text, "cache-hit\n");

            let exists_before_cleanup =
                spec_exists(&workspace, "specs/cache-only/marker.md").expect("cache file exists");
            assert!(exists_before_cleanup);
        });
    }

    #[test]
    fn resolve_runtime_mode_bundled_falls_back_to_embedded_for_invalid_active_cache() {
        with_isolated_spec_cache(|_cache_root| {
            let state = SpecSourceState {
                version: 1,
                active: "cache:1.0.0".to_string(),
                installed: vec![make_entry("1.0.0", true)],
                last_refresh_at: None,
                last_check_at: None,
                last_error: None,
            };
            save_state(&state).expect("seed state");
            let mode = resolve_runtime_mode(Path::new(".")).expect("resolve");
            assert!(matches!(mode, RuntimeSpecSource::Bundled));
            let reloaded = load_state().expect("load state");
            let last_error = reloaded.last_error.unwrap_or_default();
            assert!(last_error.contains("missing") || last_error.contains("fallback"));
        });
    }

    #[test]
    fn resolve_runtime_mode_bundled_preserves_active_cache_when_valid() {
        with_isolated_spec_cache(|cache_root| {
            write_core_bundle(cache_root, "3.0.0");
            let state = SpecSourceState {
                version: 1,
                active: "cache:3.0.0".to_string(),
                installed: vec![make_entry("3.0.0", true)],
                last_refresh_at: None,
                last_check_at: None,
                last_error: None,
            };
            save_state(&state).expect("seed state");
            let mode = resolve_runtime_mode(cache_root).expect("resolve");
            match mode {
                RuntimeSpecSource::Cache { version, .. } => {
                    assert_eq!(version, "3.0.0");
                }
                _ => panic!("expected cached source mode"),
            }
            let reloaded = load_state().expect("load state");
            assert!(reloaded.last_error.is_none());
        });
    }
}
