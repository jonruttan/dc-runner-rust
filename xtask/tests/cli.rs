use std::fs;
use std::process::Command;

use assert_cmd::cargo::cargo_bin;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn create_temp_git_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let status = Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(dir.path())
        .status()
        .expect("git init");
    assert!(status.success());

    fs::write(dir.path().join("README.md"), "x\n").expect("write readme");

    let status = Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args(["add", "."])
        .status()
        .expect("git add");
    assert!(status.success());

    let status = Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args([
            "-c",
            "user.name=xtask-test",
            "-c",
            "user.email=xtask@example.com",
            "commit",
            "-qm",
            "init",
        ])
        .status()
        .expect("git commit");
    assert!(status.success());
    dir
}

#[test]
fn help_shows_grouped_commands() {
    let output = Command::new(cargo_bin("xtask"))
        .arg("--help")
        .current_dir(repo_root())
        .output()
        .expect("run help");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    let golden = fs::read_to_string(format!(
        "{}/tests/golden/help_top_level.txt",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("golden");
    for line in golden.lines().filter(|line| !line.is_empty()) {
        assert!(text.contains(line), "missing help line: {line}\n{text}");
    }
    assert!(!text.contains("spec-sync"));
    assert!(!text.contains("compat-check"));
    assert!(!text.contains("runner-spec-sync"));
}

#[test]
fn non_tag_ref_requires_allow_ref() {
    let repo = create_temp_git_repo();
    let commit = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("rev-parse");
    assert!(commit.status.success());
    let commit = String::from_utf8_lossy(&commit.stdout).trim().to_string();

    let output = Command::new(cargo_bin("xtask"))
        .args([
            "spec",
            "sync",
            "--tag",
            &commit,
            "--source",
            repo.path().to_str().expect("path"),
        ])
        .current_dir(repo_root())
        .output()
        .expect("run spec sync");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires --allow-ref"));
}

#[test]
fn legacy_alias_is_parsed_and_hidden_from_help() {
    let output = Command::new(cargo_bin("xtask"))
        .arg("--help")
        .current_dir(repo_root())
        .output()
        .expect("run help");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("spec-sync"));
    assert!(!text.contains("spec-sync-check"));
    assert!(!text.contains("compat-check"));
    assert!(!text.contains("runner-spec-sync"));
    assert!(!text.contains("runner-spec-check"));
}

#[test]
fn json_error_output_shape_for_usage_error() {
    let repo = create_temp_git_repo();
    let commit = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("rev-parse");
    assert!(commit.status.success());
    let commit = String::from_utf8_lossy(&commit.stdout).trim().to_string();

    let output = Command::new(cargo_bin("xtask"))
        .args([
            "spec",
            "sync",
            "--tag",
            &commit,
            "--source",
            repo.path().to_str().expect("path"),
            "--json",
        ])
        .current_dir(repo_root())
        .output()
        .expect("run spec sync json");
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("\"ok\":false"));
    assert!(stderr.contains("\"command\":\"xtask\""));
}
