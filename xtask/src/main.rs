use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use walkdir::WalkDir;

const DEFAULT_SOURCE: &str = "https://github.com/jonruttan/data-contracts.git";
const LOCK_FILE: &str = "specs/upstream/data_contracts_lock_v1.yaml";
const SNAP_ROOT: &str = "specs/upstream/data-contracts";
const MANIFEST_FILE: &str = "specs/upstream/data-contracts.manifest.sha256";

const INCLUDE_PATTERNS: &[&str] = &[
    "specs/index.md",
    "specs/current.md",
    "specs/contract/**",
    "specs/schema/**",
    "specs/conformance/**",
    "specs/governance/index.md",
    "specs/governance/check_*.yaml",
    "specs/governance/cases/core/**",
    "specs/governance/metrics/**",
];

const EXCLUDE_PATTERNS: &[&str] = &["**/pending/**", "**/reviews/**", "**/snapshots/**"];

const REQUIRED_SNAPSHOT_FILES: &[&str] = &[
    "specs/index.md",
    "specs/current.md",
    "specs/contract/index.md",
    "specs/contract/policy_v1.yaml",
    "specs/contract/traceability_v1.yaml",
    "specs/schema/index.md",
    "specs/schema/runner_certification_registry_v1.yaml",
    "specs/schema/dc_runner_rust_lock_v1.yaml",
    "specs/governance/index.md",
    "specs/governance/check_sets_v1.yaml",
    "specs/governance/check_prefix_map_v1.yaml",
    "specs/governance/cases/core/index.md",
];

#[derive(Parser)]
#[command(author, version, about = "Rust-native task runner for dc-runner-rust")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Build,
    Test,
    Smoke,
    SpecSync { tag: String, source: Option<String> },
    SpecSyncCheck { source: Option<String> },
    CompatCheck { source: Option<String> },
    Verify { source: Option<String> },
}

#[derive(Debug)]
struct LockV1 {
    version: u32,
    upstream: Upstream,
    snapshot: Snapshot,
    integrity: Integrity,
}

#[derive(Debug)]
struct Upstream {
    repo: String,
    tag: String,
    commit: String,
    synced_at_utc: String,
}

#[derive(Debug)]
struct Snapshot {
    root: String,
}

#[derive(Debug)]
struct Integrity {
    file_count: u64,
    sha256_manifest: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("ERROR: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Build => cargo(&["build", "--manifest-path", "spec_runner_cli/Cargo.toml"]),
        Cmd::Test => cargo(&["test", "--manifest-path", "spec_runner_cli/Cargo.toml"]),
        Cmd::Smoke => smoke(),
        Cmd::SpecSync { tag, source } => spec_sync(&tag, source.as_deref()),
        Cmd::SpecSyncCheck { source } => spec_sync_check(source.as_deref()),
        Cmd::CompatCheck { source } => compat_check(source.as_deref()),
        Cmd::Verify { source } => {
            cargo(&["build", "--manifest-path", "spec_runner_cli/Cargo.toml"])?;
            cargo(&["test", "--manifest-path", "spec_runner_cli/Cargo.toml"])?;
            spec_sync_check(source.as_deref())?;
            compat_check(source.as_deref())
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("failed to resolve repository root"))
}

fn cargo(args: &[&str]) -> Result<()> {
    run_inherited(Command::new("cargo").args(args))
}

fn run_inherited(cmd: &mut Command) -> Result<()> {
    let program = format!("{:?}", cmd);
    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to run {program}"))?;
    if !status.success() {
        bail!("command failed: {program} (status {status})");
    }
    Ok(())
}

fn smoke() -> Result<()> {
    ensure_runner_binary()?;
    let root = repo_root()?;
    let status = Command::new(runner_bin_path(&root))
        .arg("style-check")
        .current_dir(root)
        .status()
        .context("failed to run smoke command")?;
    if !status.success() {
        bail!("smoke failed with status {status}");
    }
    Ok(())
}

fn ensure_runner_binary() -> Result<()> {
    cargo(&["build", "--manifest-path", "spec_runner_cli/Cargo.toml"])
}

fn runner_bin_path(root: &Path) -> PathBuf {
    root.join("target").join("debug").join("spec_runner_cli")
}

fn build_globset(patterns: &[&str]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        builder.add(Glob::new(pat).with_context(|| format!("invalid glob pattern: {pat}"))?);
    }
    builder.build().context("failed building globset")
}

fn is_local_git_repo(source: &str) -> bool {
    Path::new(source).join(".git").is_dir()
}

fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git -C {} {:?}", repo.display(), args))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "git command failed in {}: {:?}: {}",
            repo.display(),
            args,
            stderr.trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn resolve_ref(repo: &Path, reference: &str) -> Result<(String, String)> {
    if let Ok(commit) = run_git(
        repo,
        &[
            "rev-parse",
            "--verify",
            &format!("refs/tags/{reference}^{{commit}}"),
        ],
    ) {
        return Ok((commit, "tag".to_string()));
    }
    let commit = run_git(
        repo,
        &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
    )
    .with_context(|| format!("cannot resolve tag/ref '{reference}'"))?;
    Ok((commit, "ref".to_string()))
}

fn with_source_repo<F>(source: Option<&str>, mut f: F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    let source = source.unwrap_or(DEFAULT_SOURCE);
    if is_local_git_repo(source) {
        return f(Path::new(source));
    }

    let temp = TempDir::new().context("failed to create temp dir")?;
    let clone_path = temp.path().join("source");
    run_inherited(
        Command::new("git")
            .arg("clone")
            .arg("--quiet")
            .arg("--filter=blob:none")
            .arg("--no-checkout")
            .arg(source)
            .arg(&clone_path),
    )?;
    f(&clone_path)
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

fn copy_selected_files_from_commit(repo: &Path, commit: &str, snap_root: &Path) -> Result<()> {
    let include = build_globset(INCLUDE_PATTERNS)?;
    let exclude = build_globset(EXCLUDE_PATTERNS)?;
    let mut include_hits = vec![false; INCLUDE_PATTERNS.len()];

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
        for (idx, pat) in INCLUDE_PATTERNS.iter().enumerate() {
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
                INCLUDE_PATTERNS[idx]
            );
        }
    }

    Ok(())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn manifest_entries(snapshot_root: &Path) -> Result<Vec<(String, String)>> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(snapshot_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let rel = path
            .strip_prefix(snapshot_root)
            .with_context(|| format!("failed to strip prefix for {}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(path).with_context(|| format!("failed reading {}", path.display()))?;
        entries.push((rel, sha256_bytes(&bytes)));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

fn write_manifest(snapshot_root: &Path, manifest_file: &Path) -> Result<()> {
    if let Some(parent) = manifest_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }
    let mut out = Vec::new();
    for (rel, hash) in manifest_entries(snapshot_root)? {
        out.extend_from_slice(format!("{hash}  {rel}\n").as_bytes());
    }
    fs::write(manifest_file, out)
        .with_context(|| format!("failed writing {}", manifest_file.display()))
}

fn required_snapshot_files_exist(snapshot_root: &Path) -> Result<()> {
    for rel in REQUIRED_SNAPSHOT_FILES {
        let path = snapshot_root.join(rel);
        if !path.is_file() {
            bail!("required snapshot file missing: {rel}");
        }
    }
    Ok(())
}

fn extract_lock_value(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        if k.trim_end() != key {
            return None;
        }
        Some(v.trim().trim_matches('\"').to_string())
    })
}

fn load_lock(lock_file: &Path) -> Result<LockV1> {
    let txt = fs::read_to_string(lock_file)
        .with_context(|| format!("failed reading {}", lock_file.display()))?;
    let version = extract_lock_value(&txt, "version")
        .ok_or_else(|| anyhow!("lock version missing"))?
        .parse::<u32>()
        .context("invalid lock version")?;
    let repo =
        extract_lock_value(&txt, "  repo").ok_or_else(|| anyhow!("lock upstream.repo missing"))?;
    let tag =
        extract_lock_value(&txt, "  tag").ok_or_else(|| anyhow!("lock upstream.tag missing"))?;
    let commit = extract_lock_value(&txt, "  commit")
        .ok_or_else(|| anyhow!("lock upstream.commit missing"))?;
    let synced_at_utc = extract_lock_value(&txt, "  synced_at_utc")
        .ok_or_else(|| anyhow!("lock upstream.synced_at_utc missing"))?;
    let root =
        extract_lock_value(&txt, "  root").ok_or_else(|| anyhow!("lock snapshot.root missing"))?;
    let file_count = extract_lock_value(&txt, "  file_count")
        .ok_or_else(|| anyhow!("lock integrity.file_count missing"))?
        .parse::<u64>()
        .context("invalid lock integrity.file_count")?;
    let sha256_manifest = extract_lock_value(&txt, "  sha256_manifest")
        .ok_or_else(|| anyhow!("lock integrity.sha256_manifest missing"))?;

    Ok(LockV1 {
        version,
        upstream: Upstream {
            repo,
            tag,
            commit,
            synced_at_utc,
        },
        snapshot: Snapshot { root },
        integrity: Integrity {
            file_count,
            sha256_manifest,
        },
    })
}

fn write_lock(lock_file: &Path, lock: &LockV1) -> Result<()> {
    if let Some(parent) = lock_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }
    let txt = format!(
        "version: {version}\nupstream:\n  repo: {repo}\n  tag: {tag}\n  commit: {commit}\n  synced_at_utc: \"{synced_at_utc}\"\nsnapshot:\n  root: {root}\n  include:\n    - specs/index.md\n    - specs/current.md\n    - specs/contract/**\n    - specs/schema/**\n    - specs/conformance/**\n    - specs/governance/index.md\n    - specs/governance/check_*.yaml\n    - specs/governance/cases/core/**\n    - specs/governance/metrics/**\n  exclude:\n    - \"**/pending/**\"\n    - \"**/reviews/**\"\n    - \"**/snapshots/**\"\nintegrity:\n  file_count: {file_count}\n  sha256_manifest: {manifest}\n",
        version = lock.version,
        repo = lock.upstream.repo,
        tag = lock.upstream.tag,
        commit = lock.upstream.commit,
        synced_at_utc = lock.upstream.synced_at_utc,
        root = lock.snapshot.root,
        file_count = lock.integrity.file_count,
        manifest = lock.integrity.sha256_manifest
    );
    fs::write(lock_file, txt).with_context(|| format!("failed writing {}", lock_file.display()))
}

fn spec_sync(tag: &str, source: Option<&str>) -> Result<()> {
    let root = repo_root()?;
    let lock_file = root.join(LOCK_FILE);
    let snap_root = root.join(SNAP_ROOT);
    let manifest_file = root.join(MANIFEST_FILE);

    let source_raw = source.unwrap_or(DEFAULT_SOURCE);
    with_source_repo(Some(source_raw), |repo| {
        let (commit, kind) = resolve_ref(repo, tag)?;
        if kind != "tag" {
            eprintln!(
                "WARN: '{}' resolved as non-tag git ref; release workflow should use immutable tags.",
                tag
            );
        }

        copy_selected_files_from_commit(repo, &commit, &snap_root)?;
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
                repo: DEFAULT_SOURCE.to_string(),
                tag: tag.to_string(),
                commit: commit.clone(),
                synced_at_utc: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            },
            snapshot: Snapshot {
                root: SNAP_ROOT.to_string(),
            },
            integrity: Integrity {
                file_count,
                sha256_manifest: manifest_hash,
            },
        };
        write_lock(&lock_file, &lock)?;
        spec_sync_check(None)?;
        println!(
            "OK: wrote lock + snapshot + manifest for {} ({})",
            tag, commit
        );
        Ok(())
    })
}

fn spec_sync_check(source: Option<&str>) -> Result<()> {
    let root = repo_root()?;
    let lock_file = root.join(LOCK_FILE);
    let snap_root = root.join(SNAP_ROOT);
    let manifest_file = root.join(MANIFEST_FILE);

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
    if lock.version != 1 {
        bail!("lock version must be 1");
    }
    if lock.upstream.commit.len() != 40
        || !lock.upstream.commit.chars().all(|c| c.is_ascii_hexdigit())
    {
        bail!("lock upstream.commit must be 40-char sha");
    }
    if lock.snapshot.root != SNAP_ROOT {
        bail!("lock snapshot.root unexpected: {}", lock.snapshot.root);
    }

    let mut tmp = tempfile::NamedTempFile::new().context("failed creating temp manifest")?;
    for (rel, hash) in manifest_entries(&snap_root)? {
        writeln!(tmp, "{hash}  {rel}").context("failed writing temp manifest")?;
    }
    let tmp_bytes = fs::read(tmp.path()).context("failed reading temp manifest")?;
    let manifest_bytes = fs::read(&manifest_file)
        .with_context(|| format!("failed reading {}", manifest_file.display()))?;
    if tmp_bytes != manifest_bytes {
        bail!("snapshot manifest drift detected. Run spec sync update.");
    }

    let computed_manifest_hash = sha256_bytes(&manifest_bytes);
    let computed_file_count = manifest_bytes
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .count() as u64;

    if computed_manifest_hash != lock.integrity.sha256_manifest {
        bail!("lock manifest hash mismatch");
    }
    if computed_file_count != lock.integrity.file_count {
        bail!("lock file_count mismatch");
    }

    required_snapshot_files_exist(&snap_root)?;

    if let Some(src) = source {
        with_source_repo(Some(src), |repo| {
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

    println!("OK: upstream snapshot and lock are consistent");
    Ok(())
}

fn parse_required_subcommands(contract_file: &Path) -> Result<Vec<String>> {
    let txt = fs::read_to_string(contract_file)
        .with_context(|| format!("failed reading {}", contract_file.display()))?;
    let mut out = Vec::new();
    let mut in_markdown_block = false;
    let mut in_yaml_list = false;
    for line in txt.lines() {
        if line.contains("Required subcommands:") {
            in_markdown_block = true;
            continue;
        }
        if in_markdown_block && line.contains("CI expectation:") {
            break;
        }
        if in_markdown_block {
            let t = line.trim();
            if t.starts_with("- `") {
                let rest = &t[3..];
                if let Some(end) = rest.find('`') {
                    let cmd = &rest[..end];
                    if !cmd.is_empty() {
                        out.push(cmd.to_string());
                    }
                }
            }
            continue;
        }

        let t = line.trim();
        if t == "required_subcommands:" {
            in_yaml_list = true;
            continue;
        }
        if in_yaml_list {
            if let Some(rest) = t.strip_prefix("- ") {
                let cmd = rest.trim();
                if !cmd.is_empty() {
                    out.push(cmd.to_string());
                }
                continue;
            }
            if !t.is_empty() {
                break;
            }
        }
    }
    if out.is_empty() {
        bail!("failed to extract required subcommands from contract");
    }
    Ok(out)
}

fn run_capture(cmd: &mut Command) -> Result<(i32, String, String)> {
    let out = cmd.output().context("failed to run command")?;
    Ok((
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    ))
}

fn assert_exit_code(expected: i32, cmd: &mut Command, label: &str) -> Result<()> {
    let (code, stdout, stderr) = run_capture(cmd)?;
    if code != expected {
        bail!(
            "expected exit {}, got {} for {}\nstderr:\n{}\nstdout:\n{}",
            expected,
            code,
            label,
            stderr,
            stdout
        );
    }
    Ok(())
}

fn compat_check(source: Option<&str>) -> Result<()> {
    spec_sync_check(source)?;

    let root = repo_root()?;
    let snap_root = root.join(SNAP_ROOT);
    let runner_shim = root.join("runner_adapter.sh");
    let runner_bin = runner_bin_path(&root);

    ensure_runner_binary()?;

    if !runner_shim.is_file() {
        bail!("runner shim missing: {}", runner_shim.display());
    }
    if !runner_bin.is_file() {
        bail!("runner binary missing: {}", runner_bin.display());
    }

    for rel in [
        "specs/contract/index.md",
        "specs/contract/policy_v1.yaml",
        "specs/contract/traceability_v1.yaml",
        "specs/schema/index.md",
        "specs/schema/runner_certification_registry_v1.yaml",
        "specs/schema/dc_runner_rust_lock_v1.yaml",
        "specs/governance/index.md",
        "specs/governance/check_sets_v1.yaml",
        "specs/governance/cases/core/index.md",
    ] {
        if !snap_root.join(rel).is_file() {
            bail!("required compatibility file missing: {rel}");
        }
    }

    let contract_file = snap_root.join("specs/contract/12_runner_interface.md");
    let required = match parse_required_subcommands(&contract_file) {
        Ok(cmds) if !cmds.is_empty() => cmds,
        _ => {
            let governance_case = snap_root.join(
                "specs/governance/cases/core/runtime_runner_interface_subcommands.spec.md",
            );
            parse_required_subcommands(&governance_case)?
        }
    };

    for cmd in required {
        let (code, _stdout, stderr) =
            run_capture(Command::new(&runner_bin).arg(&cmd).current_dir(&root))?;
        if code == 2 && stderr.contains("unsupported runner adapter subcommand") {
            bail!("runner binary missing required subcommand: {cmd}");
        }
    }

    assert_exit_code(
        0,
        Command::new(&runner_bin)
            .arg("style-check")
            .current_dir(&root),
        "style-check",
    )?;

    assert_exit_code(
        1,
        Command::new(&runner_bin)
            .args(["job-run", "--ref", "#DOES_NOT_EXIST"])
            .current_dir(&root),
        "job-run --ref #DOES_NOT_EXIST",
    )?;

    assert_exit_code(
        2,
        Command::new(&runner_bin)
            .arg("__unknown_subcommand__")
            .current_dir(&root),
        "unknown subcommand",
    )?;

    let shim_text = fs::read_to_string(&runner_shim)
        .with_context(|| format!("failed reading {}", runner_shim.display()))?;
    if shim_text.contains("python ") || shim_text.contains("python3 ") {
        bail!("runner shim appears to execute python directly");
    }

    println!("OK: upstream compatibility verification passed");
    Ok(())
}
