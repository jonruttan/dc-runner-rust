use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_spec_runner_cli"))
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("run cli");
    let code = output.status.code().unwrap_or(1);
    (
        code,
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn required_contract_subcommands() -> Vec<String> {
    let contract = repo_root().join("specs/upstream/data-contracts/specs/contract/12_runner_interface.md");
    let text = fs::read_to_string(contract).expect("read contract file");
    let mut out = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        if line.contains("Required subcommands:") {
            in_block = true;
            continue;
        }
        if in_block && line.contains("CI expectation:") {
            break;
        }
        if in_block {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("- `") {
                if let Some(end) = rest.find('`') {
                    out.push(rest[..end].to_string());
                }
            }
        }
    }
    out
}

fn dispatched_subcommands_from_source() -> HashSet<String> {
    let src =
        fs::read_to_string(repo_root().join("spec_runner_cli/src/app/dispatch.rs")).expect("read app source");
    let mut set = HashSet::new();
    for line in src.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('"') || !trimmed.contains("=>") {
            continue;
        }
        if let Some(end) = trimmed[1..].find('"') {
            let cmd = &trimmed[1..1 + end];
            if !cmd.is_empty() {
                set.insert(cmd.to_string());
            }
        }
    }
    set
}

#[test]
fn required_contract_subcommands_are_dispatched() {
    let required = required_contract_subcommands();
    assert!(!required.is_empty(), "expected non-empty required subcommands");
    let dispatched = dispatched_subcommands_from_source();
    for cmd in required {
        assert!(dispatched.contains(&cmd), "missing dispatch handler for required command: {cmd}");
    }
}

#[test]
fn style_check_returns_zero() {
    let (code, _stdout, _stderr) = run_cli(&["style-check"]);
    assert_eq!(code, 0);
}

#[test]
fn unknown_subcommand_returns_two() {
    let (code, _stdout, stderr) = run_cli(&["__unknown_subcommand__"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("unsupported runner adapter subcommand"));
}

#[test]
fn job_run_unknown_ref_returns_one() {
    let (code, _stdout, _stderr) = run_cli(&["job-run", "--ref", "#DOES_NOT_EXIST"]);
    assert_eq!(code, 1);
}

#[test]
fn spec_eval_usage_error_returns_two() {
    let (code, _stdout, _stderr) = run_cli(&["spec-eval"]);
    assert_eq!(code, 2);
}

#[test]
fn ci_gate_summary_writes_outputs() {
    let out = repo_root().join(".artifacts/test-ci-gate-summary.json");
    let trace = repo_root().join(".artifacts/test-ci-gate-trace.json");
    let out_s = out.to_string_lossy().to_string();
    let trace_s = trace.to_string_lossy().to_string();
    let (code, _stdout, _stderr) = run_cli(&[
        "ci-gate-summary",
        "--out",
        &out_s,
        "--trace-out",
        &trace_s,
    ]);
    assert_ne!(code, 2, "ci-gate-summary should not fail as usage/config error");
    assert!(out.is_file());
    assert!(trace.is_file());
}
