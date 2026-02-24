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
    let output = Command::new(env!("CARGO_BIN_EXE_dc-runner"))
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
    let root = repo_root();
    let candidates = [
        "specs/upstream/data-contracts/specs/04_governance/cases/core/runner_contract/runtime_runner_interface_subcommands.spec.md",
        "specs/upstream/data-contracts/specs/governance/cases/core/runner_contract/runtime_runner_interface_subcommands.spec.md",
        "specs/upstream/data-contracts/specs/02_contracts/12_runner_interface.md",
        "specs/upstream/data-contracts/specs/contract/12_runner_interface.md",
    ];
    let text = candidates
        .iter()
        .find_map(|p| fs::read_to_string(root.join(p)).ok())
        .expect("read contract file");
    let mut out = Vec::new();
    let mut in_markdown_block = false;
    let mut in_yaml_list = false;
    for line in text.lines() {
        if line.contains("Required subcommands:") {
            in_markdown_block = true;
            continue;
        }
        if in_markdown_block && line.contains("CI expectation:") {
            break;
        }
        if in_markdown_block {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("- `") {
                if let Some(end) = rest.find('`') {
                    out.push(rest[..end].to_string());
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
        return [
            "governance",
            "style-check",
            "lint",
            "typecheck",
            "compilecheck",
            "conformance-purpose-json",
            "conformance-purpose-md",
            "runner-independence-json",
            "runner-independence-md",
            "python-dependency-json",
            "python-dependency-md",
            "ci-gate-summary",
            "docs-generate",
            "docs-generate-check",
            "conformance-parity",
            "runner-certify",
            "test-core",
            "test-full",
            "job-run",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    }
    out
}

fn dispatched_subcommands_from_source() -> HashSet<String> {
    let src = fs::read_to_string(repo_root().join("dc-runner-cli/src/app/dispatch.rs"))
        .expect("read app source");
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
    assert!(
        !required.is_empty(),
        "expected non-empty required subcommands"
    );
    let dispatched = dispatched_subcommands_from_source();
    for cmd in required {
        assert!(
            dispatched.contains(&cmd),
            "missing dispatch handler for required command: {cmd}"
        );
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
    assert!(stderr.contains("unrecognized subcommand"));
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
    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&trace);
    let out_s = out.to_string_lossy().to_string();
    let trace_s = trace.to_string_lossy().to_string();
    let runner_bin = env!("CARGO_BIN_EXE_dc-runner");
    let (code, _stdout, stderr) = run_cli(&[
        "ci-gate-summary",
        "--out",
        &out_s,
        "--trace-out",
        &trace_s,
        "--runner-bin",
        runner_bin,
    ]);
    assert_ne!(
        code, 2,
        "ci-gate-summary should not fail as usage/config error"
    );
    assert!(
        out.is_file(),
        "missing gate summary at {} ; stderr: {}",
        out.display(),
        stderr
    );
    assert!(
        trace.is_file(),
        "missing gate trace at {} ; stderr: {}",
        trace.display(),
        stderr
    );
}

#[test]
fn ci_gate_summary_invalid_runner_bin_fails_with_runtime_error() {
    let (code, _stdout, stderr) = run_cli(&[
        "ci-gate-summary",
        "--runner-bin",
        "/does/not/exist/dc-runner",
    ]);
    assert_ne!(code, 2);
    assert_ne!(code, 0);
    assert!(stderr.contains("runner binary"));
}

#[test]
fn help_works_and_mentions_specs_group() {
    let (code, stdout, _stderr) = run_cli(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("specs"));
    assert!(stdout.contains("governance"));
    assert!(stdout.contains("docs"));
    assert!(stdout.contains("schema"));
    assert!(!stdout.contains("entrypoints"));
    assert!(!stdout.contains("--profile-level"));
}

#[test]
fn specs_list_returns_zero() {
    let (code, stdout, _stderr) = run_cli(&["specs", "list"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("spec cases") || stdout.contains("No spec cases found"));
}

#[test]
fn help_advanced_lists_hidden_runtime_flags() {
    let (code, stdout, _stderr) = run_cli(&["help-advanced"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("--profile-level"));
    assert!(stdout.contains("--liveness-level"));
}

#[test]
fn governance_group_help_is_concise() {
    let (code, stdout, _stderr) = run_cli(&["governance", "--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("run"));
    assert!(!stdout.contains("--profile-level"));
}

#[test]
fn entrypoints_help_shows_list_and_run() {
    let (code, stdout, _stderr) = run_cli(&["entrypoints", "--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("run"));
}

#[test]
fn entrypoints_list_includes_required_ids() {
    let (code, stdout, _stderr) = run_cli(&["entrypoints", "list"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("governance"));
    assert!(stdout.contains("critical-gate"));
    assert!(stdout.contains("bundle-list"));
    assert!(stdout.contains("bundle-inspect"));
    assert!(stdout.contains("bundle-install"));
    assert!(stdout.contains("docs-generate"));
    assert!(stdout.contains("docs-generate-check"));
    assert!(stdout.contains("docs-build"));
    assert!(stdout.contains("docs-build-check"));
    assert!(stdout.contains("docs-lint"));
    assert!(stdout.contains("docs-graph"));
    assert!(stdout.contains("specs-refresh"));
    assert!(stdout.contains("specs-status"));
    assert!(stdout.contains("specs-versions"));
    assert!(stdout.contains("specs-use"));
    assert!(stdout.contains("specs-rollback"));
    assert!(stdout.contains("specs-verify"));
    assert!(stdout.contains("specs-clean"));
    assert!(stdout.contains("specs-info"));
    assert!(stdout.contains("specs-prune"));
}

#[test]
fn entrypoints_list_supports_explicit_spec_source_flag() {
    let (code, _stdout, _stderr) = run_cli(&["--spec-source", "bundled", "entrypoints", "list"]);
    assert_eq!(code, 0);
}

#[test]
fn entrypoints_json_includes_visibility_group_and_source() {
    let (code, stdout, _stderr) = run_cli(&["entrypoints", "list", "--format", "json"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("\"visibility\""));
    assert!(stdout.contains("\"source\""));
}

#[test]
fn invalid_spec_source_value_returns_usage_error() {
    let (code, _stdout, stderr) = run_cli(&["--spec-source", "invalid", "entrypoints", "list"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("invalid value"));
}

#[test]
fn docs_commands_resolve_via_entrypoints() {
    let commands = [
        ["docs", "generate"],
        ["docs", "generate-check"],
        ["docs", "build"],
        ["docs", "build-check"],
        ["docs", "lint"],
        ["docs", "graph"],
    ];
    for cmd in commands {
        let (code, _stdout, _stderr) = run_cli(&cmd);
        assert_ne!(code, 2, "docs command returned usage error: {:?}", cmd);
    }
}

#[test]
fn schema_commands_resolve_via_entrypoints() {
    let commands = [
        ["schema", "check"],
        ["schema", "lint"],
        ["schema", "format"],
    ];
    for cmd in commands {
        let (code, _stdout, _stderr) = run_cli(&cmd);
        assert_ne!(code, 2, "schema command returned usage error: {:?}", cmd);
    }
}

#[test]
fn legacy_schema_aliases_are_rejected() {
    let commands = [
        ["schema-registry-build"],
        ["schema-registry-check"],
        ["schema-docs-build"],
        ["schema-docs-check"],
    ];
    for cmd in commands {
        let (code, _stdout, stderr) = run_cli(&cmd);
        assert_eq!(code, 2, "expected usage error for legacy alias {:?}", cmd);
        assert!(stderr.contains("unrecognized subcommand"));
    }
}

#[cfg(not(feature = "bundler"))]
#[test]
fn bundler_group_is_absent_without_feature() {
    let (code, stdout, _stderr) = run_cli(&["--help"]);
    assert_eq!(code, 0);
    assert!(!stdout.contains("bundler"));
    let (code2, _stdout2, stderr2) = run_cli(&["bundler", "resolve"]);
    assert_eq!(code2, 2);
    assert!(stderr2.contains("unrecognized subcommand"));
}

#[cfg(feature = "bundler")]
#[test]
fn bundler_group_is_present_with_feature() {
    let (code, stdout, _stderr) = run_cli(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("bundler"));
    let (code2, stdout2, _stderr2) = run_cli(&["entrypoints", "list"]);
    assert_eq!(code2, 0);
    assert!(stdout2.contains("bundler-resolve"));
    assert!(stdout2.contains("bundler-package"));
    assert!(stdout2.contains("bundler-check"));
}

#[test]
fn entrypoints_run_executes_known_id() {
    let (code, _stdout, _stderr) = run_cli(&["entrypoints", "run", "governance"]);
    assert!(code == 0 || code == 1);
}

#[test]
fn entrypoints_run_missing_id_returns_usage_error() {
    let (code, _stdout, stderr) = run_cli(&["entrypoints", "run"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("Usage:") || stderr.contains("usage:"));
}

#[test]
fn entrypoints_run_id_flag_is_rejected() {
    let (code, _stdout, stderr) = run_cli(&["entrypoints", "run", "--id", "governance"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("unexpected argument") || stderr.contains("unrecognized"));
}

#[test]
fn entrypoints_run_unknown_id_returns_non_zero_with_ids() {
    let (code, _stdout, stderr) = run_cli(&["entrypoints", "run", "__does_not_exist__"]);
    assert_eq!(code, 2);
    assert!(stderr.contains("Available entrypoint ids"));
}
