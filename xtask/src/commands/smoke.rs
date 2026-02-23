use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::commands;

fn repo_root() -> Result<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("failed to resolve repository root"))
}

fn runner_bin_path(root: &Path) -> PathBuf {
    root.join("target").join("debug").join("dc-runner")
}

pub fn run() -> Result<()> {
    commands::build::run()?;
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
