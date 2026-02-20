use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::constants::{REQUIRED_SUBCOMMAND_FALLBACK, SNAP_ROOT};
use crate::git::run_inherited;
use crate::sync::spec_sync_check;

#[derive(Debug, Clone)]
pub struct CompatReport {
    pub required_subcommands_count: usize,
}

fn repo_root() -> Result<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("failed to resolve repository root"))
}

fn runner_bin_path(root: &Path) -> PathBuf {
    root.join("target").join("debug").join("spec_runner_cli")
}

fn ensure_runner_binary() -> Result<()> {
    run_inherited(Command::new("cargo").args([
        "build",
        "--manifest-path",
        "spec_runner_cli/Cargo.toml",
    ]))
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

pub fn compat_check(source: Option<&str>) -> Result<CompatReport> {
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
    let governance_case =
        snap_root.join("specs/governance/cases/core/runtime_runner_interface_subcommands.spec.md");
    let required = match parse_required_subcommands(&contract_file) {
        Ok(cmds) if !cmds.is_empty() => cmds,
        _ => match parse_required_subcommands(&governance_case) {
            Ok(cmds) if !cmds.is_empty() => cmds,
            _ => REQUIRED_SUBCOMMAND_FALLBACK
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        },
    };

    for cmd in &required {
        let (code, _stdout, stderr) =
            run_capture(Command::new(&runner_bin).arg(cmd).current_dir(&root))?;
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

    Ok(CompatReport {
        required_subcommands_count: required.len(),
    })
}
