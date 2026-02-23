use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::governance::{
    run_critical_gate_native, run_governance_broad_native, run_governance_heavy_native,
    run_governance_native,
};
use crate::migrators::{
    run_migrate_case_doc_metadata_v1, run_migrate_case_domain_prefix_v1,
    run_migrate_library_docs_metadata_v1,
};
use crate::profiler::{profile_options_from_env, RunProfiler};
use crate::service_registry::{resolve_exact, validate_imports_subset};
use crate::service_runtime::{
    parse_lock, parse_manifest, resolve_implementation_lock_opt_in, validate_lock,
    validate_manifest, verify_local_binary_digest, ServiceImplementationKind,
};
use crate::services::gate_summary::summarize as summarize_gate;
use crate::services::specs_ui;
use crate::spec_lang::{eval_mapping_ast, eval_mapping_ast_with_state, EvalLimits};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use serde_yaml::Value as YamlValue;
use sha2::{Digest, Sha256};

mod dispatch;
mod entry;

static ACTIVE_PROFILER: OnceLock<Mutex<Option<RunProfiler>>> = OnceLock::new();

fn profiler_cell() -> &'static Mutex<Option<RunProfiler>> {
    ACTIVE_PROFILER.get_or_init(|| Mutex::new(None))
}

fn profiler_start_span(
    name: &str,
    kind: &str,
    phase: &str,
    parent_span_id: Option<String>,
    attrs: Value,
) -> Option<String> {
    if let Ok(mut guard) = profiler_cell().lock() {
        if let Some(prof) = guard.as_mut() {
            return prof.start_span(name, kind, phase, parent_span_id, attrs);
        }
    }
    None
}

fn profiler_finish_span(span_id: Option<&str>, status: &str, error: Option<Value>) {
    let Some(sid) = span_id else { return };
    if let Ok(mut guard) = profiler_cell().lock() {
        if let Some(prof) = guard.as_mut() {
            prof.finish_span(sid, status, error);
        }
    }
}

fn profiler_event(kind: &str, span_id: Option<&str>, attrs: Value) {
    if let Ok(mut guard) = profiler_cell().lock() {
        if let Some(prof) = guard.as_mut() {
            prof.event(kind, span_id, attrs);
        }
    }
}

fn debug_enabled() -> bool {
    matches!(
        std::env::var("SPEC_RUNNER_DEBUG").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

fn debug_level() -> u8 {
    if let Ok(raw) = std::env::var("SPEC_RUNNER_DEBUG_LEVEL") {
        if let Ok(parsed) = raw.parse::<u8>() {
            return parsed;
        }
    }
    if debug_enabled() {
        1
    } else {
        0
    }
}

fn debug_log(msg: &str) {
    if debug_level() >= 1 {
        eprintln!("[spec_runner_cli debug] {msg}");
    }
}

fn debug_log_at(level: u8, msg: &str) {
    if debug_level() >= level {
        eprintln!("[spec_runner_cli debug:{level}] {msg}");
    }
}

fn find_repo_root() -> Result<PathBuf, String> {
    let mut cur = env::current_dir().map_err(|e| format!("failed to read cwd: {e}"))?;
    debug_log(&format!("find_repo_root:start cwd={}", cur.display()));
    loop {
        debug_log_at(3, &format!("find_repo_root:check {}", cur.display()));
        if cur.join(".git").exists() {
            debug_log(&format!("find_repo_root:found {}", cur.display()));
            return Ok(cur);
        }
        match cur.parent() {
            Some(parent) => {
                let next = parent.to_path_buf();
                if next == cur {
                    debug_log("find_repo_root:stuck-at-root");
                    return Err("unable to find repository root (.git)".to_string());
                }
                cur = next;
            }
            None => return Err("unable to find repository root (.git)".to_string()),
        }
    }
}

fn run_cmd(program: &str, args: &[String], root: &Path) -> i32 {
    let span_id = profiler_start_span(
        "subprocess.exec",
        "subprocess",
        "subprocess.exec",
        None,
        json!({
            "argv_preview": format!("{} {}", program, args.join(" ")),
            "cwd": root.display().to_string()
        }),
    );
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(root)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit());
    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id();
            profiler_event(
                "subprocess_state",
                span_id.as_deref(),
                json!({"state":"spawned","pid":pid}),
            );
            let code = match child.wait() {
                Ok(status) => status.code().unwrap_or(1),
                Err(e) => {
                    profiler_event(
                        "subprocess_state",
                        span_id.as_deref(),
                        json!({"state":"wait_error","message":e.to_string()}),
                    );
                    eprintln!("ERROR: failed waiting command '{program}': {e}");
                    1
                }
            };
            profiler_event(
                "subprocess_state",
                span_id.as_deref(),
                json!({"state":"exit","pid":pid,"returncode":code}),
            );
            profiler_finish_span(
                span_id.as_deref(),
                if code == 0 { "ok" } else { "error" },
                if code == 0 {
                    None
                } else {
                    Some(json!({"category":"runtime","message":format!("non-zero exit: {code}")}))
                },
            );
            code
        }
        Err(e) => {
            eprintln!("ERROR: failed to run command '{program}': {e}");
            profiler_finish_span(
                span_id.as_deref(),
                "error",
                Some(json!({"category":"runtime","message":e.to_string()})),
            );
            1
        }
    }
}

fn with_forwarded(base: Vec<String>, forwarded: &[String]) -> Vec<String> {
    base.into_iter()
        .chain(forwarded.iter().cloned())
        .collect::<Vec<_>>()
}

fn script(root: &Path, file: &str) -> String {
    root.join("scripts")
        .join(file)
        .to_string_lossy()
        .to_string()
}

fn command_stdout(root: &Path, program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

fn collect_changed_paths(root: &Path) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut push_lines = |raw: &str| {
        for line in raw.lines() {
            let rel = line.trim();
            if rel.is_empty() {
                continue;
            }
            if seen.insert(rel.to_string()) {
                out.push(rel.to_string());
            }
        }
    };

    if let Some(upstream_raw) = command_stdout(
        root,
        "git",
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    ) {
        let upstream = upstream_raw.trim();
        if !upstream.is_empty() {
            let range = format!("{upstream}...HEAD");
            if let Some(diff) =
                command_stdout(root, "git", &["diff", "--name-only", range.as_str()])
            {
                push_lines(&diff);
            }
        }
    }
    if let Some(diff) = command_stdout(root, "git", &["diff", "--name-only"]) {
        push_lines(&diff);
    }
    if let Some(diff_cached) = command_stdout(root, "git", &["diff", "--name-only", "--cached"]) {
        push_lines(&diff_cached);
    }
    if let Some(untracked) =
        command_stdout(root, "git", &["ls-files", "--others", "--exclude-standard"])
    {
        push_lines(&untracked);
    }
    out
}

fn parse_paths_arg(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn normalize_file_text(text: &str) -> String {
    text.replace("- evaluate:\n", "- ")
}

fn run_style_check_native(root: &Path, forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: style-check does not accept extra args");
        return 2;
    }
    let mut violations: Vec<String> = Vec::new();
    let mut stack = vec![root.join("specs")];
    while let Some(cur) = stack.pop() {
        let rd = match fs::read_dir(&cur) {
            Ok(x) => x,
            Err(_) => continue,
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let is_spec = p
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.ends_with(".spec.md"))
                .unwrap_or(false);
            if !is_spec {
                continue;
            }
            let raw = match fs::read_to_string(&p) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if raw.contains("- evaluate:") {
                let rel = p
                    .strip_prefix(root)
                    .map(|x| x.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| p.to_string_lossy().replace('\\', "/"));
                violations.push(format!("{rel}: contains forbidden evaluate wrapper"));
            }
        }
    }
    if violations.is_empty() {
        println!("OK: style-check passed");
        return 0;
    }
    for v in violations {
        eprintln!("ERROR: {v}");
    }
    1
}

fn run_specs_list_native(root: &Path, forwarded: &[String]) -> i32 {
    let mut path: Option<String> = None;
    let mut format = crate::cli::args::OutputFormat::Text;
    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--path" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --path requires value");
                    return 2;
                }
                path = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--format" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --format requires value");
                    return 2;
                }
                format = match forwarded[i + 1].as_str() {
                    "text" => crate::cli::args::OutputFormat::Text,
                    "json" => crate::cli::args::OutputFormat::Json,
                    other => {
                        eprintln!("ERROR: unsupported format: {other} (expected text|json)");
                        return 2;
                    }
                };
                i += 2;
            }
            other => {
                eprintln!("ERROR: unsupported specs list arg: {other}");
                return 2;
            }
        }
    }
    let cases = match specs_ui::list_specs(root, path.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    specs_ui::print_specs(&cases, format);
    0
}

fn run_specs_run_native(root: &Path, forwarded: &[String]) -> i32 {
    if forwarded.len() != 2 || forwarded[0] != "--ref" {
        eprintln!("usage: specs run --ref <file#id>");
        return 2;
    }
    let normalized = specs_ui::normalize_spec_ref(&forwarded[1]);
    run_job_run_native(root, &["--ref".to_string(), normalized])
}

fn run_specs_run_all_native(root: &Path, forwarded: &[String]) -> i32 {
    let mut source_root: Option<String> = None;
    let mut fail_fast = false;
    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--root" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --root requires value");
                    return 2;
                }
                source_root = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--fail-fast" => {
                fail_fast = true;
                i += 1;
            }
            "--continue-on-fail" => {
                fail_fast = false;
                i += 1;
            }
            other => {
                eprintln!("ERROR: unsupported specs run-all arg: {other}");
                return 2;
            }
        }
    }

    let cases = match specs_ui::list_specs(root, source_root.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    if cases.is_empty() {
        println!("No spec cases found.");
        return 0;
    }

    let summary = specs_ui::run_all_specs(&cases, fail_fast, |spec_ref| {
        println!("==> {spec_ref}");
        run_job_run_native(root, &["--ref".to_string(), spec_ref.to_string()])
    });
    println!(
        "specs run-all summary: total={} attempted={} passed={} failed={} skipped={}",
        summary.total, summary.attempted, summary.passed, summary.failed, summary.skipped
    );
    if summary.failed > 0 {
        for failed in summary.failed_refs {
            eprintln!("FAILED: {failed}");
        }
        1
    } else {
        0
    }
}

fn run_specs_check_native(root: &Path, forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: specs check does not accept extra args");
        return 2;
    }
    let style_code = run_style_check_native(root, &[]);
    if style_code != 0 {
        return style_code;
    }
    run_docs_lint_native(root, &[])
}

fn run_help_advanced_native(forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: help-advanced does not accept extra args");
        return 2;
    }
    println!("{}", crate::cli::help::ADVANCED_HELP);
    0
}

fn run_spec_lang_lint_native(root: &Path, forwarded: &[String]) -> i32 {
    if forwarded.is_empty() {
        return run_style_check_native(root, &[]);
    }
    if forwarded.len() == 2 && forwarded[0] == "--cases" {
        return run_style_check_native(root, &[]);
    }
    eprintln!("ERROR: spec-lang-lint accepts only optional '--cases <path>'");
    2
}

fn run_spec_lang_format_native(root: &Path, forwarded: &[String]) -> i32 {
    let mut check_mode = true;
    let mut pass_through: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--check" => {
                check_mode = true;
                i += 1;
            }
            "--write" => {
                check_mode = false;
                i += 1;
            }
            "--cases" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --cases requires value");
                    return 2;
                }
                i += 2;
            }
            "--changed-only" | "--paths" | "--path" => {
                pass_through.push(forwarded[i].clone());
                if (forwarded[i] == "--paths" || forwarded[i] == "--path")
                    && i + 1 < forwarded.len()
                {
                    pass_through.push(forwarded[i + 1].clone());
                    i += 2;
                } else if forwarded[i] == "--changed-only" {
                    i += 1;
                } else {
                    eprintln!("ERROR: {} requires value", forwarded[i]);
                    return 2;
                }
            }
            other => {
                eprintln!("ERROR: unsupported spec-lang-format arg: {other}");
                return 2;
            }
        }
    }
    run_normalize_mode(root, &pass_through, !check_mode)
}

fn run_normalize_mode(root: &Path, forwarded: &[String], fix: bool) -> i32 {
    let mut changed_only = env_bool("SPEC_RUNNER_NORMALIZE_CHANGED_ONLY", false);
    let mut selected_paths: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--changed-only" => {
                changed_only = true;
                i += 1;
            }
            "--paths" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --paths requires value");
                    return 2;
                }
                selected_paths.extend(parse_paths_arg(&forwarded[i + 1]));
                i += 2;
            }
            "--path" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --path requires value");
                    return 2;
                }
                let one = forwarded[i + 1].trim();
                if !one.is_empty() {
                    selected_paths.push(one.to_string());
                }
                i += 2;
            }
            other => {
                eprintln!("ERROR: unsupported normalize arg: {other}");
                return 2;
            }
        }
    }

    if changed_only && selected_paths.is_empty() {
        selected_paths = collect_changed_paths(root);
    }
    if !selected_paths.is_empty() {
        let mut uniq = HashSet::<String>::new();
        selected_paths.retain(|p| uniq.insert(p.clone()));
    }

    if selected_paths.is_empty() && changed_only {
        println!(
            "OK: normalization {} skipped (no changed paths)",
            if fix { "fix" } else { "check" }
        );
        return 0;
    }

    if selected_paths.is_empty() {
        selected_paths.push("specs".to_string());
    }
    let mut violations = 0_i64;
    for rel in selected_paths {
        let path = root.join(rel.trim_start_matches('/'));
        let is_file = path.is_file();
        let mut files: Vec<PathBuf> = Vec::new();
        if is_file {
            files.push(path.clone());
        } else {
            let mut stack = vec![path.clone()];
            while let Some(cur) = stack.pop() {
                let rd = match fs::read_dir(&cur) {
                    Ok(x) => x,
                    Err(_) => continue,
                };
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else if p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.ends_with(".spec.md"))
                        .unwrap_or(false)
                    {
                        files.push(p);
                    }
                }
            }
        }
        for file in files {
            let raw = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let normalized = normalize_file_text(&raw);
            if raw != normalized {
                violations += 1;
                if fix {
                    if let Err(e) = fs::write(&file, normalized) {
                        eprintln!("ERROR: failed writing {}: {e}", file.display());
                        return 1;
                    }
                }
            }
        }
    }
    if violations == 0 {
        println!(
            "OK: normalization {} clean",
            if fix { "fix" } else { "check" }
        );
        return 0;
    }
    if fix {
        println!("OK: normalization fix applied to {violations} file(s)");
        0
    } else {
        eprintln!("ERROR: normalization check failed: {violations} file(s) require fixes");
        1
    }
}

#[derive(Debug, Clone)]
enum ProjectScaffoldMode {
    Canonical {
        bundle_id: String,
        bundle_version: String,
    },
    External {
        bundle_url: String,
        sha256: String,
    },
}

#[derive(Debug, Clone)]
struct ProjectScaffoldArgs {
    project_root: PathBuf,
    mode: ProjectScaffoldMode,
    runner: String,
    overwrite: bool,
}

#[derive(Debug, Serialize)]
struct ProjectBundleLockV1 {
    version: i32,
    project_id: String,
    bundles: Vec<ProjectBundleLockEntry>,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct ProjectBundleLockEntry {
    bundle_id: String,
    bundle_version: String,
    role: String,
    source: ProjectBundleLockSource,
    install_dir: String,
}

#[derive(Debug, Serialize)]
struct ProjectBundleLockSource {
    repo: String,
    release_tag: String,
    asset_url: String,
    sha256: String,
}

fn parse_project_scaffold_args(forwarded: &[String]) -> Result<ProjectScaffoldArgs, String> {
    let mut project_root: Option<PathBuf> = None;
    let mut bundle_id: Option<String> = None;
    let mut bundle_version: Option<String> = None;
    let mut bundle_url: Option<String> = None;
    let mut sha256: Option<String> = None;
    let mut allow_external = false;
    let mut runner: Option<String> = None;
    let mut overwrite = false;

    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--project-root" => {
                if i + 1 >= forwarded.len() {
                    return Err("--project-root requires value".to_string());
                }
                project_root = Some(PathBuf::from(forwarded[i + 1].clone()));
                i += 2;
            }
            "--bundle-id" => {
                if i + 1 >= forwarded.len() {
                    return Err("--bundle-id requires value".to_string());
                }
                bundle_id = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--bundle-version" => {
                if i + 1 >= forwarded.len() {
                    return Err("--bundle-version requires value".to_string());
                }
                bundle_version = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--bundle-url" => {
                if i + 1 >= forwarded.len() {
                    return Err("--bundle-url requires value".to_string());
                }
                bundle_url = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--sha256" => {
                if i + 1 >= forwarded.len() {
                    return Err("--sha256 requires value".to_string());
                }
                sha256 = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--allow-external" => {
                allow_external = true;
                i += 1;
            }
            "--runner" => {
                if i + 1 >= forwarded.len() {
                    return Err("--runner requires value".to_string());
                }
                runner = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--overwrite" => {
                overwrite = true;
                i += 1;
            }
            other => return Err(format!("unsupported project-scaffold arg: {other}")),
        }
    }

    let project_root = project_root.ok_or_else(|| "--project-root is required".to_string())?;
    let runner = runner.unwrap_or_else(|| "rust".to_string());
    if !matches!(runner.as_str(), "rust" | "python" | "php") {
        return Err(format!(
            "invalid --runner value: {} (expected rust|python|php)",
            runner
        ));
    }

    let mode = match (bundle_id, bundle_version, bundle_url, sha256) {
        (Some(id), Some(version), None, None) => {
            if !is_valid_bundle_id(&id) {
                return Err(format!(
                    "invalid --bundle-id `{}` (expected lowercase id token)",
                    id
                ));
            }
            if !is_semver_like(&version) {
                return Err(format!(
                    "invalid --bundle-version `{}` (expected semver-like x.y.z)",
                    version
                ));
            }
            ProjectScaffoldMode::Canonical {
                bundle_id: id,
                bundle_version: version,
            }
        }
        (None, None, Some(url), Some(sum)) => {
            if !allow_external {
                return Err(
                    "external bundle URLs require --allow-external with --bundle-url and --sha256"
                        .to_string(),
                );
            }
            let sum = normalize_sha256_hex(&sum)
                .ok_or_else(|| "invalid --sha256 (expected 64 lowercase/uppercase hex chars)".to_string())?;
            ProjectScaffoldMode::External {
                bundle_url: url,
                sha256: sum,
            }
        }
        _ => {
            return Err(
                "use canonical mode (--bundle-id + --bundle-version) or external mode (--bundle-url + --sha256 + --allow-external)"
                    .to_string(),
            )
        }
    };

    Ok(ProjectScaffoldArgs {
        project_root,
        mode,
        runner,
        overwrite,
    })
}

fn is_valid_bundle_id(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

fn is_semver_like(value: &str) -> bool {
    let mut parts = value.split('.');
    let (Some(major), Some(minor), Some(rest)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    if major.parse::<u64>().is_err() || minor.parse::<u64>().is_err() {
        return false;
    }
    let patch_end = rest
        .find(|c: char| !(c.is_ascii_digit()))
        .unwrap_or(rest.len());
    if patch_end == 0 {
        return false;
    }
    if rest[..patch_end].parse::<u64>().is_err() {
        return false;
    }
    if patch_end == rest.len() {
        return true;
    }
    let suffix = &rest[patch_end..];
    (suffix.starts_with('-') || suffix.starts_with('+'))
        && suffix
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')
}

fn normalize_sha256_hex(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.len() != 64 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

fn read_url_to_string(url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args(["--fail", "--location", "--silent", "--show-error", url])
        .output()
        .map_err(|e| format!("failed to execute curl for {url}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed fetching URL {} (curl exit: {})",
            url,
            output.status.code().unwrap_or(1)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("invalid UTF-8 from {}: {e}", url))
}

fn download_url_to_file(url: &str, out_path: &Path) -> Result<(), String> {
    let status = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--output",
            &out_path.to_string_lossy(),
            url,
        ])
        .status()
        .map_err(|e| format!("failed to execute curl for {url}: {e}"))?;
    if !status.success() {
        return Err(format!(
            "failed downloading URL {} (curl exit: {})",
            url,
            status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

fn parse_sha256_sidecar(raw: &str) -> Result<String, String> {
    for line in raw.lines() {
        let token = line.split_whitespace().next().unwrap_or_default();
        if let Some(sum) = normalize_sha256_hex(token) {
            return Ok(sum);
        }
    }
    Err("invalid .sha256 sidecar: expected 64 hex checksum".to_string())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn canonical_bundle_release(
    bundle_id: &str,
    bundle_version: &str,
) -> (String, String, String, String, String) {
    let asset_name = format!(
        "data-contract-bundle-{}-v{}.tar.gz",
        bundle_id, bundle_version
    );
    let sidecar_name = format!("{}.sha256", asset_name);
    let release_tag = format!("v{}", bundle_version);
    let base = format!(
        "https://github.com/jonruttan/data-contracts-bundles/releases/download/{}",
        release_tag
    );
    let asset_url = format!("{}/{}", base, asset_name);
    let sidecar_url = format!("{}/{}", base, sidecar_name);
    (
        asset_name,
        sidecar_name,
        release_tag,
        asset_url,
        sidecar_url,
    )
}

fn run_bundler_bundle_subcommand(
    root: &Path,
    subcommand: &str,
    project_root: &Path,
    lock_path: &Path,
) -> Result<(), String> {
    let run_and_check = |program: &str, args: &[String]| -> Result<(), String> {
        let status = Command::new(program)
            .args(args)
            .current_dir(root)
            .stdin(process::Stdio::inherit())
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .status()
            .map_err(|e| format!("failed to execute {}: {}", program, e))?;
        if !status.success() {
            return Err(format!(
                "{} exited non-zero for bundle {}",
                program, subcommand
            ));
        }
        Ok(())
    };

    let bundler_args = vec![
        "bundle".to_string(),
        subcommand.to_string(),
        "--project-root".to_string(),
        project_root.to_string_lossy().to_string(),
        "--lock".to_string(),
        lock_path.to_string_lossy().to_string(),
    ];

    if let Ok(bin) = env::var("DATA_CONTRACTS_BUNDLER_BIN") {
        return run_and_check(bin.as_str(), &bundler_args);
    }
    if run_and_check("data-contracts-bundler", &bundler_args).is_ok() {
        return Ok(());
    }

    let manifest_path = root.join("../data-contracts-bundler-rust/Cargo.toml");
    if manifest_path.exists() {
        let mut cargo_args = vec![
            "run".to_string(),
            "--quiet".to_string(),
            "--manifest-path".to_string(),
            manifest_path.to_string_lossy().to_string(),
            "--".to_string(),
        ];
        cargo_args.extend(bundler_args);
        return run_and_check("cargo", &cargo_args);
    }

    Err("unable to execute data-contracts-bundler (set DATA_CONTRACTS_BUNDLER_BIN or install binary)".to_string())
}

pub(super) fn run_project_scaffold_native(root: &Path, forwarded: &[String]) -> i32 {
    if forwarded.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "usage: project scaffold --project-root <path> --bundle-id <id> --bundle-version <semver> [--runner <rust|python|php>] [--overwrite]"
        );
        println!(
            "   or: project scaffold --project-root <path> --bundle-url <url> --sha256 <hex> --allow-external [--runner <rust|python|php>] [--overwrite]"
        );
        return 0;
    }

    let args = match parse_project_scaffold_args(forwarded) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            return 2;
        }
    };

    let project_root = if args.project_root.is_absolute() {
        args.project_root.clone()
    } else {
        root.join(&args.project_root)
    };
    if let Err(e) = fs::create_dir_all(&project_root) {
        eprintln!(
            "ERROR: failed to create project root {}: {}",
            project_root.display(),
            e
        );
        return 1;
    }

    let lock_path = project_root.join("bundles.lock.yaml");
    if lock_path.exists() && !args.overwrite {
        eprintln!(
            "ERROR: bundles.lock.yaml already exists at {} (use --overwrite)",
            lock_path.display()
        );
        return 1;
    }

    let artifacts_dir = project_root.join(".artifacts");
    if let Err(e) = fs::create_dir_all(&artifacts_dir) {
        eprintln!("ERROR: failed to create {}: {}", artifacts_dir.display(), e);
        return 1;
    }

    let (bundle_id, bundle_version, release_tag, asset_url, sha256) = match &args.mode {
        ProjectScaffoldMode::Canonical {
            bundle_id,
            bundle_version,
        } => {
            let (asset_name, _sidecar_name, release_tag, asset_url, sidecar_url) =
                canonical_bundle_release(bundle_id, bundle_version);
            let sidecar_raw = match read_url_to_string(&sidecar_url) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: canonical checksum sidecar fetch failed: {}", e);
                    return 1;
                }
            };
            let sidecar_sha = match parse_sha256_sidecar(&sidecar_raw) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    return 1;
                }
            };

            let cache_dir = artifacts_dir.join("bundle_cache");
            if let Err(e) = fs::create_dir_all(&cache_dir) {
                eprintln!("ERROR: failed to create {}: {}", cache_dir.display(), e);
                return 1;
            }
            let local_tarball = cache_dir.join(asset_name);
            if let Err(e) = download_url_to_file(&asset_url, &local_tarball) {
                eprintln!("ERROR: canonical bundle fetch failed: {}", e);
                return 1;
            }
            let local_sum = match sha256_file(&local_tarball) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    return 1;
                }
            };
            if local_sum != sidecar_sha {
                eprintln!(
                    "ERROR: checksum mismatch for canonical bundle (expected {}, got {})",
                    sidecar_sha, local_sum
                );
                return 1;
            }
            (
                bundle_id.clone(),
                bundle_version.clone(),
                release_tag,
                format!("file://{}", local_tarball.display()),
                sidecar_sha,
            )
        }
        ProjectScaffoldMode::External { bundle_url, sha256 } => {
            let cache_dir = artifacts_dir.join("bundle_cache");
            if let Err(e) = fs::create_dir_all(&cache_dir) {
                eprintln!("ERROR: failed to create {}: {}", cache_dir.display(), e);
                return 1;
            }
            let local_tarball = cache_dir.join("external_bundle.tar.gz");
            if let Err(e) = download_url_to_file(bundle_url, &local_tarball) {
                eprintln!("ERROR: external bundle fetch failed: {}", e);
                return 1;
            }
            let local_sum = match sha256_file(&local_tarball) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    return 1;
                }
            };
            if local_sum != *sha256 {
                eprintln!(
                    "ERROR: checksum mismatch for external bundle (expected {}, got {})",
                    sha256, local_sum
                );
                return 1;
            }
            (
                "external".to_string(),
                "0.0.0".to_string(),
                "external".to_string(),
                format!("file://{}", local_tarball.display()),
                sha256.clone(),
            )
        }
    };

    let install_dir = format!(".bundles/{}", bundle_id);
    let lock = ProjectBundleLockV1 {
        version: 1,
        project_id: project_root
            .file_name()
            .and_then(|v| v.to_str())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or("project")
            .to_string(),
        bundles: vec![ProjectBundleLockEntry {
            bundle_id: bundle_id.clone(),
            bundle_version: bundle_version.clone(),
            role: "primary".to_string(),
            source: ProjectBundleLockSource {
                repo: "jonruttan/data-contracts-bundles".to_string(),
                release_tag,
                asset_url: asset_url.clone(),
                sha256: sha256.clone(),
            },
            install_dir: install_dir.clone(),
        }],
        updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    };

    let lock_yaml = match serde_yaml::to_string(&lock) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: failed to encode bundles.lock.yaml: {}", e);
            return 1;
        }
    };
    if let Err(e) = fs::write(&lock_path, lock_yaml) {
        eprintln!("ERROR: failed writing {}: {}", lock_path.display(), e);
        return 1;
    }

    if let Err(e) = run_bundler_bundle_subcommand(root, "install", &project_root, &lock_path) {
        eprintln!("ERROR: bundle install failed: {}", e);
        return 1;
    }
    if let Err(e) = run_bundler_bundle_subcommand(root, "check", &project_root, &lock_path) {
        eprintln!("ERROR: bundle check failed: {}", e);
        return 1;
    }

    let summary_json_path = artifacts_dir.join("project-scaffold.json");
    let summary_md_path = artifacts_dir.join("project-scaffold.md");
    let payload = json!({
        "version": 1,
        "status": "ok",
        "project_root": project_root.display().to_string(),
        "runner": args.runner,
        "bundle": {
            "id": bundle_id,
            "version": bundle_version,
            "asset_url": asset_url,
            "sha256": sha256
        },
        "lock_path": lock_path.display().to_string(),
        "install_dir": install_dir
    });
    let pretty_json = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
    if let Err(e) = fs::write(&summary_json_path, pretty_json) {
        eprintln!(
            "ERROR: failed writing {}: {}",
            summary_json_path.display(),
            e
        );
        return 1;
    }
    let markdown = format!(
        "# Project Scaffold\n\n- status: `ok`\n- project_root: `{}`\n- lock: `{}`\n- install_dir: `{}`\n",
        project_root.display(),
        lock_path.display(),
        project_root.join(".bundles").display()
    );
    if let Err(e) = fs::write(&summary_md_path, markdown) {
        eprintln!("ERROR: failed writing {}: {}", summary_md_path.display(), e);
        return 1;
    }

    println!(
        "OK: project scaffold complete (lock: {}, install: {})",
        lock_path.display(),
        project_root.join(".bundles").display()
    );
    0
}

pub(super) fn run_service_plugin_check_native(root: &Path, forwarded: &[String]) -> i32 {
    let mut manifest_path: Option<PathBuf> = None;
    let mut lock_path: Option<PathBuf> = None;
    let mut service_type: Option<String> = None;
    let mut profile: Option<String> = None;
    let mut imports_csv: Option<String> = None;

    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--manifest" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --manifest requires value");
                    return 2;
                }
                manifest_path = Some(root.join(&forwarded[i + 1]));
                i += 2;
            }
            "--lock" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --lock requires value");
                    return 2;
                }
                lock_path = Some(root.join(&forwarded[i + 1]));
                i += 2;
            }
            "--service-type" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --service-type requires value");
                    return 2;
                }
                service_type = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--profile" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --profile requires value");
                    return 2;
                }
                profile = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--imports" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --imports requires value");
                    return 2;
                }
                imports_csv = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--help" | "-h" => {
                println!(
                    "usage: service-plugin-check [--manifest <path>] [--lock <path>] [--service-type <io.*>] [--profile <name>] [--imports a,b,c]"
                );
                return 0;
            }
            other => {
                eprintln!("ERROR: unsupported service-plugin-check arg: {other}");
                return 2;
            }
        }
    }

    if manifest_path.is_none() {
        let default = root.join("specs/01_schema/service_plugin_manifest_v1.yaml");
        if default.exists() {
            manifest_path = Some(default);
        }
    }
    if lock_path.is_none() {
        let default = root.join("specs/01_schema/service_plugin_lock_v1.yaml");
        if default.exists() {
            lock_path = Some(default);
        }
    }

    let manifest = if let Some(path) = manifest_path.as_ref() {
        match parse_manifest(path) {
            Ok(manifest) => {
                if let Err(e) = validate_manifest(&manifest) {
                    eprintln!("ERROR: {e}");
                    return 2;
                }
                Some(manifest)
            }
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 2;
            }
        }
    } else {
        None
    };

    let lock = if let Some(path) = lock_path.as_ref() {
        match parse_lock(path) {
            Ok(lock) => {
                if let Err(e) = validate_lock(&lock) {
                    eprintln!("ERROR: {e}");
                    return 2;
                }
                Some(lock)
            }
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 2;
            }
        }
    } else {
        None
    };

    if let (Some(st), Some(pf)) = (service_type.as_ref(), profile.as_ref()) {
        let builtin = match resolve_exact(st, pf) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 2;
            }
        };
        let resolved = match resolve_implementation_lock_opt_in(
            st,
            pf,
            &builtin,
            lock.as_ref(),
            manifest.as_ref(),
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 2;
            }
        };

        let declared_imports: Vec<String> = imports_csv
            .as_ref()
            .map(|raw| {
                raw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let import_check_target = crate::service_registry::ResolvedServiceImplementation {
            implementation_id: resolved.implementation_id.clone(),
            service_type: resolved.service_type.clone(),
            profile: resolved.profile.clone(),
            imports: resolved.imports.clone(),
            packaging_mode: match resolved.kind {
                ServiceImplementationKind::Builtin => "builtin".to_string(),
                ServiceImplementationKind::Plugin => "plugin".to_string(),
            },
        };
        if let Err(e) = validate_imports_subset(&import_check_target, &declared_imports) {
            eprintln!("ERROR: {e}");
            return 2;
        }

        if resolved.kind == ServiceImplementationKind::Plugin {
            let Some(manifest_ref) = manifest.as_ref() else {
                eprintln!("ERROR: runtime.plugin.manifest.required_for_locked_service: {st}/{pf}");
                return 2;
            };
            let Some(digest) = resolved.digest.as_ref() else {
                eprintln!("ERROR: runtime.plugin.lock.digest_required: {st}/{pf}");
                return 2;
            };
            let binary_path = if Path::new(&manifest_ref.binary.path).is_absolute() {
                PathBuf::from(&manifest_ref.binary.path)
            } else if let Some(manifest_file) = manifest_path.as_ref() {
                manifest_file
                    .parent()
                    .unwrap_or(root)
                    .join(&manifest_ref.binary.path)
            } else {
                root.join(&manifest_ref.binary.path)
            };
            if let Err(e) = verify_local_binary_digest(&binary_path, digest) {
                eprintln!("ERROR: {e}");
                return 2;
            }
        }
        println!(
            "OK: service-plugin-check resolved {} {} via {} ({})",
            st,
            pf,
            resolved.implementation_id,
            match resolved.kind {
                ServiceImplementationKind::Builtin => "builtin",
                ServiceImplementationKind::Plugin => "plugin",
            }
        );
        return 0;
    }

    println!("OK: service-plugin-check passed");
    0
}

fn normalize_step_metadata_from_command(root: &Path, command: &[String]) -> (String, Option<i64>) {
    let mut changed_only = env_bool("SPEC_RUNNER_NORMALIZE_CHANGED_ONLY", false);
    let mut selected_paths: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < command.len() {
        match command[i].as_str() {
            "--changed-only" => {
                changed_only = true;
                i += 1;
            }
            "--paths" => {
                if i + 1 < command.len() {
                    changed_only = true;
                    selected_paths.extend(parse_paths_arg(&command[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--path" => {
                if i + 1 < command.len() {
                    changed_only = true;
                    let one = command[i + 1].trim();
                    if !one.is_empty() {
                        selected_paths.push(one.to_string());
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    if changed_only && selected_paths.is_empty() {
        selected_paths = collect_changed_paths(root);
    }
    if changed_only {
        (
            "changed_only".to_string(),
            Some(selected_paths.len() as i64),
        )
    } else {
        ("full_tree".to_string(), None)
    }
}

fn now_iso_utc_fallback() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn env_bool(name: &str, default_value: bool) -> bool {
    let raw = std::env::var(name).unwrap_or_default();
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return default_value;
    }
    if matches!(normalized.as_str(), "1" | "true" | "yes" | "on") {
        return true;
    }
    if matches!(normalized.as_str(), "0" | "false" | "no" | "off") {
        return false;
    }
    default_value
}

fn profile_level_or_off(raw: &str) -> String {
    let lvl = raw.trim().to_ascii_lowercase();
    if matches!(lvl.as_str(), "off" | "basic" | "detailed" | "debug") {
        lvl
    } else {
        "off".to_string()
    }
}

fn run_schema_docs_native(root: &Path, forwarded: &[String], check: bool) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: schema-docs command does not accept extra args");
        return 2;
    }
    let schema_root = root.join("specs").join("01_schema");
    if !schema_root.exists() {
        eprintln!("ERROR: missing schema root: {}", schema_root.display());
        return 1;
    }
    let registry = schema_root.join("registry").join("v1");
    if !registry.exists() {
        eprintln!("ERROR: missing schema registry: {}", registry.display());
        return 1;
    }
    let out = root.join(".artifacts").join("schema-docs-summary.md");
    let mut content = String::new();
    content.push_str("# Schema Docs Summary\n\n");
    content.push_str("- source: `specs/01_schema`\n");
    content.push_str("- status: `ok`\n");
    if check {
        if !out.exists() {
            eprintln!(
                "ERROR: schema-docs check failed: {} is missing",
                out.display()
            );
            return 1;
        }
        return 0;
    }
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&out, content) {
        eprintln!("ERROR: failed writing {}: {e}", out.display());
        return 1;
    }
    println!("OK: schema-docs build wrote {}", out.display());
    0
}

fn run_docs_generate_native(root: &Path, forwarded: &[String], check: bool) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: docs-generate command does not accept extra args");
        return 2;
    }
    let script_path = root.join("scripts").join("docs_generate_all.py");
    if script_path.exists() {
        let mut args = vec![script_path.to_string_lossy().to_string()];
        if check {
            args.push("--check".to_string());
        } else {
            args.push("--build".to_string());
        }
        return run_cmd("python3", &args, root);
    }
    let manifest = root
        .join("docs")
        .join("book")
        .join("reference_manifest.yaml");
    if !manifest.exists() {
        eprintln!(
            "ERROR: docs-generate fallback failed: missing {}",
            manifest.display()
        );
        return 1;
    }
    if check {
        println!(
            "OK: docs-generate-check fallback passed (no docs_generate_all.py; manifest present)"
        );
    } else {
        println!("OK: docs-generate fallback passed (no docs_generate_all.py; manifest present)");
    }
    0
}

fn run_docs_lint_native(root: &Path, forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: docs-lint does not accept extra args");
        return 2;
    }
    let manifest = root
        .join("docs")
        .join("book")
        .join("reference_manifest.yaml");
    let docs_quality_contract = root
        .join("specs")
        .join("contract")
        .join("10_docs_quality.md");
    if !manifest.exists() {
        eprintln!("ERROR: docs-lint failed: missing {}", manifest.display());
        return 1;
    }
    if !docs_quality_contract.exists() {
        eprintln!(
            "ERROR: docs-lint failed: missing {}",
            docs_quality_contract.display()
        );
        return 1;
    }
    println!("OK: docs-lint passed");
    0
}

fn run_lint_native(root: &Path, forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: lint does not accept extra args");
        return 2;
    }
    let status = Command::new("cargo")
        .args([
            "fmt",
            "--manifest-path",
            "runners/rust/spec_runner_cli/Cargo.toml",
            "--all",
            "--",
            "--check",
        ])
        .current_dir(root)
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("ERROR: failed to run cargo fmt: {e}");
            1
        }
    }
}

fn run_typecheck_native(root: &Path, forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: typecheck does not accept extra args");
        return 2;
    }
    let status = Command::new("cargo")
        .args([
            "check",
            "--manifest-path",
            "runners/rust/spec_runner_cli/Cargo.toml",
        ])
        .current_dir(root)
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("ERROR: failed to run cargo check: {e}");
            1
        }
    }
}

fn run_compilecheck_native(root: &Path, forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: compilecheck does not accept extra args");
        return 2;
    }
    let status = Command::new("cargo")
        .args([
            "check",
            "--all-targets",
            "--manifest-path",
            "runners/rust/spec_runner_cli/Cargo.toml",
        ])
        .current_dir(root)
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("ERROR: failed to run cargo check --all-targets: {e}");
            1
        }
    }
}

fn run_tests_native(root: &Path, forwarded: &[String]) -> i32 {
    if !forwarded.is_empty() {
        eprintln!("ERROR: test command does not accept extra args");
        return 2;
    }
    let status = Command::new("cargo")
        .args([
            "test",
            "--manifest-path",
            "runners/rust/spec_runner_cli/Cargo.toml",
        ])
        .current_dir(root)
        .status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("ERROR: failed to run cargo test: {e}");
            1
        }
    }
}

fn yaml_map_get<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a YamlValue> {
    map.get(&YamlValue::String(key.to_string()))
}

fn yaml_as_non_empty_string(node: Option<&YamlValue>, field: &str) -> Result<String, String> {
    let Some(value) = node else {
        return Err(format!("missing required field: {field}"));
    };
    let Some(raw) = value.as_str() else {
        return Err(format!("invalid field type for {field}: expected string"));
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("invalid empty field: {field}"));
    }
    Ok(trimmed.to_string())
}

fn yaml_as_string_list(node: Option<&YamlValue>, field: &str) -> Result<Vec<String>, String> {
    let Some(value) = node else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_sequence() else {
        return Err(format!("invalid field type for {field}: expected list"));
    };
    let mut out = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        let Some(raw) = item.as_str() else {
            return Err(format!("invalid {field}[{idx}]: expected string"));
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(format!("invalid {field}[{idx}]: empty string"));
        }
        out.push(trimmed.to_string());
    }
    Ok(out)
}

fn canonicalize_json_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(items) => {
            let mut out = String::from("[");
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push_str(&canonicalize_json_value(item));
            }
            out.push(']');
            out
        }
        Value::Object(map) => {
            let mut keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            keys.sort_unstable();
            let mut out = String::from("{");
            for (idx, key) in keys.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string()));
                out.push(':');
                if let Some(v) = map.get(*key) {
                    out.push_str(&canonicalize_json_value(v));
                } else {
                    out.push_str("null");
                }
            }
            out.push('}');
            out
        }
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn git_commit_sha(root: &Path) -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(root)
        .output();
    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    }
}

fn run_cert_command(root: &Path, command: &str, args: &[String]) -> i32 {
    match command {
        "governance" => run_governance_native(root, args),
        "governance-heavy" => run_governance_heavy_native(root, args),
        "style-check" => run_style_check_native(root, args),
        "spec-lang-lint" => run_spec_lang_lint_native(root, args),
        "spec-lang-format" => run_spec_lang_format_native(root, args),
        "normalize-check" => run_normalize_mode(root, args, false),
        "normalize-fix" => run_normalize_mode(root, args, true),
        "schema-docs-check" => run_schema_docs_native(root, args, true),
        "schema-docs-build" => run_schema_docs_native(root, args, false),
        "lint" => run_lint_native(root, args),
        "typecheck" => run_typecheck_native(root, args),
        "compilecheck" => run_compilecheck_native(root, args),
        "test-core" => run_tests_native(root, args),
        "test-full" => run_tests_native(root, args),
        "docs-generate-check" => run_docs_generate_native(root, args, true),
        "docs-generate" => run_docs_generate_native(root, args, false),
        "conformance-parity" => run_job_for_command(root, "conformance-parity", args),
        "perf-smoke" => run_job_for_command(root, "perf-smoke", args),
        "schema-registry-check" => run_job_for_command(root, "schema-registry-check", args),
        "schema-registry-build" => run_job_for_command(root, "schema-registry-build", args),
        "ci-gate-summary" => run_ci_gate_summary_native(root, args),
        _ => 2,
    }
}

fn run_runner_certify_native(root: &Path, forwarded: &[String]) -> i32 {
    let mut runner_id = String::new();
    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--runner" => {
                i += 1;
                if i >= forwarded.len() {
                    eprintln!("ERROR: --runner requires a value");
                    return 2;
                }
                runner_id = forwarded[i].trim().to_string();
            }
            "--help" | "-h" => {
                println!("usage: runner-certify --runner <id>");
                return 0;
            }
            other => {
                eprintln!("ERROR: unsupported argument for runner-certify: {other}");
                return 2;
            }
        }
        i += 1;
    }
    if runner_id.is_empty() {
        eprintln!("ERROR: runner-certify requires --runner <id>");
        return 2;
    }

    let registry_candidates = [
        root.join("specs/01_schema/runner_certification_registry_v1.yaml"),
        root.join(
            "specs/upstream/data-contracts/specs/01_schema/runner_certification_registry_v1.yaml",
        ),
    ];
    let mut registry_path: Option<PathBuf> = None;
    let mut registry_text = String::new();
    for candidate in registry_candidates {
        if candidate.exists() {
            match fs::read_to_string(&candidate) {
                Ok(v) => {
                    registry_path = Some(candidate);
                    registry_text = v;
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "ERROR: failed to read runner certification registry {}: {e}",
                        candidate.display()
                    );
                    return 2;
                }
            }
        }
    }
    let Some(registry_path) = registry_path else {
        eprintln!("ERROR: runner certification registry v1 not found");
        return 2;
    };
    let registry_yaml: YamlValue = match serde_yaml::from_str(&registry_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "ERROR: failed to parse runner certification registry {}: {e}",
                registry_path.display()
            );
            return 2;
        }
    };
    let registry_map = match registry_yaml.as_mapping() {
        Some(m) => m,
        None => {
            eprintln!("ERROR: invalid registry root; expected mapping");
            return 2;
        }
    };
    let registry_version = yaml_map_get(registry_map, "version")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    if registry_version != 1 {
        eprintln!(
            "ERROR: unsupported runner certification registry version {}; expected 1",
            registry_version
        );
        return 2;
    }
    let runners = match yaml_map_get(registry_map, "runners").and_then(|v| v.as_sequence()) {
        Some(v) => v,
        None => {
            eprintln!("ERROR: runner certification registry missing runners list");
            return 2;
        }
    };
    let mut selected: Option<&serde_yaml::Mapping> = None;
    for item in runners {
        if let Some(map) = item.as_mapping() {
            let rid = yaml_map_get(map, "runner_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if rid == runner_id {
                selected = Some(map);
                break;
            }
        }
    }
    let Some(entry) = selected else {
        eprintln!("ERROR: unknown runner id for certification: {runner_id}");
        return 2;
    };

    let runner_class = match yaml_as_non_empty_string(yaml_map_get(entry, "class"), "class") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: invalid runner registry entry for {runner_id}: {e}");
            return 2;
        }
    };
    if runner_class != "required" && runner_class != "compatibility_non_blocking" {
        eprintln!(
            "ERROR: invalid runner class for {}: {} (expected required|compatibility_non_blocking)",
            runner_id, runner_class
        );
        return 2;
    }
    let runner_status = match yaml_as_non_empty_string(yaml_map_get(entry, "status"), "status") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: invalid runner registry entry for {runner_id}: {e}");
            return 2;
        }
    };
    if runner_status != "active"
        && runner_status != "planned"
        && runner_status != "retired"
        && runner_status != "disabled"
    {
        eprintln!(
            "ERROR: invalid runner status for {}: {} (expected active|planned|retired|disabled)",
            runner_id, runner_status
        );
        return 2;
    }
    let implementation_repo = match yaml_map_get(entry, "entrypoints")
        .and_then(|v| v.as_mapping())
        .and_then(|m| yaml_map_get(m, "implementation_repo"))
        .and_then(|v| v.as_str())
    {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => {
            eprintln!("ERROR: invalid runner registry entry for {runner_id}: missing entrypoints.implementation_repo");
            return 2;
        }
    };

    let required_core_checks = match yaml_as_string_list(
        yaml_map_get(entry, "required_core_checks"),
        "required_core_checks",
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: invalid required_core_checks for {runner_id}: {e}");
            return 2;
        }
    };
    let required_core_cases = match yaml_as_string_list(
        yaml_map_get(entry, "required_core_cases"),
        "required_core_cases",
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: invalid required_core_cases for {runner_id}: {e}");
            return 2;
        }
    };

    let command_items = yaml_map_get(entry, "command_contract_subset")
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_default();
    let mut command_specs: Vec<(String, Vec<String>, Vec<i32>)> = Vec::new();
    for (idx, item) in command_items.iter().enumerate() {
        let Some(map) = item.as_mapping() else {
            eprintln!(
                "ERROR: invalid command_contract_subset[{idx}] for {runner_id}: expected mapping"
            );
            return 2;
        };
        let name = match yaml_as_non_empty_string(
            yaml_map_get(map, "name"),
            "command_contract_subset[].name",
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: invalid command_contract_subset[{idx}] for {runner_id}: {e}");
                return 2;
            }
        };
        let args = match yaml_as_string_list(
            yaml_map_get(map, "args"),
            "command_contract_subset[].args",
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: invalid command args for {runner_id}/{name}: {e}");
                return 2;
            }
        };
        let expect_exit = match yaml_map_get(map, "expect_exit").and_then(|v| v.as_sequence()) {
            Some(seq) => {
                let mut exits = Vec::new();
                for (eidx, ev) in seq.iter().enumerate() {
                    let Some(code) = ev.as_i64() else {
                        eprintln!("ERROR: invalid expect_exit[{eidx}] for {runner_id}/{name}: expected integer");
                        return 2;
                    };
                    exits.push(code as i32);
                }
                if exits.is_empty() {
                    vec![0]
                } else {
                    exits
                }
            }
            None => vec![0],
        };
        command_specs.push((name, args, expect_exit));
    }

    let artifact_contract =
        match yaml_map_get(entry, "artifact_contract").and_then(|v| v.as_mapping()) {
            Some(v) => v,
            None => {
                eprintln!("ERROR: missing artifact_contract for runner {runner_id}");
                return 2;
            }
        };
    let json_out_pat = match yaml_as_non_empty_string(
        yaml_map_get(artifact_contract, "json_out"),
        "artifact_contract.json_out",
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: invalid artifact contract for {runner_id}: {e}");
            return 2;
        }
    };
    let md_out_pat = match yaml_as_non_empty_string(
        yaml_map_get(artifact_contract, "md_out"),
        "artifact_contract.md_out",
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: invalid artifact contract for {runner_id}: {e}");
            return 2;
        }
    };
    let json_out_rel = json_out_pat
        .replace("{runner}", &runner_id)
        .trim_start_matches('/')
        .to_string();
    let md_out_rel = md_out_pat
        .replace("{runner}", &runner_id)
        .trim_start_matches('/')
        .to_string();
    if json_out_rel.is_empty() || md_out_rel.is_empty() {
        eprintln!(
            "ERROR: invalid artifact contract paths after substitution for runner {runner_id}"
        );
        return 2;
    }
    let json_out_path = root.join(&json_out_rel);
    let md_out_path = root.join(&md_out_rel);
    if let Some(parent) = json_out_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "ERROR: failed to create artifact directory {}: {e}",
                parent.display()
            );
            return 1;
        }
    }
    if let Some(parent) = md_out_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "ERROR: failed to create artifact directory {}: {e}",
                parent.display()
            );
            return 1;
        }
    }

    let mut check_results: Vec<Value> = Vec::new();
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let blocking = runner_class == "required" && runner_status == "active";

    let mut push_result = |group: &str, id: &str, status: &str, exit_code: i32, detail: &str| {
        match status {
            "pass" => passed += 1,
            "fail" => failed += 1,
            _ => skipped += 1,
        }
        check_results.push(json!({
            "group": group,
            "id": id,
            "status": status,
            "exit_code": exit_code,
            "detail": detail,
        }));
    };

    push_result(
        "contract",
        "registry.entry.shape",
        "pass",
        0,
        "runner certification registry entry parsed and validated",
    );

    if runner_status != "active" {
        push_result(
            "command",
            "command.subset",
            "skipped",
            0,
            "runner status is not active; command subset execution skipped",
        );
    } else if runner_id != "rust" {
        push_result(
            "command",
            "command.subset",
            "skipped",
            0,
            "non-rust lane certification is metadata-only in rust command path",
        );
    } else {
        for (name, args, expected_exits) in &command_specs {
            let code = run_cert_command(root, name, args);
            if expected_exits.contains(&code) {
                push_result(
                    "command",
                    &format!("command.{name}"),
                    "pass",
                    code,
                    "command contract subset matched expected exit semantics",
                );
            } else {
                push_result(
                    "command",
                    &format!("command.{name}"),
                    "fail",
                    code,
                    &format!("unexpected exit code; expected one of {:?}", expected_exits),
                );
            }
        }
    }

    if runner_status != "active" {
        push_result(
            "governance-sync",
            "governance.required_core_checks",
            "skipped",
            0,
            "runner status is not active; governance sync checks skipped",
        );
    } else if runner_id != "rust" {
        push_result(
            "governance-sync",
            "governance.required_core_checks",
            "skipped",
            0,
            "non-rust lane governance checks are non-blocking and not executed here",
        );
    } else {
        for check_id in &required_core_checks {
            let args = vec!["--check-id".to_string(), check_id.to_string()];
            let code = run_governance_native(root, &args);
            if code == 0 {
                push_result(
                    "governance-sync",
                    &format!("governance.{check_id}"),
                    "pass",
                    code,
                    "governance check passed",
                );
            } else {
                push_result(
                    "governance-sync",
                    &format!("governance.{check_id}"),
                    "fail",
                    code,
                    "governance check failed",
                );
            }
        }
    }

    if runner_status != "active" {
        push_result(
            "conformance",
            "conformance.required_core_cases",
            "skipped",
            0,
            "runner status is not active; conformance subset skipped",
        );
    } else if runner_id != "rust" {
        push_result(
            "conformance",
            "conformance.required_core_cases",
            "skipped",
            0,
            "non-rust lane conformance checks are non-blocking and not executed here",
        );
    } else {
        for spec_ref in &required_core_cases {
            let (path_part, anchor_part) = match parse_spec_ref(spec_ref) {
                Ok(v) => v,
                Err(e) => {
                    push_result(
                        "conformance",
                        &format!("spec.{spec_ref}"),
                        "fail",
                        1,
                        &format!("invalid spec ref: {e}"),
                    );
                    continue;
                }
            };
            let spec_path = root.join(path_part.trim_start_matches('/'));
            if !spec_path.exists() {
                push_result(
                    "conformance",
                    &format!("spec.{spec_ref}"),
                    "fail",
                    1,
                    "required core case file not found",
                );
                continue;
            }
            let text = match fs::read_to_string(&spec_path) {
                Ok(v) => v,
                Err(e) => {
                    push_result(
                        "conformance",
                        &format!("spec.{spec_ref}"),
                        "fail",
                        1,
                        &format!("failed to read core case file: {e}"),
                    );
                    continue;
                }
            };
            let anchor = anchor_part.unwrap_or_default();
            if !anchor.is_empty() && !text.contains(&format!("id: {anchor}")) {
                push_result(
                    "conformance",
                    &format!("spec.{spec_ref}"),
                    "fail",
                    1,
                    "required core case id not found in case file",
                );
                continue;
            }
            if anchor.is_empty() {
                push_result(
                    "conformance",
                    &format!("spec.{spec_ref}"),
                    "skipped",
                    0,
                    "core case ref has no anchor; skipped executable certification check",
                );
                continue;
            }
            push_result(
                "conformance",
                &format!("spec.{spec_ref}"),
                "pass",
                0,
                "required core case ref resolved",
            );
        }
    }

    let status = if failed == 0 { "pass" } else { "fail" };

    let mut required_core_checks_norm = required_core_checks.clone();
    required_core_checks_norm.sort();
    required_core_checks_norm.dedup();

    let mut required_core_cases_norm = required_core_cases.clone();
    required_core_cases_norm.sort();
    required_core_cases_norm.dedup();

    let mut command_contract_subset_norm: Vec<Value> = command_specs
        .iter()
        .map(|(name, args, expect_exit)| {
            let mut exits = expect_exit.clone();
            exits.sort_unstable();
            exits.dedup();
            json!({
                "name": name,
                "args": args,
                "expect_exit": exits,
            })
        })
        .collect();
    command_contract_subset_norm.sort_by(|a, b| {
        let an = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if an != bn {
            return an.cmp(bn);
        }
        let aa = a
            .get("args")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>()
                    .join("\u{1f}")
            })
            .unwrap_or_default();
        let ba = b
            .get("args")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>()
                    .join("\u{1f}")
            })
            .unwrap_or_default();
        if aa != ba {
            return aa.cmp(&ba);
        }
        let ae = a
            .get("expect_exit")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_i64())
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            })
            .unwrap_or_default();
        let be = b
            .get("expect_exit")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_i64())
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            })
            .unwrap_or_default();
        ae.cmp(&be)
    });

    let execution_intent = json!({
        "required_core_checks": required_core_checks_norm,
        "required_core_cases": required_core_cases_norm,
        "command_contract_subset": command_contract_subset_norm,
        "registry_ref": {
            "path": "/specs/01_schema/runner_certification_registry_v1.yaml",
            "version": 1
        }
    });
    let intent_hash = sha256_hex(&canonicalize_json_value(&execution_intent));

    let commit_sha = git_commit_sha(root);
    let certified_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let mut payload = json!({
        "version": 2,
        "runner": {
            "runner_id": runner_id,
            "class": runner_class,
            "status": runner_status,
            "blocking": blocking,
            "implementation_repo": implementation_repo,
            "commit_sha": commit_sha,
            "certified_at": certified_at
        },
        "execution_intent": execution_intent,
        "equivalence": {
            "normalization_version": "intent-v1",
            "hash_algorithm": "sha256",
            "intent_hash": intent_hash
        },
        "summary": {
            "status": status,
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "blocking": blocking
        },
        "checks": check_results,
        "proof": {
            "canonicalization": "json-c14n-v1",
            "payload_sha256": ""
        }
    });
    let payload_hash = sha256_hex(&canonicalize_json_value(&payload));
    if let Some(proof) = payload.get_mut("proof").and_then(|v| v.as_object_mut()) {
        proof.insert("payload_sha256".to_string(), Value::String(payload_hash));
    }

    if let Err(e) = fs::write(
        &json_out_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        ),
    ) {
        eprintln!(
            "ERROR: failed writing certification JSON artifact {}: {e}",
            json_out_path.display()
        );
        return 1;
    }

    let mut md = String::new();
    md.push_str("# Runner Certification Report\n\n");
    md.push_str(&format!("- version: `{}`\n", payload["version"]));
    md.push_str(&format!(
        "- runner_id: `{}`\n",
        payload["runner"]["runner_id"]
    ));
    md.push_str(&format!("- class: `{}`\n", payload["runner"]["class"]));
    md.push_str(&format!("- status: `{}`\n", payload["runner"]["status"]));
    md.push_str(&format!(
        "- blocking: `{}`\n",
        payload["runner"]["blocking"]
    ));
    md.push_str(&format!(
        "- implementation_repo: `{}`\n",
        payload["runner"]["implementation_repo"]
    ));
    md.push_str(&format!(
        "- commit_sha: `{}`\n",
        payload["runner"]["commit_sha"]
    ));
    md.push_str(&format!(
        "- certified_at: `{}`\n",
        payload["runner"]["certified_at"]
    ));
    md.push_str(&format!(
        "- equivalence.intent_hash: `{}`\n",
        payload["equivalence"]["intent_hash"]
    ));
    md.push_str(&format!(
        "- proof.payload_sha256: `{}`\n",
        payload["proof"]["payload_sha256"]
    ));
    md.push_str(&format!(
        "- summary.status: `{}`\n",
        payload["summary"]["status"]
    ));
    md.push_str(&format!(
        "- summary.passed: `{}`\n",
        payload["summary"]["passed"]
    ));
    md.push_str(&format!(
        "- summary.failed: `{}`\n",
        payload["summary"]["failed"]
    ));
    md.push_str(&format!(
        "- summary.skipped: `{}`\n\n",
        payload["summary"]["skipped"]
    ));
    md.push_str("## Checks\n\n");
    md.push_str("| group | id | status | exit_code | detail |\n");
    md.push_str("|---|---|---:|---:|---|\n");
    if let Some(rows) = payload.get("checks").and_then(|v| v.as_array()) {
        for row in rows {
            let group = row.get("group").and_then(|v| v.as_str()).unwrap_or("");
            let id = row.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let row_status = row.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let exit_code = row.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(0);
            let detail = row.get("detail").and_then(|v| v.as_str()).unwrap_or("");
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                group, id, row_status, exit_code, detail
            ));
        }
    }
    if let Err(e) = fs::write(&md_out_path, md) {
        eprintln!(
            "ERROR: failed writing certification markdown artifact {}: {e}",
            md_out_path.display()
        );
        return 1;
    }

    println!(
        "OK: runner certification report written: {}",
        json_out_path.display()
    );
    println!(
        "OK: runner certification report written: {}",
        md_out_path.display()
    );
    if failed == 0 {
        0
    } else {
        1
    }
}

fn command_spec_ref(subcommand: &str) -> Option<&'static str> {
    crate::domain::command_map::command_spec_ref(subcommand)
}

fn parse_spec_ref(spec_ref: &str) -> Result<(String, Option<String>), String> {
    crate::domain::refs::parse_spec_ref(spec_ref)
}

fn parse_job_ref(job_ref: &str) -> Result<(Option<String>, String), String> {
    crate::domain::refs::parse_job_ref(job_ref)
}

fn extract_spec_test_blocks(markdown: &str) -> Vec<String> {
    crate::domain::refs::extract_spec_test_blocks(markdown)
}

fn block_id(block: &str) -> Option<String> {
    crate::domain::refs::block_id(block)
}

fn enforce_runner_spec_boundary(path: &Path, case_map: &serde_yaml::Mapping) -> Result<(), String> {
    let path_text = path.to_string_lossy();
    if !path_text.contains("dc-runner-spec") {
        return Ok(());
    }

    let expected_schema_ref = "/specs/01_schema/schema_v1.md";
    let schema_ref = case_map
        .get(&YamlValue::String("schema_ref".to_string()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "dc-runner-spec cases must set `schema_ref: {expected_schema_ref}` in {}",
                path.display()
            )
        })?;
    if schema_ref != expected_schema_ref {
        return Err(format!(
            "dc-runner-spec cases must set canonical schema authority via `schema_ref: {expected_schema_ref}` (found `{schema_ref}` in {})",
            path.display()
        ));
    }

    let spec_version = case_map
        .get(&YamlValue::String("spec_version".to_string()))
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            format!(
                "dc-runner-spec cases must set `spec_version: 1` in {}",
                path.display()
            )
        })?;
    if spec_version != 1 {
        return Err(format!(
            "dc-runner-spec cases must set `spec_version: 1` (found `{spec_version}` in {})",
            path.display()
        ));
    }

    Ok(())
}

fn load_case_block_from_spec_ref(root: &Path, spec_ref: &str) -> Result<String, String> {
    let (path_raw, case_id) = parse_spec_ref(spec_ref)?;
    let rel = path_raw.trim_start_matches('/');
    let path = root.join(rel);
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read producer spec {}: {e}", path.display()))?;
    let blocks = extract_spec_test_blocks(&text);
    if blocks.is_empty() {
        return Err(format!(
            "no `yaml contract-spec` blocks in {}",
            path.display()
        ));
    }
    for block in blocks {
        let parsed: YamlValue = serde_yaml::from_str(&block).map_err(|e| {
            format!(
                "failed to parse contract-spec block in {}: {e}",
                path.display()
            )
        })?;
        if let Some(map) = parsed.as_mapping() {
            if map.contains_key(&YamlValue::String("contracts".to_string())) {
                let suite_defaults = map
                    .get(&YamlValue::String("defaults".to_string()))
                    .and_then(|v| v.as_mapping())
                    .cloned()
                    .unwrap_or_default();
                let suite_spec_version = map
                    .get(&YamlValue::String("spec_version".to_string()))
                    .cloned();
                let suite_schema_ref = map
                    .get(&YamlValue::String("schema_ref".to_string()))
                    .cloned();
                let suite_title = map.get(&YamlValue::String("title".to_string())).cloned();
                let suite_purpose = map.get(&YamlValue::String("purpose".to_string())).cloned();
                let suite_domain = map.get(&YamlValue::String("domain".to_string())).cloned();
                let contracts_val = map.get(&YamlValue::String("contracts".to_string()));
                let contracts: Vec<&YamlValue> =
                    if let Some(seq) = contracts_val.and_then(|v| v.as_sequence()) {
                        seq.iter().collect()
                    } else if let Some(contracts_map) = contracts_val.and_then(|v| v.as_mapping()) {
                        if let Some(seq) = contracts_map
                            .get(&YamlValue::String("clauses".to_string()))
                            .and_then(|v| v.as_sequence())
                        {
                            seq.iter().collect()
                        } else if let Some(seq) = contracts_map
                            .get(&YamlValue::String("asserts".to_string()))
                            .and_then(|v| v.as_sequence())
                        {
                            seq.iter().collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                if contracts.is_empty() {
                    return Err(format!(
                        "contract-spec suite in {} must include non-empty contracts list or contracts.clauses list",
                        path.display()
                    ));
                }
                let mut selected: Option<serde_yaml::Mapping> = None;
                for item in contracts {
                    let Some(item_map) = item.as_mapping() else {
                        continue;
                    };
                    let item_id = item_map
                        .get(&YamlValue::String("id".to_string()))
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    if let Some(want) = &case_id {
                        if item_id.as_deref() != Some(want.as_str()) {
                            continue;
                        }
                    } else if selected.is_some() {
                        continue;
                    }
                    let mut merged = suite_defaults.clone();
                    for (k, v) in item_map {
                        merged.insert(k.clone(), v.clone());
                    }
                    // Carry suite-level runtime context into extracted clause docs.
                    for suite_key in [
                        "harness",
                        "artifacts",
                        "adapters",
                        "services",
                        "exports",
                        "when",
                        "library",
                        "expect",
                    ] {
                        let k = YamlValue::String(suite_key.to_string());
                        if !merged.contains_key(&k) {
                            if let Some(v) = map.get(&k).cloned() {
                                merged.insert(k, v);
                            }
                        }
                    }
                    if !merged.contains_key(&YamlValue::String("spec_version".to_string())) {
                        if let Some(v) = suite_spec_version.clone() {
                            merged.insert(YamlValue::String("spec_version".to_string()), v);
                        }
                    }
                    if !merged.contains_key(&YamlValue::String("schema_ref".to_string())) {
                        if let Some(v) = suite_schema_ref.clone() {
                            merged.insert(YamlValue::String("schema_ref".to_string()), v);
                        }
                    }
                    if !merged.contains_key(&YamlValue::String("title".to_string())) {
                        if let Some(v) = suite_title.clone() {
                            merged.insert(YamlValue::String("title".to_string()), v);
                        }
                    }
                    if !merged.contains_key(&YamlValue::String("purpose".to_string())) {
                        if let Some(v) = suite_purpose.clone() {
                            merged.insert(YamlValue::String("purpose".to_string()), v);
                        }
                    }
                    if !merged.contains_key(&YamlValue::String("domain".to_string())) {
                        if let Some(v) = suite_domain.clone() {
                            merged.insert(YamlValue::String("domain".to_string()), v);
                        }
                    }
                    selected = Some(merged);
                    if case_id.is_some() {
                        break;
                    }
                }
                if let Some(found) = selected {
                    enforce_runner_spec_boundary(&path, &found)?;
                    let rendered =
                        serde_yaml::to_string(&YamlValue::Mapping(found)).map_err(|e| {
                            format!(
                                "failed to render expanded suite case in {}: {e}",
                                path.display()
                            )
                        })?;
                    return Ok(rendered);
                }
                continue;
            }
        }
        if let Some(want) = &case_id {
            if block_id(&block).as_deref() != Some(want.as_str()) {
                continue;
            }
        }
        if let Some(map) = parsed.as_mapping() {
            enforce_runner_spec_boundary(&path, map)?;
        }
        return Ok(block);
    }
    Err(format!("case not found via spec ref: {}", spec_ref))
}

fn resolve_job_case_block(
    root: &Path,
    job_ref: &str,
    doc_ref: Option<&str>,
) -> Result<(String, String), String> {
    let (path_opt, case_id) = parse_job_ref(job_ref)?;
    let full_ref = if let Some(path) = path_opt {
        format!("{path}#{case_id}")
    } else if let Some(doc) = doc_ref {
        let (path, _frag) = parse_spec_ref(doc)?;
        format!("{path}#{case_id}")
    } else {
        return Err(
            "same-document job ref (#CASE) requires --doc <path#id> or SPEC_RUNNER_JOB_DOC"
                .to_string(),
        );
    };
    let block = load_case_block_from_spec_ref(root, &full_ref)?;
    Ok((full_ref, block))
}

fn json_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|x| x != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_job_hook_event(
    hook_exprs: &HashMap<String, Vec<Value>>,
    event: &str,
    step_idx: usize,
    step_id: Option<&str>,
    requirement: &str,
    assert_path: &str,
    target: Option<&str>,
    status: &str,
    failure_message: Option<&str>,
    passed_clauses: i64,
    failed_clauses: i64,
    required_passed: i64,
    optional_passed: i64,
    summary_json: &mut Value,
    case_id: &str,
    case_type: &str,
    case_doc_path: &str,
) -> Result<(), String> {
    let Some(exprs) = hook_exprs.get(event) else {
        return Ok(());
    };
    let subject = json!({
        "event": event,
        "case": {
            "id": case_id,
            "type": case_type,
            "doc_path": case_doc_path,
        },
        "clause": {
            "index": step_idx,
            "id": step_id,
            "requirement": requirement,
            "required": requirement == "required",
            "assert_path": assert_path,
            "target": target,
        },
        "runtime": {
            "impl": std::env::var("SPEC_RUNNER_IMPL").unwrap_or_else(|_| "unknown".to_string()),
            "profile_enabled": false,
        },
        "status": status,
        "failure": {
            "message": failure_message,
            "token": failure_message
                .and_then(|m| m.split(':').next())
                .map(|x| x.trim().to_string())
                .filter(|x| x.contains('.')),
        },
        "totals": {
            "passed_clauses": passed_clauses,
            "failed_clauses": failed_clauses,
            "required_passed": required_passed,
            "optional_passed": optional_passed,
        }
    });
    for (hook_idx, expr) in exprs.iter().enumerate() {
        let result = eval_mapping_ast_with_state(
            expr,
            subject.clone(),
            HashMap::new(),
            EvalLimits::default(),
        )
        .map_err(|e| {
            format!(
                "runtime.on_hook.failed: event={event} index={hook_idx}: {}",
                e.message
            )
        })?;
        if let Some(dispatched) = result.last_dispatch_result.clone() {
            *summary_json = dispatched;
        }
        if !json_truthy(&result.value) {
            return Err(format!(
                "runtime.on_hook.failed: event={event} index={hook_idx}: expression returned falsy"
            ));
        }
    }
    Ok(())
}

fn run_job_run_native(root: &Path, forwarded: &[String]) -> i32 {
    let mut ref_arg = String::new();
    let mut doc_arg: Option<String> = std::env::var("SPEC_RUNNER_JOB_DOC").ok();
    let mut input_pairs = Vec::<(String, String)>::new();
    let mut i = 0usize;
    while i < forwarded.len() {
        match forwarded[i].as_str() {
            "--ref" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --ref requires value");
                    return 2;
                }
                ref_arg = forwarded[i + 1].clone();
                i += 2;
            }
            "--doc" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --doc requires value");
                    return 2;
                }
                doc_arg = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--input" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --input requires key=value");
                    return 2;
                }
                let raw = forwarded[i + 1].clone();
                let mut parts = raw.splitn(2, '=');
                let k = parts.next().unwrap_or("").trim().to_string();
                let v = parts.next().unwrap_or("").to_string();
                if k.is_empty() {
                    eprintln!("ERROR: --input requires key=value");
                    return 2;
                }
                input_pairs.push((k, v));
                i += 2;
            }
            other => {
                eprintln!("ERROR: unsupported job-run arg: {other}");
                return 2;
            }
        }
    }
    if ref_arg.trim().is_empty() {
        eprintln!(
            "ERROR: job-run requires --ref <path#id|#id> (example: /specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-001)"
        );
        return 2;
    }

    let (resolved_ref, case_block) =
        match resolve_job_case_block(root, &ref_arg, doc_arg.as_deref()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 1;
            }
        };
    let doc: YamlValue = match serde_yaml::from_str(&case_block) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: failed to parse job case yaml: {e}");
            return 1;
        }
    };
    let case_map = match doc.as_mapping() {
        Some(m) => m,
        None => {
            eprintln!("ERROR: job case must be yaml mapping");
            return 1;
        }
    };
    let case_type = case_map
        .get(&YamlValue::String("type".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("contract.job");
    let case_id = case_map
        .get(&YamlValue::String("id".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>")
        .to_string();
    let case_doc_path = resolved_ref.split('#').next().unwrap_or("").to_string();
    let harness = case_map
        .get(&YamlValue::String("harness".to_string()))
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    let harness_runtime = harness
        .get(&YamlValue::String("config".to_string()))
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_else(|| harness.clone());
    if harness_runtime.contains_key(&YamlValue::String("job".to_string())) {
        eprintln!("ERROR: contract.job requires harness.jobs metadata list");
        return 1;
    }
    let jobs_yaml = match harness_runtime.get(&YamlValue::String("jobs".to_string())) {
        Some(v) => v,
        None => {
            eprintln!("ERROR: contract.job requires harness.jobs metadata list");
            return 1;
        }
    };
    let jobs_json = yaml_to_json(jobs_yaml);
    let jobs_list = match jobs_json.as_array() {
        Some(v) if !v.is_empty() => v,
        _ => {
            eprintln!("ERROR: harness.jobs must be a non-empty list");
            return 1;
        }
    };
    let mut jobs_obj = serde_json::Map::<String, Value>::new();
    for (idx, row) in jobs_list.iter().enumerate() {
        let Some(row_obj) = row.as_object() else {
            eprintln!("ERROR: harness.jobs[{idx}] must be mapping");
            return 1;
        };
        let job_id = row_obj
            .get("id")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if job_id.is_empty() {
            eprintln!("ERROR: harness.jobs[{idx}].id is required");
            return 1;
        }
        if jobs_obj.contains_key(job_id) {
            eprintln!("ERROR: harness.jobs id must be unique: {job_id}");
            return 1;
        }
        jobs_obj.insert(job_id.to_string(), Value::Object(row_obj.clone()));
    }

    let mut input_override = serde_json::Map::<String, Value>::new();
    for (k, v) in input_pairs {
        input_override.insert(k, Value::String(v));
    }

    let mut caps = Vec::<String>::new();
    if let Some(spec_lang) = harness_runtime
        .get(&YamlValue::String("spec_lang".to_string()))
        .and_then(|v| v.as_mapping())
    {
        if let Some(seq) = spec_lang
            .get(&YamlValue::String("capabilities".to_string()))
            .and_then(|v| v.as_sequence())
        {
            for item in seq {
                if let Some(s) = item.as_str() {
                    let t = s.trim();
                    if !t.is_empty() {
                        caps.push(t.to_string());
                    }
                }
            }
        }
    }
    let prev_caps = std::env::var("SPEC_RUNNER_SPEC_LANG_CAPABILITIES").ok();
    let prev_jobs = std::env::var("SPEC_RUNNER_SPEC_LANG_JOBS_JSON").ok();
    let prev_cli_overrides = std::env::var("SPEC_RUNNER_JOB_INPUT_OVERRIDES_JSON").ok();
    if !caps.is_empty() {
        std::env::set_var("SPEC_RUNNER_SPEC_LANG_CAPABILITIES", caps.join(","));
    }
    std::env::set_var(
        "SPEC_RUNNER_SPEC_LANG_JOBS_JSON",
        serde_json::to_string(&Value::Object(jobs_obj.clone()))
            .unwrap_or_else(|_| "{}".to_string()),
    );
    if !input_override.is_empty() {
        std::env::set_var(
            "SPEC_RUNNER_JOB_INPUT_OVERRIDES_JSON",
            serde_json::to_string(&Value::Object(input_override.clone()))
                .unwrap_or_else(|_| "{}".to_string()),
        );
    }

    let mut hook_exprs: HashMap<String, Vec<Value>> = HashMap::new();
    if harness_runtime.contains_key(&YamlValue::String("on".to_string())) {
        eprintln!("ERROR: when.harness_on_forbidden: harness.on is not supported; use case.when");
        return 1;
    }
    if harness_runtime.contains_key(&YamlValue::String("when".to_string())) {
        eprintln!(
            "ERROR: when.harness_when_forbidden: harness.when is not supported; use case.when"
        );
        return 1;
    }
    if let Some(raw_when_value) = case_map.get(&YamlValue::String("when".to_string())) {
        let Some(raw_when) = raw_when_value.as_mapping() else {
            eprintln!("ERROR: when.invalid_shape: when must be a mapping");
            return 1;
        };
        for (raw_key, raw_values) in raw_when {
            let key = raw_key.as_str().unwrap_or("").trim().to_string();
            if !matches!(key.as_str(), "required" | "optional" | "fail" | "complete") {
                eprintln!("ERROR: when.unknown_key: {key}");
                return 1;
            }
            let Some(seq) = raw_values.as_sequence() else {
                eprintln!("ERROR: when.invalid_shape: when.{key} must be non-empty list");
                return 1;
            };
            if seq.is_empty() {
                eprintln!("ERROR: when.invalid_shape: when.{key} must be non-empty list");
                return 1;
            }
            let mut compiled = Vec::<Value>::new();
            for (idx, item) in seq.iter().enumerate() {
                let expr = yaml_to_json(item);
                if !expr.is_object() {
                    eprintln!(
                        "ERROR: when.expr_invalid: when.{key}[{idx}] must be mapping expression"
                    );
                    return 1;
                }
                compiled.push(expr);
            }
            hook_exprs.insert(key, compiled);
        }
    }

    let restore_env = || {
        if let Some(prev) = prev_caps.clone() {
            std::env::set_var("SPEC_RUNNER_SPEC_LANG_CAPABILITIES", prev);
        } else {
            std::env::remove_var("SPEC_RUNNER_SPEC_LANG_CAPABILITIES");
        }
        if let Some(prev) = prev_jobs.clone() {
            std::env::set_var("SPEC_RUNNER_SPEC_LANG_JOBS_JSON", prev);
        } else {
            std::env::remove_var("SPEC_RUNNER_SPEC_LANG_JOBS_JSON");
        }
        if let Some(prev) = prev_cli_overrides.clone() {
            std::env::set_var("SPEC_RUNNER_JOB_INPUT_OVERRIDES_JSON", prev);
        } else {
            std::env::remove_var("SPEC_RUNNER_JOB_INPUT_OVERRIDES_JSON");
        }
    };

    let mut summary_json = Value::Null;
    let mut passed_clauses = 0_i64;
    let mut failed_clauses = 0_i64;
    let mut required_passed = 0_i64;
    let mut optional_passed = 0_i64;

    let contract_container = case_map
        .get(&YamlValue::String("clauses".to_string()))
        .and_then(|v| v.as_mapping())
        .map(|m| ("clauses", m))
        .or_else(|| {
            case_map
                .get(&YamlValue::String("asserts".to_string()))
                .and_then(|v| v.as_mapping())
                .map(|m| ("asserts", m))
        });
    if let Some((container_name, contract_map)) = contract_container {
        let parse_imports = |raw: Option<&YamlValue>,
                             where_path: &str|
         -> Result<HashMap<String, String>, String> {
            let mut out = HashMap::<String, String>::new();
            let Some(raw_imports) = raw else {
                return Ok(out);
            };
            let Some(items) = raw_imports.as_sequence() else {
                return Err(format!(
                    "{where_path} imports must be list form with from/names/as"
                ));
            };
            for (item_idx, raw_item) in items.iter().enumerate() {
                let Some(item_map) = raw_item.as_mapping() else {
                    return Err(format!("{where_path}.imports[{item_idx}] must be mapping"));
                };
                let src = item_map
                    .get(&YamlValue::String("from".to_string()))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                if src != "artifact" {
                    return Err(format!(
                        "{where_path}.imports[{item_idx}].from must be artifact"
                    ));
                }
                let names_val = item_map.get(&YamlValue::String("names".to_string()));
                let Some(names_seq) = names_val.and_then(|v| v.as_sequence()) else {
                    return Err(format!(
                        "{where_path}.imports[{item_idx}].names must be non-empty list"
                    ));
                };
                if names_seq.is_empty() {
                    return Err(format!(
                        "{where_path}.imports[{item_idx}].names must be non-empty list"
                    ));
                }
                let mut alias_map = HashMap::<String, String>::new();
                if let Some(raw_alias) = item_map.get(&YamlValue::String("as".to_string())) {
                    let Some(alias_yaml) = raw_alias.as_mapping() else {
                        return Err(format!(
                            "{where_path}.imports[{item_idx}].as must be mapping"
                        ));
                    };
                    for (raw_key, raw_value) in alias_yaml {
                        let source_name = raw_key.as_str().unwrap_or("").trim().to_string();
                        let local_name = raw_value.as_str().unwrap_or("").trim().to_string();
                        if source_name.is_empty() || local_name.is_empty() {
                            return Err(format!(
                                "{where_path}.imports[{item_idx}].as keys and values must be non-empty strings"
                            ));
                        }
                        alias_map.insert(source_name, local_name);
                    }
                }
                let mut source_names = HashSet::<String>::new();
                for (name_idx, raw_name) in names_seq.iter().enumerate() {
                    let source_name = raw_name.as_str().unwrap_or("").trim().to_string();
                    if source_name.is_empty() {
                        return Err(format!(
                            "{where_path}.imports[{item_idx}].names[{name_idx}] must be non-empty string"
                        ));
                    }
                    source_names.insert(source_name.clone());
                    let local_name = alias_map
                        .get(&source_name)
                        .cloned()
                        .unwrap_or_else(|| source_name.clone());
                    out.insert(local_name, source_name);
                }
                for alias_key in alias_map.keys() {
                    if !source_names.contains(alias_key) {
                        return Err(format!(
                            "{where_path}.imports[{item_idx}].as key {alias_key:?} is not present in names"
                        ));
                    }
                }
            }
            Ok(out)
        };

        let default_required = contract_map
            .get(&YamlValue::String("defaults".to_string()))
            .and_then(|v| v.as_mapping())
            .and_then(|m| m.get(&YamlValue::String("required".to_string())))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let default_imports = match parse_imports(
            contract_map.get(&YamlValue::String("imports".to_string())),
            container_name,
        ) {
            Ok(v) => v,
            Err(err) => {
                restore_env();
                eprintln!("ERROR: {err}");
                return 1;
            }
        };
        let (steps_key, step_path_label) = if container_name == "asserts" {
            ("checks", "asserts.checks")
        } else {
            ("predicates", "clauses.predicates")
        };
        let contract_steps = contract_map
            .get(&YamlValue::String(steps_key.to_string()))
            .and_then(|v| v.as_sequence())
            .cloned()
            .unwrap_or_default();

        for (step_idx, step) in contract_steps.iter().enumerate() {
            let step_map = match step.as_mapping() {
                Some(m) => m,
                None => continue,
            };
            if step_map.contains_key(&YamlValue::String("target".to_string()))
                || step_map.contains_key(&YamlValue::String("on".to_string()))
            {
                restore_env();
                eprintln!(
                    "ERROR: {step_path_label}[{step_idx}] target/on is forbidden; use imports"
                );
                return 1;
            }
            if step_map.contains_key(&YamlValue::String("class".to_string())) {
                restore_env();
                eprintln!(
                    "ERROR: {step_path_label}[{step_idx}].class is removed; use required/priority/severity"
                );
                return 1;
            }
            let required = match step_map.get(&YamlValue::String("required".to_string())) {
                None => default_required,
                Some(v) => {
                    let Some(val) = v.as_bool() else {
                        restore_env();
                        eprintln!("ERROR: {step_path_label}[{step_idx}].required must be boolean");
                        return 1;
                    };
                    val
                }
            };
            let parse_positive_int = |field: &str| -> Result<i64, String> {
                match step_map.get(&YamlValue::String(field.to_string())) {
                    None => Ok(1),
                    Some(raw) => raw.as_i64().filter(|v| *v >= 1).ok_or_else(|| {
                        format!("{step_path_label}[{step_idx}].{field} must be integer >= 1")
                    }),
                }
            };
            if let Err(err) = parse_positive_int("priority") {
                restore_env();
                eprintln!("ERROR: {err}");
                return 1;
            }
            if let Err(err) = parse_positive_int("severity") {
                restore_env();
                eprintln!("ERROR: {err}");
                return 1;
            }
            if let Some(raw_purpose) = step_map.get(&YamlValue::String("purpose".to_string())) {
                if raw_purpose.as_str().map(str::trim).unwrap_or("").is_empty() {
                    restore_env();
                    eprintln!(
                        "ERROR: {step_path_label}[{step_idx}].purpose must be non-empty string"
                    );
                    return 1;
                }
            }
            let step_id = step_map
                .get(&YamlValue::String("id".to_string()))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let assert_path = if let Some(id) = step_id {
                format!("clauses.predicates[{step_idx}]<{id}>")
            } else {
                format!("clauses.predicates[{step_idx}]")
            };
            let step_imports = match parse_imports(
                step_map.get(&YamlValue::String("imports".to_string())),
                &format!("{step_path_label}[{step_idx}]"),
            ) {
                Ok(v) => v,
                Err(err) => {
                    restore_env();
                    eprintln!("ERROR: {err}");
                    return 1;
                }
            };
            let mut effective_imports = default_imports.clone();
            for (k, v) in step_imports {
                effective_imports.insert(k, v);
            }

            let raw_assert = match step_map.get(&YamlValue::String("assert".to_string())) {
                Some(v) => v,
                None => {
                    restore_env();
                    eprintln!("ERROR: {step_path_label}[{step_idx}].assert is required");
                    return 1;
                }
            };
            let assert_items: Vec<&YamlValue> = if let Some(seq) = raw_assert.as_sequence() {
                if seq.is_empty() {
                    restore_env();
                    eprintln!("ERROR: {step_path_label}[{step_idx}].assert must be non-empty");
                    return 1;
                }
                seq.iter().collect()
            } else {
                vec![raw_assert]
            };

            let mut clause_pass = true;
            let mut clause_error: Option<String> = None;
            let target_name: Option<String> = effective_imports.get("subject").cloned();

            for (assert_idx, raw_expr) in assert_items.iter().enumerate() {
                let mut symbols = HashMap::<String, Value>::new();
                let mut subject_value = Value::Null;
                for (local_name, source_key) in &effective_imports {
                    let resolved = match source_key.as_str() {
                        "summary_json" => summary_json.clone(),
                        "meta_json" => json!({
                            "job_ref": ref_arg,
                            "resolved_ref": resolved_ref,
                            "jobs": jobs_obj.clone(),
                        }),
                        "violation_count" => Value::Number(failed_clauses.into()),
                        "status" => Value::String(
                            if failed_clauses == 0 { "pass" } else { "fail" }.to_string(),
                        ),
                        key => summary_json.get(key).cloned().unwrap_or(Value::Null),
                    };
                    if local_name == "subject" {
                        subject_value = resolved.clone();
                    }
                    symbols.insert(local_name.clone(), json!({"lit": resolved}));
                }
                let subject = subject_value;
                let expr = yaml_to_json(&normalize_evaluate_yaml_expr(raw_expr));
                let result = match eval_mapping_ast_with_state(
                    &expr,
                    subject.clone(),
                    symbols,
                    EvalLimits::default(),
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        clause_pass = false;
                        clause_error = Some(format!(
                            "contract evaluation failed at step {} assert {}: {}",
                            step_idx, assert_idx, e.message
                        ));
                        break;
                    }
                };
                if let Some(dispatched) = result.last_dispatch_result.clone() {
                    summary_json = dispatched;
                }
                let ok = json_truthy(&result.value);
                if !ok {
                    clause_pass = false;
                    break;
                }
            }
            clause_pass = clause_pass && clause_error.is_none();
            let requirement_name = if required { "required" } else { "optional" };

            if clause_pass {
                passed_clauses += 1;
                if required {
                    required_passed += 1;
                } else {
                    optional_passed += 1;
                }
                if let Err(e) = run_job_hook_event(
                    &hook_exprs,
                    requirement_name,
                    step_idx,
                    step_id,
                    requirement_name,
                    &assert_path,
                    target_name.as_deref(),
                    "pass",
                    None,
                    passed_clauses,
                    failed_clauses,
                    required_passed,
                    optional_passed,
                    &mut summary_json,
                    &case_id,
                    case_type,
                    &case_doc_path,
                ) {
                    failed_clauses += 1;
                    if let Err(hook_err) = run_job_hook_event(
                        &hook_exprs,
                        "fail",
                        step_idx,
                        step_id,
                        requirement_name,
                        &assert_path,
                        target_name.as_deref(),
                        "fail",
                        Some(&e),
                        passed_clauses,
                        failed_clauses,
                        required_passed,
                        optional_passed,
                        &mut summary_json,
                        &case_id,
                        case_type,
                        &case_doc_path,
                    ) {
                        restore_env();
                        eprintln!("ERROR: runtime.on_hook.fail_handler_failed: {hook_err}");
                        return 1;
                    }
                    restore_env();
                    eprintln!("ERROR: {e}");
                    return 1;
                }
                continue;
            }

            failed_clauses += 1;
            let fail_message = clause_error.unwrap_or_else(|| {
                if required {
                    "contract clause failed: required".to_string()
                } else {
                    "contract clause failed: optional".to_string()
                }
            });
            if !required {
                continue;
            }
            if let Err(hook_err) = run_job_hook_event(
                &hook_exprs,
                "fail",
                step_idx,
                step_id,
                requirement_name,
                &assert_path,
                target_name.as_deref(),
                "fail",
                Some(&fail_message),
                passed_clauses,
                failed_clauses,
                required_passed,
                optional_passed,
                &mut summary_json,
                &case_id,
                case_type,
                &case_doc_path,
            ) {
                restore_env();
                eprintln!("ERROR: runtime.on_hook.fail_handler_failed: {hook_err}");
                return 1;
            }
            restore_env();
            eprintln!("ERROR: {fail_message}");
            return 1;
        }
    }

    if failed_clauses == 0 {
        if let Err(e) = run_job_hook_event(
            &hook_exprs,
            "complete",
            passed_clauses.saturating_sub(1) as usize,
            None,
            "required",
            "contract",
            None,
            "pass",
            None,
            passed_clauses,
            failed_clauses,
            required_passed,
            optional_passed,
            &mut summary_json,
            &case_id,
            case_type,
            &case_doc_path,
        ) {
            restore_env();
            eprintln!("ERROR: {e}");
            return 1;
        }
    }
    restore_env();

    if failed_clauses == 0 {
        println!("OK: job-run passed ({resolved_ref})");
        0
    } else {
        eprintln!("ERROR: job-run contract failures ({resolved_ref}): {failed_clauses}");
        1
    }
}

fn run_job_for_command(root: &Path, subcommand: &str, forwarded: &[String]) -> i32 {
    let Some(spec_ref) = command_spec_ref(subcommand) else {
        eprintln!("ERROR: no registered spec ref for command: {subcommand}");
        return 1;
    };
    let mut args = vec!["--ref".to_string(), spec_ref.to_string()];
    args.extend(forwarded.iter().cloned());
    run_job_run_native(root, &args)
}

fn ensure_validate_report_export_contract(case_block: &str, spec_ref: &str) -> Result<(), String> {
    let required_tokens = [
        "type: contract.export",
        "as: domain.conformance.validate_report_errors",
        "from: assert.function",
        "path: /__export__domain.conformance.validate_report_errors",
        "params:",
        "- report",
        "report.version must equal 1",
        "report.results must be a list",
    ];
    for token in required_tokens {
        if !case_block.contains(token) {
            return Err(format!(
                "producer contract drift for {spec_ref}: missing token `{token}`"
            ));
        }
    }
    Ok(())
}

fn yaml_to_json(value: &YamlValue) -> Value {
    match value {
        YamlValue::Null => Value::Null,
        YamlValue::Bool(b) => Value::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        YamlValue::String(s) => Value::String(s.clone()),
        YamlValue::Sequence(seq) => Value::Array(seq.iter().map(yaml_to_json).collect()),
        YamlValue::Mapping(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if let YamlValue::String(key) = k {
                    out.insert(key.clone(), yaml_to_json(v));
                }
            }
            Value::Object(out)
        }
    }
}

fn parse_validate_report_expr_from_case(case_block: &str, spec_ref: &str) -> Result<Value, String> {
    let doc: YamlValue = serde_yaml::from_str(case_block)
        .map_err(|e| format!("failed to parse producer yaml for {spec_ref}: {e}"))?;
    let root = match doc {
        YamlValue::Mapping(m) => m,
        _ => {
            return Err(format!(
                "invalid producer case shape for {spec_ref}: expected mapping"
            ))
        }
    };
    let clauses_node = root
        .get(&YamlValue::String("clauses".to_string()))
        .and_then(|v| v.as_mapping())
        .map(|m| ("predicates", m))
        .or_else(|| {
            root.get(&YamlValue::String("asserts".to_string()))
                .and_then(|v| v.as_mapping())
                .map(|m| ("checks", m))
        })
        .ok_or_else(|| format!("missing clauses/asserts in producer case: {spec_ref}"))?;
    let (pred_key, clauses_map) = clauses_node;
    let assert_seq = clauses_map
        .get(&YamlValue::String(pred_key.to_string()))
        .ok_or_else(|| format!("missing {pred_key} in producer case: {spec_ref}"))?;
    let assert_seq = match assert_seq {
        YamlValue::Sequence(seq) => seq,
        _ => {
            return Err(format!(
                "producer clauses.predicates must be sequence: {spec_ref}"
            ))
        }
    };
    let target_step_id = "__export__domain.conformance.validate_report_errors";
    for step in assert_seq {
        let step_map = match step {
            YamlValue::Mapping(m) => m,
            _ => continue,
        };
        let sid = step_map
            .get(&YamlValue::String("id".to_string()))
            .and_then(|v| match v {
                YamlValue::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("");
        if sid != target_step_id {
            continue;
        }
        let checks = step_map
            .get(&YamlValue::String("assert".to_string()))
            .ok_or_else(|| format!("producer predicate missing assert: {target_step_id}"))?;
        let check_seq: Vec<YamlValue> = match checks {
            YamlValue::Sequence(seq) => seq.clone(),
            YamlValue::Mapping(_) => vec![checks.clone()],
            _ => {
                return Err(format!(
                    "producer predicate assert must be mapping or sequence: {target_step_id}"
                ))
            }
        };
        if check_seq.len() != 1 {
            return Err(format!(
                "producer predicate assert must contain exactly one expression: {target_step_id}"
            ));
        }
        return Ok(yaml_to_json(&normalize_evaluate_yaml_expr(&check_seq[0])));
    }
    Err(format!(
        "producer step not found in {spec_ref}: {target_step_id}"
    ))
}

fn validate_report_payload(payload: &Value, expr: &Value) -> Vec<String> {
    match eval_mapping_ast(
        expr,
        payload.clone(),
        std::collections::HashMap::new(),
        EvalLimits::default(),
    ) {
        Ok(Value::Array(items)) => items
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>(),
        Ok(_) => vec!["validate_report expression must return list".to_string()],
        Err(e) => vec![format!("spec_lang error: {}", e.message)],
    }
}

fn normalize_evaluate_yaml_expr(raw_expr: &YamlValue) -> YamlValue {
    let inner = if let YamlValue::Mapping(map) = raw_expr {
        if map.len() == 1 {
            if let Some(v) = map.get(&YamlValue::String("evaluate".to_string())) {
                v.clone()
            } else {
                raw_expr.clone()
            }
        } else {
            raw_expr.clone()
        }
    } else {
        raw_expr.clone()
    };
    match inner {
        YamlValue::Sequence(seq) => {
            if seq.len() == 1 {
                seq[0].clone()
            } else {
                let mut wrapped = serde_yaml::Mapping::new();
                wrapped.insert(
                    YamlValue::String("std.logic.and".to_string()),
                    YamlValue::Sequence(seq),
                );
                YamlValue::Mapping(wrapped)
            }
        }
        other => other,
    }
}

fn run_validate_report_native(root: &Path, forwarded: &[String]) -> i32 {
    debug_log("validate-report:start");
    if forwarded.len() != 1 {
        eprintln!("usage: validate-report <report-json-path>");
        return 2;
    }
    let report_path = {
        let p = PathBuf::from(&forwarded[0]);
        if p.is_absolute() {
            p
        } else {
            root.join(p)
        }
    };
    let report_text = match fs::read_to_string(&report_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "ERROR: failed to read report {}: {e}",
                report_path.display()
            );
            return 1;
        }
    };
    debug_log(&format!(
        "validate-report:report-bytes={}",
        report_text.len()
    ));
    let payload: Value = match serde_json::from_str(&report_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: invalid report json {}: {e}", report_path.display());
            return 1;
        }
    };
    let spec_ref = match command_spec_ref("validate-report") {
        Some(v) => v,
        None => {
            eprintln!("ERROR: missing spec ref registration for validate-report");
            return 1;
        }
    };
    debug_log(&format!("validate-report:spec-ref={spec_ref}"));
    let case_block = match load_case_block_from_spec_ref(root, spec_ref) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    debug_log(&format!(
        "validate-report:producer-case-bytes={}",
        case_block.len()
    ));
    if let Err(e) = ensure_validate_report_export_contract(&case_block, spec_ref) {
        eprintln!("ERROR: {e}");
        return 1;
    }
    let expr = match parse_validate_report_expr_from_case(&case_block, spec_ref) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    debug_log_at(2, "validate-report:loaded expression from producer export");
    let errors = validate_report_payload(&payload, &expr);
    debug_log(&format!("validate-report:error-count={}", errors.len()));
    if errors.is_empty() {
        println!("OK: valid conformance report ({})", report_path.display());
        0
    } else {
        for err in errors {
            eprintln!("ERROR: {err}");
        }
        1
    }
}

fn run_spec_ref_print(subcommand: &str) -> i32 {
    debug_log(&format!("spec-ref:lookup subcommand={subcommand}"));
    let Some(spec_ref) = command_spec_ref(subcommand) else {
        eprintln!("ERROR: no registered spec ref for command: {subcommand}");
        return 1;
    };
    debug_log(&format!("spec-ref:resolved {spec_ref}"));
    println!("{spec_ref}");
    0
}

fn run_spec_eval_native(root: &Path, forwarded: &[String]) -> i32 {
    debug_log(&format!(
        "spec-eval:start cwd={} args={}",
        root.display(),
        forwarded.len()
    ));
    let mut expr_json: Option<String> = None;
    let mut expr_file: Option<String> = None;
    let mut subject_json: Option<String> = None;
    let mut subject_file: Option<String> = None;

    let mut i = 0usize;
    while i < forwarded.len() {
        let arg = forwarded[i].as_str();
        match arg {
            "--expr-json" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --expr-json requires value");
                    return 2;
                }
                expr_json = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--expr-file" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --expr-file requires value");
                    return 2;
                }
                expr_file = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--subject-json" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --subject-json requires value");
                    return 2;
                }
                subject_json = Some(forwarded[i + 1].clone());
                i += 2;
            }
            "--subject-file" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --subject-file requires value");
                    return 2;
                }
                subject_file = Some(forwarded[i + 1].clone());
                i += 2;
            }
            _ => {
                eprintln!("ERROR: unsupported spec-eval arg: {arg}");
                return 2;
            }
        }
    }

    let expr_val: Value = match (expr_json, expr_file) {
        (Some(raw), None) => match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: invalid --expr-json: {e}");
                return 2;
            }
        },
        (None, Some(path)) => {
            let p = root.join(path.trim_start_matches('/'));
            let raw = match fs::read_to_string(&p) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ERROR: failed to read --expr-file {}: {e}", p.display());
                    return 2;
                }
            };
            match serde_json::from_str(&raw) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: invalid JSON in --expr-file {}: {e}", p.display());
                    return 2;
                }
            }
        }
        _ => {
            eprintln!("ERROR: provide exactly one of --expr-json or --expr-file");
            return 2;
        }
    };

    let subject_val: Value = match (subject_json, subject_file) {
        (Some(raw), None) => match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: invalid --subject-json: {e}");
                return 2;
            }
        },
        (None, Some(path)) => {
            let p = root.join(path.trim_start_matches('/'));
            let raw = match fs::read_to_string(&p) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ERROR: failed to read --subject-file {}: {e}", p.display());
                    return 2;
                }
            };
            match serde_json::from_str(&raw) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: invalid JSON in --subject-file {}: {e}", p.display());
                    return 2;
                }
            }
        }
        (None, None) => Value::Null,
        _ => {
            eprintln!("ERROR: provide at most one of --subject-json or --subject-file");
            return 2;
        }
    };

    match eval_mapping_ast(
        &expr_val,
        subject_val,
        std::collections::HashMap::new(),
        EvalLimits::default(),
    ) {
        Ok(v) => {
            debug_log("spec-eval:success");
            println!(
                "{}",
                serde_json::to_string_pretty(&v).unwrap_or_else(|_| "null".to_string())
            );
            0
        }
        Err(e) => {
            debug_log(&format!("spec-eval:error {}", e.message));
            eprintln!("ERROR: {}", e.message);
            1
        }
    }
}

fn runner_command(runner_bin: &str, runner_impl: &str, subcommand: &str) -> Vec<String> {
    let normalized = runner_bin.replace('\\', "/");
    let adapter_rel = "runners/public/runner_adapter.sh".to_string();
    let adapter_prefixed = format!("./{}", adapter_rel);
    let adapter_suffix = format!("/{}", adapter_rel);
    if normalized.ends_with(&adapter_suffix)
        || normalized == adapter_rel
        || normalized == adapter_prefixed
    {
        return vec![
            runner_bin.to_string(),
            "--impl".to_string(),
            runner_impl.to_string(),
            subcommand.to_string(),
        ];
    }
    vec![runner_bin.to_string(), subcommand.to_string()]
}

fn runner_command_with_liveness(
    runner_bin: &str,
    runner_impl: &str,
    subcommand: &str,
    level: &str,
    stall_ms: &str,
    kill_grace_ms: &str,
    hard_cap_ms: &str,
) -> Vec<String> {
    let normalized = runner_bin.replace('\\', "/");
    let adapter_rel = "runners/public/runner_adapter.sh".to_string();
    let adapter_prefixed = format!("./{}", adapter_rel);
    let adapter_suffix = format!("/{}", adapter_rel);
    if normalized.ends_with(&adapter_suffix)
        || normalized == adapter_rel
        || normalized == adapter_prefixed
    {
        return vec![
            runner_bin.to_string(),
            "--impl".to_string(),
            runner_impl.to_string(),
            "--liveness-level".to_string(),
            level.to_string(),
            "--liveness-stall-ms".to_string(),
            stall_ms.to_string(),
            "--liveness-kill-grace-ms".to_string(),
            kill_grace_ms.to_string(),
            "--liveness-hard-cap-ms".to_string(),
            hard_cap_ms.to_string(),
            subcommand.to_string(),
        ];
    }
    vec![runner_bin.to_string(), subcommand.to_string()]
}

fn run_command_capture_code(command: &[String], root: &Path) -> i32 {
    if command.is_empty() {
        return 1;
    }
    let span_id = profiler_start_span(
        "subprocess.exec",
        "subprocess",
        "subprocess.exec",
        None,
        json!({
            "argv_preview": command.join(" "),
            "cwd": root.display().to_string()
        }),
    );
    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..])
        .current_dir(root)
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit());
    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id();
            profiler_event(
                "subprocess_state",
                span_id.as_deref(),
                json!({"state":"spawned","pid":pid}),
            );
            let code = match child.wait() {
                Ok(status) => status.code().unwrap_or(1),
                Err(e) => {
                    profiler_event(
                        "subprocess_state",
                        span_id.as_deref(),
                        json!({"state":"wait_error","message":e.to_string()}),
                    );
                    eprintln!("ERROR: failed waiting command '{}': {e}", command[0]);
                    1
                }
            };
            profiler_event(
                "subprocess_state",
                span_id.as_deref(),
                json!({"state":"exit","pid":pid,"returncode":code}),
            );
            profiler_finish_span(
                span_id.as_deref(),
                if code == 0 { "ok" } else { "error" },
                if code == 0 {
                    None
                } else {
                    Some(json!({"category":"runtime","message":format!("non-zero exit: {code}")}))
                },
            );
            code
        }
        Err(e) => {
            eprintln!("ERROR: failed to run command '{}': {e}", command[0]);
            profiler_finish_span(
                span_id.as_deref(),
                "error",
                Some(json!({"category":"runtime","message":e.to_string()})),
            );
            1
        }
    }
}

fn collect_unit_test_opt_out(root: &Path) -> Value {
    let tests_root = root.join("tests");
    let baseline_path = root.join("specs/04_governance/metrics/unit_test_opt_out_baseline.json");
    let mut total = 0_i64;
    let mut opted_out = 0_i64;
    let prefix = "# SPEC-OPT-OUT:";

    if tests_root.exists() {
        if let Ok(entries) = fs::read_dir(&tests_root) {
            let mut files = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with("test_") && n.ends_with("_unit.py"))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            files.sort();
            for path in files {
                total += 1;
                let first_non_empty = fs::read_to_string(&path)
                    .ok()
                    .and_then(|txt| {
                        txt.lines()
                            .map(|l| l.trim())
                            .find(|l| !l.is_empty())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_default();
                if first_non_empty.starts_with(prefix) {
                    opted_out += 1;
                }
            }
        }
    }

    let mut baseline_max = 0_i64;
    if let Ok(text) = fs::read_to_string(&baseline_path) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(x) = v
                .as_object()
                .and_then(|m| m.get("max_opt_out_file_count"))
                .and_then(|n| n.as_i64())
            {
                baseline_max = x;
            }
        }
    }

    json!({
        "total_unit_test_files": total,
        "opt_out_file_count": opted_out,
        "baseline_max_opt_out_file_count": baseline_max,
    })
}

fn run_ci_gate_summary_native(root: &Path, forwarded: &[String]) -> i32 {
    let mut out = ".artifacts/gate-summary.json".to_string();
    let mut runner_bin = env::var("SPEC_CI_RUNNER_BIN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
        })
        .unwrap_or_else(|| "./target/debug/spec_runner_cli".to_string());
    let mut runner_impl = env::var("SPEC_RUNNER_IMPL").unwrap_or_else(|_| "rust".to_string());
    let mut trace_out = env::var("SPEC_RUNNER_TRACE_OUT").unwrap_or_default();
    let mut fail_fast = env_bool("SPEC_RUNNER_FAIL_FAST", true);
    let mut profile_on_fail = profile_level_or_off(
        env::var("SPEC_RUNNER_PROFILE_ON_FAIL")
            .unwrap_or_else(|_| "basic".to_string())
            .as_str(),
    );

    let mut i = 0usize;
    while i < forwarded.len() {
        let arg = forwarded[i].as_str();
        if arg == "--out" {
            if i + 1 >= forwarded.len() {
                eprintln!("ERROR: --out requires value");
                return 2;
            }
            out = forwarded[i + 1].clone();
            i += 2;
            continue;
        }
        if arg == "--runner-bin" {
            if i + 1 >= forwarded.len() {
                eprintln!("ERROR: --runner-bin requires value");
                return 2;
            }
            runner_bin = forwarded[i + 1].clone();
            i += 2;
            continue;
        }
        if arg == "--runner-impl" {
            if i + 1 >= forwarded.len() {
                eprintln!("ERROR: --runner-impl requires value");
                return 2;
            }
            runner_impl = forwarded[i + 1].clone();
            i += 2;
            continue;
        }
        if arg == "--trace-out" {
            if i + 1 >= forwarded.len() {
                eprintln!("ERROR: --trace-out requires value");
                return 2;
            }
            trace_out = forwarded[i + 1].clone();
            i += 2;
            continue;
        }
        if arg == "--policy-case" {
            // Reserved for compatibility; policy is currently "all steps must pass".
            if i + 1 >= forwarded.len() {
                eprintln!("ERROR: --policy-case requires value");
                return 2;
            }
            i += 2;
            continue;
        }
        if arg == "--fail-fast" {
            fail_fast = true;
            i += 1;
            continue;
        }
        if arg == "--continue-on-fail" {
            fail_fast = false;
            i += 1;
            continue;
        }
        if arg == "--profile-on-fail" {
            if i + 1 >= forwarded.len() {
                eprintln!("ERROR: --profile-on-fail requires value");
                return 2;
            }
            profile_on_fail = profile_level_or_off(&forwarded[i + 1]);
            i += 2;
            continue;
        }
        eprintln!("ERROR: unsupported ci-gate-summary arg: {arg}");
        return 2;
    }

    if runner_impl == "rust" {
        let runner_path = Path::new(&runner_bin);
        if !runner_path.exists() {
            eprintln!(
                "ERROR: ci-gate-summary runner binary not found: {} (set --runner-bin or SPEC_CI_RUNNER_BIN)",
                runner_bin
            );
        }
        if !runner_path.is_file() {
            eprintln!(
                "ERROR: ci-gate-summary runner binary is not a file: {} (set --runner-bin or SPEC_CI_RUNNER_BIN)",
                runner_bin
            );
        }
        #[cfg(unix)]
        {
            match fs::metadata(runner_path) {
                Ok(meta) => {
                    if meta.permissions().mode() & 0o111 == 0 {
                        eprintln!(
                            "ERROR: ci-gate-summary runner binary is not executable: {} (set --runner-bin or SPEC_CI_RUNNER_BIN)",
                            runner_bin
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "ERROR: ci-gate-summary unable to inspect runner binary '{}': {}",
                        runner_bin, e
                    );
                }
            }
        }
    }

    let broad_liveness_level =
        env::var("SPEC_CI_GOV_BROAD_LIVENESS_LEVEL").unwrap_or_else(|_| "strict".to_string());
    let broad_liveness_stall_ms =
        env::var("SPEC_CI_GOV_BROAD_LIVENESS_STALL_MS").unwrap_or_else(|_| "5000".to_string());
    let broad_liveness_kill_grace_ms =
        env::var("SPEC_CI_GOV_BROAD_LIVENESS_KILL_GRACE_MS").unwrap_or_else(|_| "1000".to_string());
    let broad_liveness_hard_cap_ms =
        env::var("SPEC_CI_GOV_BROAD_LIVENESS_HARD_CAP_MS").unwrap_or_else(|_| "120000".to_string());
    let include_conformance_parity = env_bool("SPEC_CI_INCLUDE_CONFORMANCE_PARITY", false);
    let mut default_steps: Vec<(&str, Vec<String>)> = vec![
        (
            "governance_broad",
            runner_command_with_liveness(
                &runner_bin,
                &runner_impl,
                "governance-broad-native",
                &broad_liveness_level,
                &broad_liveness_stall_ms,
                &broad_liveness_kill_grace_ms,
                &broad_liveness_hard_cap_ms,
            ),
        ),
        (
            "docs_generate_check",
            runner_command(&runner_bin, &runner_impl, "docs-generate-check"),
        ),
        (
            "docs_lint",
            runner_command(&runner_bin, &runner_impl, "docs-lint"),
        ),
        (
            "normalize_check",
            runner_command(&runner_bin, &runner_impl, "normalize-check"),
        ),
        (
            "schema_registry_build",
            runner_command(&runner_bin, &runner_impl, "schema-registry-build"),
        ),
        (
            "schema_registry_check",
            runner_command(&runner_bin, &runner_impl, "schema-registry-check"),
        ),
        (
            "schema_docs_check",
            runner_command(&runner_bin, &runner_impl, "schema-docs-check"),
        ),
        (
            "evaluate_style",
            runner_command(&runner_bin, &runner_impl, "style-check"),
        ),
    ];
    if include_conformance_parity {
        default_steps.push((
            "conformance_parity",
            runner_command(&runner_bin, &runner_impl, "conformance-parity"),
        ));
    }

    let started = now_iso_utc_fallback();
    let t0 = Instant::now();
    let mut steps = Vec::<Value>::new();
    let mut events = Vec::<Value>::new();
    let mut first_failure_step: Option<String> = None;
    let mut aborted = false;
    for (name, command) in default_steps {
        let mut normalize_mode: Option<String> = None;
        let mut normalized_file_count: Option<i64> = None;
        if name == "normalize_check" {
            let (mode, count) = normalize_step_metadata_from_command(root, &command);
            normalize_mode = Some(mode);
            normalized_file_count = count;
        }
        if aborted {
            steps.push(json!({
                "name": name,
                "command": command,
                "status": "skipped",
                "exit_code": Value::Null,
                "duration_ms": 0,
                "skip_reason": "fail_fast.after_failure",
                "blocked_by": first_failure_step.clone(),
            }));
            events.push(json!({
                "ts_ns": t0.elapsed().as_nanos() as i64,
                "kind": "checkpoint",
                "span_id": "run.total",
                "attrs": {"event":"gate.step.skipped","step":name,"blocked_by":first_failure_step.clone()}
            }));
            continue;
        }
        events.push(json!({
            "ts_ns": t0.elapsed().as_nanos() as i64,
            "kind": "checkpoint",
            "span_id": "run.total",
            "attrs": {"event":"gate.step.start","step":name}
        }));
        if let Some(mode) = normalize_mode.as_ref() {
            if let Some(attrs) = events
                .last_mut()
                .and_then(Value::as_object_mut)
                .and_then(|v| v.get_mut("attrs"))
                .and_then(Value::as_object_mut)
            {
                attrs.insert("normalize_mode".to_string(), Value::String(mode.clone()));
                attrs.insert(
                    "normalized_file_count".to_string(),
                    normalized_file_count.map_or(Value::Null, |n| Value::from(n)),
                );
            }
        }
        println!("[gate] {name}: {}", command.join(" "));
        let step_start = Instant::now();
        let code = run_command_capture_code(&command, root);
        let duration_ms = step_start.elapsed().as_millis() as i64;
        let status = if code == 0 { "pass" } else { "fail" };
        let mut step_row = json!({
            "name": name,
            "command": command,
            "status": status,
            "exit_code": code,
            "duration_ms": duration_ms,
        });
        if let Some(dst) = step_row.as_object_mut() {
            if let Some(mode) = normalize_mode.as_ref() {
                dst.insert("normalize_mode".to_string(), Value::String(mode.clone()));
                dst.insert(
                    "normalized_file_count".to_string(),
                    normalized_file_count.map_or(Value::Null, |n| Value::from(n)),
                );
            }
        }
        if name == "governance_broad" {
            if let Some(dst) = step_row.as_object_mut() {
                dst.insert(
                    "triage_phase".to_string(),
                    Value::String("broad".to_string()),
                );
                dst.insert("broad_required".to_string(), Value::Bool(true));
            }
        }
        steps.push(step_row);
        events.push(json!({
            "ts_ns": t0.elapsed().as_nanos() as i64,
            "kind": "checkpoint",
            "span_id": "run.total",
            "attrs": {"event":format!("gate.step.{}", status),"step":name,"exit_code":code}
        }));
        if status == "fail" && first_failure_step.is_none() {
            first_failure_step = Some(name.to_string());
            if fail_fast {
                aborted = true;
                events.push(json!({
                    "ts_ns": t0.elapsed().as_nanos() as i64,
                    "kind": "checkpoint",
                    "span_id": "run.total",
                    "attrs": {"event":"gate.fail_fast.abort","after_step":name}
                }));
            }
        }
    }

    let verdict = steps
        .iter()
        .all(|s| s.get("status").and_then(Value::as_str) == Some("pass"));
    let first_failure = steps
        .iter()
        .find_map(|s| {
            s.get("exit_code")
                .and_then(Value::as_i64)
                .filter(|c| *c != 0)
        })
        .unwrap_or(1) as i32;
    let exit_code = if verdict { 0 } else { first_failure };

    let finished = now_iso_utc_fallback();
    let total_duration_ms = t0.elapsed().as_millis() as i64;
    let skipped_step_count = steps
        .iter()
        .filter(|x| x.get("status").and_then(Value::as_str) == Some("skipped"))
        .count();
    let first_failure_for_payload = first_failure_step.clone();
    let aborted_after_for_payload = if fail_fast {
        first_failure_step.clone()
    } else {
        None
    };
    let payload = json!({
        "version": 1,
        "status": if verdict { "pass" } else { "fail" },
        "policy_verdict": if verdict { "pass" } else { "fail" },
        "policy_case": Value::Null,
        "policy_expr": Value::Null,
        "started_at": started,
        "finished_at": finished,
        "total_duration_ms": total_duration_ms,
        "steps": steps,
        "events": events,
        "fail_fast_enabled": fail_fast,
        "first_failure_step": first_failure_for_payload,
        "aborted_after_step": aborted_after_for_payload,
        "skipped_step_count": skipped_step_count,
        "runner_bin": runner_bin,
        "runner_impl": runner_impl,
        "unit_test_opt_out": collect_unit_test_opt_out(root),
    });
    let mut payload = payload;
    let normalize_step_value =
        payload
            .get("steps")
            .and_then(Value::as_array)
            .and_then(|steps_arr| {
                steps_arr
                    .iter()
                    .find(|s| s.get("name").and_then(Value::as_str) == Some("normalize_check"))
                    .cloned()
            });
    if let Some(normalize_step) = normalize_step_value {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "normalize_mode".to_string(),
                normalize_step
                    .get("normalize_mode")
                    .cloned()
                    .unwrap_or(Value::String("full_tree".to_string())),
            );
            obj.insert(
                "normalized_file_count".to_string(),
                normalize_step
                    .get("normalized_file_count")
                    .cloned()
                    .unwrap_or(Value::Null),
            );
        }
    }
    let governance_step_value =
        payload
            .get("steps")
            .and_then(Value::as_array)
            .and_then(|steps_arr| {
                steps_arr
                    .iter()
                    .find(|s| s.get("name").and_then(Value::as_str) == Some("governance_broad"))
                    .cloned()
            });
    if let Some(governance_step) = governance_step_value {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "triage_attempted".to_string(),
                governance_step
                    .get("triage_attempted")
                    .cloned()
                    .unwrap_or(Value::Bool(false)),
            );
            obj.insert(
                "triage_mode".to_string(),
                governance_step
                    .get("triage_mode")
                    .cloned()
                    .unwrap_or(Value::String("not_run".to_string())),
            );
            obj.insert(
                "triage_result".to_string(),
                governance_step
                    .get("triage_result")
                    .cloned()
                    .unwrap_or(Value::String("not_run".to_string())),
            );
            obj.insert(
                "failing_check_ids".to_string(),
                governance_step
                    .get("failing_check_ids")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
            );
            obj.insert(
                "failing_check_prefixes".to_string(),
                governance_step
                    .get("failing_check_prefixes")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
            );
            obj.insert(
                "stall_detected".to_string(),
                governance_step
                    .get("stall_detected")
                    .cloned()
                    .unwrap_or(Value::Bool(false)),
            );
            obj.insert(
                "stall_phase".to_string(),
                governance_step
                    .get("stall_phase")
                    .cloned()
                    .unwrap_or(Value::Null),
            );
        }
    }

    let out_path = if Path::new(&out).is_absolute() {
        PathBuf::from(&out)
    } else {
        root.join(&out)
    };
    if let Some(parent) = out_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "ERROR: failed to create output directory for {}: {e}",
                out_path.display()
            );
            return 1;
        }
    }
    if let Err(e) = fs::write(
        &out_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        ),
    ) {
        eprintln!(
            "ERROR: failed to write gate summary {}: {e}",
            out_path.display()
        );
        return 1;
    }
    if !trace_out.trim().is_empty() {
        let trace_path = if Path::new(&trace_out).is_absolute() {
            PathBuf::from(&trace_out)
        } else {
            root.join(&trace_out)
        };
        if let Some(parent) = trace_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "ERROR: failed to create trace directory for {}: {e}",
                    trace_path.display()
                );
                return 1;
            }
        }
        let trace_payload = json!({
            "version": 1,
            "runner_bin": payload.get("runner_bin").cloned().unwrap_or(Value::Null),
            "runner_impl": payload.get("runner_impl").cloned().unwrap_or(Value::Null),
            "steps": payload.get("steps").cloned().unwrap_or(Value::Array(vec![])),
            "events": payload.get("events").cloned().unwrap_or(Value::Array(vec![])),
            "fail_fast_enabled": payload.get("fail_fast_enabled").cloned().unwrap_or(Value::Bool(false)),
            "first_failure_step": payload.get("first_failure_step").cloned().unwrap_or(Value::Null),
        });
        if let Err(e) = fs::write(
            &trace_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&trace_payload).unwrap_or_else(|_| "{}".to_string())
            ),
        ) {
            eprintln!(
                "ERROR: failed to write gate trace {}: {e}",
                trace_path.display()
            );
            return 1;
        }
        println!("[gate] trace: {}", trace_path.display());
    }
    if exit_code != 0 && profile_on_fail != "off" {
        let gate_summary = summarize_gate(
            payload
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            payload.get("first_failure_step").and_then(Value::as_str),
        );
        let run_trace_path = root.join(".artifacts/run-trace.json");
        let run_summary_path = root.join(".artifacts/run-trace-summary.md");
        if let Some(parent) = run_trace_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Some(parent) = run_summary_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let fail_profile_payload = json!({
            "version": 1,
            "run_id": format!("gate-{}", SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)),
            "runner_impl": payload.get("runner_impl").cloned().unwrap_or(Value::Null),
            "started_at": payload.get("started_at").cloned().unwrap_or(Value::Null),
            "ended_at": payload.get("finished_at").cloned().unwrap_or(Value::Null),
            "status": payload.get("status").cloned().unwrap_or(Value::Null),
            "command": "ci-gate-summary",
            "args": [],
            "env_profile": {},
            "spans": [{
                "span_id":"s1",
                "parent_span_id": Value::Null,
                "kind":"run",
                "name":"run.total",
                "phase":"run.total",
                "start_ns":0,
                "end_ns": payload.get("total_duration_ms").and_then(Value::as_i64).unwrap_or(0) * 1_000_000,
                "duration_ms": payload.get("total_duration_ms").and_then(Value::as_i64).unwrap_or(0),
                "status": if exit_code == 0 { "ok" } else { "error" },
                "attrs": {
                    "source":"ci-gate-summary",
                    "normalize_mode": payload.get("normalize_mode").cloned().unwrap_or(Value::Null),
                    "normalized_file_count": payload.get("normalized_file_count").cloned().unwrap_or(Value::Null)
                },
                "error": Value::Null
            }],
            "events": payload.get("events").cloned().unwrap_or(Value::Array(vec![])),
            "summary": {
                "step_count": payload.get("steps").and_then(Value::as_array).map(|x| x.len()).unwrap_or(0),
                "failed_step": payload.get("first_failure_step").cloned().unwrap_or(Value::Null)
            }
        });
        let _ = fs::write(
            &run_trace_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&fail_profile_payload)
                    .unwrap_or_else(|_| "{}".to_string())
            ),
        );
        let mut summary_md = String::new();
        summary_md.push_str("# Run Trace Summary\n\n");
        summary_md.push_str(&format!("- status: `{}`\n", gate_summary.status));
        summary_md.push_str(&format!(
            "- first_failure_step: `{}`\n",
            gate_summary.failed_step.as_deref().unwrap_or("")
        ));
        summary_md.push_str(&format!(
            "- skipped_step_count: `{}`\n\n",
            payload
                .get("skipped_step_count")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ));
        summary_md.push_str("## Suggested Next Command\n\n");
        summary_md.push_str("- `spec_runner_cli --profile-level detailed ci-gate-summary`\n");
        let _ = fs::write(&run_summary_path, summary_md);
        println!("[gate] profile: {}", run_trace_path.display());
        println!("[gate] profile-summary: {}", run_summary_path.display());
    }
    println!("[gate] summary: {}", out_path.display());
    exit_code
}

pub fn run() {
    debug_log("main:start");
    let args: Vec<String> = env::args().collect();
    debug_log("main:args_collected");
    let parsed = match entry::parse_entry(&args) {
        Ok(parsed) => parsed,
        Err(code) => process::exit(code),
    };
    debug_log_at(2, &format!("main:debug-level={}", debug_level()));
    let subcommand = parsed.subcommand;
    let forwarded = parsed.forwarded;
    debug_log(&format!(
        "main:subcommand_parsed subcommand={} forwarded={}",
        subcommand,
        forwarded.len()
    ));

    let root = match find_repo_root() {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("ERROR: {msg}");
            process::exit(1);
        }
    };
    debug_log("main:repo_root_resolved");
    if let Ok(mut guard) = profiler_cell().lock() {
        let opts = profile_options_from_env(&subcommand, &forwarded);
        *guard = Some(RunProfiler::from_options(&opts));
    }
    let dispatch_span = profiler_start_span(
        "runner.dispatch",
        "runner",
        "runner.dispatch",
        None,
        json!({"subcommand": subcommand, "forwarded_count": forwarded.len()}),
    );

    let code = dispatch::dispatch(&root, &subcommand, &forwarded);
    profiler_finish_span(
        dispatch_span.as_deref(),
        if code == 0 { "ok" } else { "error" },
        if code == 0 {
            None
        } else {
            Some(
                json!({"category":"runtime","message":format!("subcommand {} failed with {}", subcommand, code)}),
            )
        },
    );
    if let Ok(mut guard) = profiler_cell().lock() {
        if let Some(prof) = guard.as_mut() {
            let out_path = std::env::var("SPEC_RUNNER_PROFILE_OUT")
                .unwrap_or_else(|_| "/.artifacts/run-trace.json".to_string());
            let summary_out = std::env::var("SPEC_RUNNER_PROFILE_SUMMARY_OUT")
                .unwrap_or_else(|_| "/.artifacts/run-trace-summary.md".to_string());
            prof.close(
                if code == 0 { "pass" } else { "fail" },
                code,
                &root,
                &out_path,
                &summary_out,
            );
        }
    }

    process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spec_ref_accepts_path_and_fragment() {
        let got = parse_spec_ref("/specs/example.spec.md#CASE-1").expect("parse");
        assert_eq!(got.0, "/specs/example.spec.md");
        assert_eq!(got.1.as_deref(), Some("CASE-1"));
    }

    #[test]
    fn parse_spec_ref_accepts_path_only() {
        let got = parse_spec_ref("/specs/example.spec.md").expect("parse");
        assert_eq!(got.0, "/specs/example.spec.md");
        assert!(got.1.is_none());
    }

    #[test]
    fn parse_spec_ref_rejects_empty_fragment() {
        let err = parse_spec_ref("/specs/example.spec.md#").expect_err("expected error");
        assert!(err.contains("empty case id fragment"));
    }

    #[test]
    fn parse_spec_ref_rejects_empty_path() {
        let err = parse_spec_ref("#CASE-1").expect_err("expected error");
        assert!(err.contains("must include path"));
    }

    #[test]
    fn parse_job_ref_accepts_path_and_fragment() {
        let got = parse_job_ref("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-001")
            .expect("parse");
        assert_eq!(
            got.0.as_deref(),
            Some("/specs/impl/rust/jobs/script_jobs.spec.md")
        );
        assert_eq!(got.1, "DCIMPL-RUST-JOB-001");
    }

    #[test]
    fn parse_job_ref_accepts_same_doc_fragment() {
        let got = parse_job_ref("#DCIMPL-RUST-JOB-001").expect("parse");
        assert!(got.0.is_none());
        assert_eq!(got.1, "DCIMPL-RUST-JOB-001");
    }

    #[test]
    fn extract_spec_test_blocks_finds_tagged_yaml_blocks() {
        let md = r#"
before
```yaml contract-spec
id: CASE-1
type: contract.check
```
middle
```yaml
id: NOT-A-SPEC
```
```yaml contract-spec
id: CASE-2
```
after
"#;
        let blocks = extract_spec_test_blocks(md);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].contains("id: CASE-1"));
        assert!(blocks[1].contains("id: CASE-2"));
    }

    #[test]
    fn block_id_extracts_id() {
        let block = "id: DCTEST-001\ncheck: runtime.foo\n";
        assert_eq!(block_id(block).as_deref(), Some("DCTEST-001"));
    }

    #[test]
    fn command_spec_ref_has_validate_report_mapping() {
        let got = command_spec_ref("validate-report");
        assert!(got.is_some());
        assert!(got.expect("mapping").contains("#"));
    }

    #[test]
    fn run_spec_ref_print_returns_nonzero_for_unknown() {
        let code = run_spec_ref_print("unknown-command");
        assert_ne!(code, 0);
    }

    #[test]
    fn load_case_block_supports_contracts_clauses_mapping_shape() {
        let base = std::env::temp_dir().join(format!(
            "dc_runner_rust_case_shape_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let spec_dir = base.join("specs");
        std::fs::create_dir_all(&spec_dir).expect("mkdir");
        let spec_path = spec_dir.join("shape.spec.md");
        let md = r#"```yaml contract-spec
spec_version: 1
schema_ref: /specs/01_schema/schema_v1.md
title: suite title
contracts:
  clauses:
  - id: CASE-001
    purpose: from-clause
    asserts:
      checks:
      - id: assert_1
        assert:
          lit: true
```"#;
        std::fs::write(&spec_path, md).expect("write");

        let out = load_case_block_from_spec_ref(&base, "/specs/shape.spec.md#CASE-001")
            .expect("load case");
        assert!(out.contains("id: CASE-001"));
        assert!(out.contains("spec_version: 1"));
        assert!(out.contains("schema_ref: /specs/01_schema/schema_v1.md"));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn load_case_block_supports_legacy_contracts_list_shape() {
        let base = std::env::temp_dir().join(format!(
            "dc_runner_rust_case_shape_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let spec_dir = base.join("specs");
        std::fs::create_dir_all(&spec_dir).expect("mkdir");
        let spec_path = spec_dir.join("shape_legacy.spec.md");
        let md = r#"```yaml contract-spec
spec_version: 1
schema_ref: /specs/01_schema/schema_v1.md
contracts:
- id: CASE-LEGACY-001
  asserts:
    checks:
    - id: assert_1
      assert:
        lit: true
```"#;
        std::fs::write(&spec_path, md).expect("write");

        let out =
            load_case_block_from_spec_ref(&base, "/specs/shape_legacy.spec.md#CASE-LEGACY-001")
                .expect("load case");
        assert!(out.contains("id: CASE-LEGACY-001"));
        assert!(out.contains("schema_ref: /specs/01_schema/schema_v1.md"));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn load_case_block_rejects_runner_spec_non_canonical_schema_ref() {
        let base = std::env::temp_dir().join(format!(
            "dc-runner-spec-boundary-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let spec_dir = base.join("specs");
        std::fs::create_dir_all(&spec_dir).expect("mkdir");
        let spec_path = spec_dir.join("shape.spec.md");
        let md = r#"```yaml contract-spec
spec_version: 1
schema_ref: /specs/local/schema.md
contracts:
  clauses:
  - id: CASE-001
    asserts:
      checks:
      - id: assert_1
        assert:
          lit: true
```"#;
        std::fs::write(&spec_path, md).expect("write");

        let err = load_case_block_from_spec_ref(&base, "/specs/shape.spec.md#CASE-001")
            .expect_err("expected boundary error");
        assert!(err.contains("canonical schema authority"));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn parse_project_scaffold_args_accepts_canonical_mode() {
        let args = vec![
            "--project-root".to_string(),
            "tmp/demo".to_string(),
            "--bundle-id".to_string(),
            "core".to_string(),
            "--bundle-version".to_string(),
            "1.2.3".to_string(),
        ];
        let parsed = parse_project_scaffold_args(&args).expect("parse");
        assert_eq!(parsed.project_root, PathBuf::from("tmp/demo"));
        match parsed.mode {
            ProjectScaffoldMode::Canonical {
                bundle_id,
                bundle_version,
            } => {
                assert_eq!(bundle_id, "core");
                assert_eq!(bundle_version, "1.2.3");
            }
            _ => panic!("expected canonical mode"),
        }
    }

    #[test]
    fn parse_project_scaffold_args_rejects_external_without_flag() {
        let args = vec![
            "--project-root".to_string(),
            "tmp/demo".to_string(),
            "--bundle-url".to_string(),
            "https://example.invalid/bundle.tar.gz".to_string(),
            "--sha256".to_string(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ];
        let err = parse_project_scaffold_args(&args).expect_err("expected error");
        assert!(err.contains("--allow-external"));
    }

    #[test]
    fn parse_sha256_sidecar_supports_common_formats() {
        let flat = parse_sha256_sidecar(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n",
        )
        .expect("flat checksum");
        assert_eq!(
            flat,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );

        let with_file = parse_sha256_sidecar(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  bundle.tar.gz\n",
        )
        .expect("checksum with filename");
        assert_eq!(flat, with_file);
    }

    #[test]
    fn canonical_bundle_release_builds_expected_paths() {
        let (asset_name, sidecar_name, release_tag, asset_url, sidecar_url) =
            canonical_bundle_release("core", "1.2.3");
        assert_eq!(asset_name, "data-contract-bundle-core-v1.2.3.tar.gz");
        assert_eq!(
            sidecar_name,
            "data-contract-bundle-core-v1.2.3.tar.gz.sha256"
        );
        assert_eq!(release_tag, "v1.2.3");
        assert!(asset_url.ends_with("/v1.2.3/data-contract-bundle-core-v1.2.3.tar.gz"));
        assert!(sidecar_url.ends_with("/v1.2.3/data-contract-bundle-core-v1.2.3.tar.gz.sha256"));
    }
}
