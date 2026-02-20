use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::constants::{
    DEFAULT_RUNNER_SPEC_SOURCE, DEFAULT_SOURCE, EXCLUDE_PATTERNS, INCLUDE_PATTERNS, LOCK_FILE,
    MANIFEST_FILE, REQUIRED_RUNNER_SPEC_FILES, REQUIRED_SNAPSHOT_FILES,
    RUNNER_SPEC_EXCLUDE_PATTERNS, RUNNER_SPEC_INCLUDE_PATTERNS, RUNNER_SPEC_LOCK_FILE,
    RUNNER_SPEC_MANIFEST_FILE, RUNNER_SPEC_SNAP_ROOT, SNAP_ROOT,
};
use crate::error::{AppError, AppResult};
use crate::git::{resolve_ref, run_git, with_source_repo, ResolvedRefKind};
use crate::lockfile::{
    load_lock, validate_lock, write_lock, Integrity, LockV1, Snapshot, Upstream,
};
use crate::manifest::{sha256_bytes, verify_manifest, write_manifest};

#[derive(Debug, Clone)]
pub struct SyncReport {
    pub tag: String,
    pub commit: String,
    pub file_count: u64,
    pub manifest_hash: String,
    pub non_tag_ref: bool,
}

#[derive(Debug, Clone)]
pub struct CheckReport {
    pub commit: String,
    pub file_count: u64,
    pub manifest_hash: String,
}

#[derive(Debug, Clone)]
struct SyncConfig<'a> {
    default_source: &'a str,
    lock_file: &'a str,
    snap_root: &'a str,
    manifest_file: &'a str,
    include: &'a [&'a str],
    exclude: &'a [&'a str],
    required_files: &'a [&'a str],
}

const DATA_CONTRACTS_SYNC_CONFIG: SyncConfig<'static> = SyncConfig {
    default_source: DEFAULT_SOURCE,
    lock_file: LOCK_FILE,
    snap_root: SNAP_ROOT,
    manifest_file: MANIFEST_FILE,
    include: INCLUDE_PATTERNS,
    exclude: EXCLUDE_PATTERNS,
    required_files: REQUIRED_SNAPSHOT_FILES,
};

const RUNNER_SPEC_SYNC_CONFIG: SyncConfig<'static> = SyncConfig {
    default_source: DEFAULT_RUNNER_SPEC_SOURCE,
    lock_file: RUNNER_SPEC_LOCK_FILE,
    snap_root: RUNNER_SPEC_SNAP_ROOT,
    manifest_file: RUNNER_SPEC_MANIFEST_FILE,
    include: RUNNER_SPEC_INCLUDE_PATTERNS,
    exclude: RUNNER_SPEC_EXCLUDE_PATTERNS,
    required_files: REQUIRED_RUNNER_SPEC_FILES,
};

fn repo_root() -> Result<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("failed to resolve repository root"))
}

fn build_globset(patterns: &[&str]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        builder.add(Glob::new(pat).with_context(|| format!("invalid glob pattern: {pat}"))?);
    }
    builder.build().context("failed building globset")
}

fn all_files_from_commit(repo: &Path, commit: &str) -> Result<Vec<String>> {
    let out = run_git(repo, &["ls-tree", "-r", "--name-only", commit, "specs"])?;
    let mut files = out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn show_file_from_commit(repo: &Path, commit: &str, rel: &str) -> Result<Vec<u8>> {
    let spec = format!("{commit}:{rel}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("show")
        .arg(spec)
        .output()
        .with_context(|| format!("failed to read file from commit: {rel}"))?;
    if !output.status.success() {
        bail!("failed reading file '{}' from commit '{}'", rel, commit);
    }
    Ok(output.stdout)
}

fn copy_selected_files_from_commit(
    repo: &Path,
    commit: &str,
    snap_root: &Path,
    include_patterns: &[&str],
    exclude_patterns: &[&str],
) -> Result<()> {
    let include = build_globset(include_patterns)?;
    let exclude = build_globset(exclude_patterns)?;
    let mut include_hits = vec![false; include_patterns.len()];

    if snap_root.exists() {
        fs::remove_dir_all(snap_root)
            .with_context(|| format!("failed removing {}", snap_root.display()))?;
    }
    fs::create_dir_all(snap_root)
        .with_context(|| format!("failed creating {}", snap_root.display()))?;

    for rel in all_files_from_commit(repo, commit)? {
        let rel_path = Path::new(&rel);
        if !include.is_match(rel_path) || exclude.is_match(rel_path) {
            continue;
        }
        for (idx, pat) in include_patterns.iter().enumerate() {
            let g = Glob::new(pat)?;
            if g.compile_matcher().is_match(rel_path) {
                include_hits[idx] = true;
            }
        }

        let bytes = show_file_from_commit(repo, commit, &rel)?;
        let dest = snap_root.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed creating {}", parent.display()))?;
        }
        fs::write(&dest, bytes).with_context(|| format!("failed writing {}", dest.display()))?;
    }

    for (idx, hit) in include_hits.iter().enumerate() {
        if !hit {
            bail!(
                "include pattern matched no files: {}",
                include_patterns[idx]
            );
        }
    }

    Ok(())
}

fn required_snapshot_files_exist(snapshot_root: &Path, required_files: &[&str]) -> Result<()> {
    for rel in required_files {
        let path = snapshot_root.join(rel);
        if !path.is_file() {
            bail!("required snapshot file missing: {rel}");
        }
    }
    Ok(())
}

fn sync_with_config(
    config: &SyncConfig<'_>,
    tag: &str,
    source: Option<&str>,
    allow_ref: bool,
) -> AppResult<SyncReport> {
    let root = repo_root()?;
    let lock_file = root.join(config.lock_file);
    let snap_root = root.join(config.snap_root);
    let manifest_file = root.join(config.manifest_file);

    let source_raw = source.unwrap_or(config.default_source);
    let mut report: Option<SyncReport> = None;
    with_source_repo(source_raw, |repo| {
        let (commit, kind) = resolve_ref(repo, tag)?;
        if kind != ResolvedRefKind::Tag && !allow_ref {
            return Err(anyhow!("non-tag ref '{tag}' requires --allow-ref"));
        }

        copy_selected_files_from_commit(repo, &commit, &snap_root, config.include, config.exclude)?;
        write_manifest(&snap_root, &manifest_file)?;
        let manifest_bytes = fs::read(&manifest_file)
            .with_context(|| format!("failed reading {}", manifest_file.display()))?;
        let file_count = manifest_bytes
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
            .count() as u64;
        let manifest_hash = sha256_bytes(&manifest_bytes);

        let lock = LockV1 {
            version: 1,
            upstream: Upstream {
                repo: config.default_source.to_string(),
                tag: tag.to_string(),
                commit: commit.clone(),
                synced_at_utc: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            },
            snapshot: Snapshot::curated(config.snap_root, config.include, config.exclude),
            integrity: Integrity {
                file_count,
                sha256_manifest: manifest_hash.clone(),
            },
        };
        write_lock(&lock_file, &lock)?;
        let checked = check_with_config(config, None)?;
        report = Some(SyncReport {
            tag: tag.to_string(),
            commit,
            file_count: checked.file_count,
            manifest_hash: checked.manifest_hash,
            non_tag_ref: kind == ResolvedRefKind::Ref,
        });
        Ok(())
    })
    .map_err(|error| {
        let msg = format!("{error:#}");
        if msg.contains("requires --allow-ref") {
            AppError::usage(msg)
        } else {
            AppError::from(error)
        }
    })?;

    report.ok_or_else(|| AppError::from(anyhow!("sync did not produce a report")))
}

fn check_with_config(config: &SyncConfig<'_>, source: Option<&str>) -> Result<CheckReport> {
    let root = repo_root()?;
    let lock_file = root.join(config.lock_file);
    let snap_root = root.join(config.snap_root);
    let manifest_file = root.join(config.manifest_file);

    if !lock_file.is_file() {
        bail!("lock file missing: {}", lock_file.display());
    }
    if !manifest_file.is_file() {
        bail!("manifest missing: {}", manifest_file.display());
    }
    if !snap_root.is_dir() {
        bail!("snapshot root missing: {}", snap_root.display());
    }

    let lock = load_lock(&lock_file)?;
    validate_lock(&lock, Some(config.snap_root))?;
    let (computed_file_count, computed_manifest_hash) =
        verify_manifest(&snap_root, &manifest_file)?;

    if computed_manifest_hash != lock.integrity.sha256_manifest {
        bail!("lock manifest hash mismatch");
    }
    if computed_file_count != lock.integrity.file_count {
        bail!("lock file_count mismatch");
    }

    required_snapshot_files_exist(&snap_root, config.required_files)?;

    if let Some(src) = source {
        with_source_repo(src, |repo| {
            match resolve_ref(repo, &lock.upstream.tag) {
                Ok((resolved, _)) => {
                    if resolved != lock.upstream.commit {
                        bail!("lock tag resolves to different commit in source");
                    }
                }
                Err(_) => {
                    eprintln!(
                        "WARN: lock tag '{}' not found in source '{}'; commit pinned locally only",
                        lock.upstream.tag, src
                    );
                }
            }
            Ok(())
        })?;
    }

    Ok(CheckReport {
        commit: lock.upstream.commit,
        file_count: computed_file_count,
        manifest_hash: computed_manifest_hash,
    })
}

pub fn spec_sync(tag: &str, source: Option<&str>, allow_ref: bool) -> AppResult<SyncReport> {
    sync_with_config(&DATA_CONTRACTS_SYNC_CONFIG, tag, source, allow_ref)
}

pub fn spec_sync_check(source: Option<&str>) -> Result<CheckReport> {
    check_with_config(&DATA_CONTRACTS_SYNC_CONFIG, source)
}

pub fn runner_spec_sync(tag: &str, source: Option<&str>, allow_ref: bool) -> AppResult<SyncReport> {
    sync_with_config(&RUNNER_SPEC_SYNC_CONFIG, tag, source, allow_ref)
}

pub fn runner_spec_check(source: Option<&str>) -> Result<CheckReport> {
    check_with_config(&RUNNER_SPEC_SYNC_CONFIG, source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globset_builds_for_defaults() {
        build_globset(INCLUDE_PATTERNS).expect("include globset");
        build_globset(EXCLUDE_PATTERNS).expect("exclude globset");
        build_globset(RUNNER_SPEC_INCLUDE_PATTERNS).expect("runner-spec include globset");
        build_globset(RUNNER_SPEC_EXCLUDE_PATTERNS).expect("runner-spec exclude globset");
    }
}
