use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use tempfile::TempDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedRefKind {
    Tag,
    Ref,
}

pub fn is_local_git_repo(source: &str) -> bool {
    Path::new(source).join(".git").is_dir()
}

pub fn run_inherited(cmd: &mut Command) -> Result<()> {
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

pub fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
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

pub fn resolve_ref(repo: &Path, reference: &str) -> Result<(String, ResolvedRefKind)> {
    if let Ok(commit) = run_git(
        repo,
        &[
            "rev-parse",
            "--verify",
            &format!("refs/tags/{reference}^{{commit}}"),
        ],
    ) {
        return Ok((commit, ResolvedRefKind::Tag));
    }
    let commit = run_git(
        repo,
        &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
    )
    .with_context(|| format!("cannot resolve tag/ref '{reference}'"))?;
    Ok((commit, ResolvedRefKind::Ref))
}

pub fn with_source_repo<F>(source: &str, mut f: F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    if is_local_git_repo(source) {
        return f(Path::new(source));
    }

    let temp = TempDir::new().context("failed to create temp dir")?;
    let clone_path: PathBuf = temp.path().join("source");
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
