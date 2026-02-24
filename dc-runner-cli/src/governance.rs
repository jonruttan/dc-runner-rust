use crate::spec_source::{read_spec_text, spec_exists};
use serde_json::{json, Value};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Debug)]
struct CriticalCheckResult {
    check_id: String,
    status: String,
    duration_ms: i64,
    reason_token: Option<String>,
    details: Vec<String>,
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

fn now_iso_utc_fallback() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn resolve(root: &Path, raw: &str) -> std::path::PathBuf {
    root.join(raw.trim_start_matches('/'))
}

fn is_specs_path(raw: &str) -> bool {
    raw.trim_start_matches('/').starts_with("specs/")
}

fn read_check_subject(root: &Path, raw: &str) -> Result<String, String> {
    if is_specs_path(raw) {
        return read_spec_text(root, raw);
    }
    let p = resolve(root, raw);
    fs::read_to_string(&p).map_err(|e| format!("failed to read {}: {e}", p.display()))
}

fn check_subject_exists(root: &Path, raw: &str) -> Result<bool, String> {
    if is_specs_path(raw) {
        return spec_exists(root, raw);
    }
    Ok(resolve(root, raw).exists())
}

fn yaml_str_list(map: &serde_yaml::Mapping, key: &str) -> Vec<String> {
    map.get(&YamlValue::String(key.to_string()))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_manifest(
    root: &Path,
    manifest_ref: &str,
    profile_name: &str,
) -> Result<Vec<serde_yaml::Mapping>, String> {
    let raw = read_check_subject(root, manifest_ref)
        .map_err(|e| format!("failed to read manifest {manifest_ref}: {e}"))?;
    let doc: YamlValue = serde_yaml::from_str(&raw)
        .map_err(|e| format!("failed to parse manifest {manifest_ref}: {e}"))?;
    let root = doc
        .as_mapping()
        .ok_or_else(|| "critical manifest root must be mapping".to_string())?;
    let profiles = root
        .get(&YamlValue::String("profiles".to_string()))
        .and_then(|v| v.as_mapping())
        .ok_or_else(|| "critical manifest missing profiles mapping".to_string())?;
    let profile = profiles
        .get(&YamlValue::String(profile_name.to_string()))
        .and_then(|v| v.as_mapping())
        .ok_or_else(|| format!("critical manifest missing profiles.{profile_name}"))?;
    let checks = profile
        .get(&YamlValue::String("checks".to_string()))
        .and_then(|v| v.as_sequence())
        .ok_or_else(|| format!("critical manifest missing profiles.{profile_name}.checks"))?;
    if checks.is_empty() {
        return Err(format!(
            "critical manifest profiles.{profile_name}.checks must be non-empty"
        ));
    }
    let mut out = Vec::<serde_yaml::Mapping>::new();
    for item in checks {
        if let Some(m) = item.as_mapping() {
            out.push(m.clone());
        }
    }
    if out.is_empty() {
        return Err("critical manifest checks must contain mapping entries".to_string());
    }
    Ok(out)
}

fn run_file_tokens(root: &Path, cfg: &serde_yaml::Mapping) -> (bool, Vec<String>) {
    let path_raw = cfg
        .get(&YamlValue::String("path".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if path_raw.trim().is_empty() {
        return (false, vec!["missing path".to_string()]);
    }
    if !check_subject_exists(root, path_raw).unwrap_or(false) {
        return (false, vec![format!("missing file: {}", path_raw)]);
    }
    let text = match read_check_subject(root, path_raw) {
        Ok(v) => v,
        Err(e) => return (false, vec![e]),
    };
    let must_contain = yaml_str_list(cfg, "must_contain");
    let must_not_contain = yaml_str_list(cfg, "must_not_contain");
    let mut details = Vec::<String>::new();
    let mut ok = true;
    for tok in must_contain {
        if !text.contains(&tok) {
            ok = false;
            details.push(format!("missing token: {tok}"));
        }
    }
    for tok in must_not_contain {
        if text.contains(&tok) {
            ok = false;
            details.push(format!("forbidden token present: {tok}"));
        }
    }
    (ok, details)
}

fn run_file_ordered_tokens(root: &Path, cfg: &serde_yaml::Mapping) -> (bool, Vec<String>) {
    let path_raw = cfg
        .get(&YamlValue::String("path".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if path_raw.trim().is_empty() {
        return (false, vec!["missing path".to_string()]);
    }
    if !check_subject_exists(root, path_raw).unwrap_or(false) {
        return (false, vec![format!("missing file: {}", path_raw)]);
    }
    let text = match read_check_subject(root, path_raw) {
        Ok(v) => v,
        Err(e) => return (false, vec![e]),
    };
    let ordered_tokens = yaml_str_list(cfg, "ordered_tokens");
    if ordered_tokens.len() < 2 {
        return (
            false,
            vec!["ordered_tokens must contain at least two entries".to_string()],
        );
    }
    let mut details = Vec::<String>::new();
    let mut ok = true;
    let mut last_idx = 0usize;
    let mut first = true;
    for tok in ordered_tokens {
        if let Some(found_idx) = text.find(&tok) {
            if !first && found_idx < last_idx {
                ok = false;
                details.push(format!("token order violation: {tok}"));
            }
            last_idx = found_idx;
            first = false;
        } else {
            ok = false;
            details.push(format!("missing ordered token: {tok}"));
        }
    }
    (ok, details)
}

fn run_manifest_non_empty(root: &Path, cfg: &serde_yaml::Mapping) -> (bool, Vec<String>) {
    let path_raw = cfg
        .get(&YamlValue::String("path".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let profile = cfg
        .get(&YamlValue::String("profile".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("critical");
    let key = cfg
        .get(&YamlValue::String("key".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("checks");
    if path_raw.trim().is_empty() {
        return (false, vec!["missing path".to_string()]);
    }
    if !check_subject_exists(root, path_raw).unwrap_or(false) {
        return (false, vec![format!("missing file: {}", path_raw)]);
    }
    let raw = match read_check_subject(root, path_raw) {
        Ok(v) => v,
        Err(e) => return (false, vec![e]),
    };
    let doc: YamlValue = match serde_yaml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => return (false, vec![format!("invalid yaml: {e}")]),
    };
    let root_map = match doc.as_mapping() {
        Some(m) => m,
        None => return (false, vec!["manifest root must be mapping".to_string()]),
    };
    let profiles = match root_map
        .get(&YamlValue::String("profiles".to_string()))
        .and_then(|v| v.as_mapping())
    {
        Some(m) => m,
        None => return (false, vec!["missing profiles mapping".to_string()]),
    };
    let profile_map = match profiles
        .get(&YamlValue::String(profile.to_string()))
        .and_then(|v| v.as_mapping())
    {
        Some(m) => m,
        None => return (false, vec![format!("missing profile: {profile}")]),
    };
    let seq = profile_map
        .get(&YamlValue::String(key.to_string()))
        .and_then(|v| v.as_sequence());
    if seq.is_none() || seq.is_some_and(|s| s.is_empty()) {
        return (false, vec![format!("missing/non-empty key: {key}")]);
    }
    (true, Vec::new())
}

fn run_slo(total_ms: i64, cfg: &serde_yaml::Mapping) -> (bool, Vec<String>) {
    let max_ms = cfg
        .get(&YamlValue::String("max_ms".to_string()))
        .and_then(|v| v.as_i64())
        .unwrap_or(10_000);
    if total_ms <= max_ms {
        (true, Vec::new())
    } else {
        (
            false,
            vec![format!(
                "critical gate exceeded slo: total_ms={total_ms} max_ms={max_ms}"
            )],
        )
    }
}

fn run_governance_profile_native(
    root: &Path,
    forwarded: &[String],
    default_profile: &str,
    default_out_prefix: &str,
    default_prefixes: &[&str],
) -> i32 {
    let mut profile = default_profile.to_string();
    let mut out_path = format!(".artifacts/{default_out_prefix}-summary.json");
    let mut trace_out_path = format!(".artifacts/{default_out_prefix}-trace.json");
    let mut summary_out_path = format!(".artifacts/{default_out_prefix}-summary.md");
    let mut fail_fast = env_bool("SPEC_RUNNER_FAIL_FAST", true);
    let mut check_ids: Vec<String> = Vec::new();
    let mut check_prefixes: Vec<String> = default_prefixes.iter().map(|x| x.to_string()).collect();

    let mut i = 0usize;
    while i < forwarded.len() {
        let arg = forwarded[i].as_str();
        match arg {
            "--out" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --out requires value");
                    return 2;
                }
                out_path = forwarded[i + 1].clone();
                i += 2;
            }
            "--trace-out" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --trace-out requires value");
                    return 2;
                }
                trace_out_path = forwarded[i + 1].clone();
                i += 2;
            }
            "--summary-out" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --summary-out requires value");
                    return 2;
                }
                summary_out_path = forwarded[i + 1].clone();
                i += 2;
            }
            "--profile" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --profile requires value");
                    return 2;
                }
                profile = forwarded[i + 1].clone();
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
            "--check-id" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --check-id requires value");
                    return 2;
                }
                let value = forwarded[i + 1].trim();
                if value.is_empty() {
                    eprintln!("ERROR: --check-id requires non-empty value");
                    return 2;
                }
                check_ids.push(value.to_string());
                i += 2;
            }
            "--check-prefix" => {
                if i + 1 >= forwarded.len() {
                    eprintln!("ERROR: --check-prefix requires value");
                    return 2;
                }
                let value = forwarded[i + 1].trim();
                if value.is_empty() {
                    eprintln!("ERROR: --check-prefix requires non-empty value");
                    return 2;
                }
                check_prefixes.push(value.to_string());
                i += 2;
            }
            _ => {
                eprintln!("ERROR: unsupported critical-gate arg: {arg}");
                return 2;
            }
        }
    }

    let started = now_iso_utc_fallback();
    let t0 = Instant::now();
    let manifest_ref = "/specs/governance/check_sets_v1.yaml";

    let mut checks = Vec::<CriticalCheckResult>::new();
    let mut traces = Vec::<Value>::new();

    let defs = match parse_manifest(root, manifest_ref, &profile) {
        Ok(v) => v,
        Err(e) => {
            checks.push(CriticalCheckResult {
                check_id: "runtime.critical_check_set_manifest_required".to_string(),
                status: "fail".to_string(),
                duration_ms: t0.elapsed().as_millis() as i64,
                reason_token: Some("manifest.invalid".to_string()),
                details: vec![e],
            });
            Vec::new()
        }
    };

    for def in defs {
        let check_id = def
            .get(&YamlValue::String("id".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("runtime.critical_gate.unknown")
            .to_string();
        if !check_ids.is_empty() && !check_ids.iter().any(|id| id == &check_id) {
            continue;
        }
        if !check_prefixes.is_empty()
            && !check_prefixes
                .iter()
                .any(|prefix| check_id.starts_with(prefix))
        {
            continue;
        }
        let kind = def
            .get(&YamlValue::String("kind".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("file_tokens");

        if kind == "slo_budget" {
            continue;
        }

        let c0 = Instant::now();
        let (ok, details) = match kind {
            "file_tokens" => run_file_tokens(root, &def),
            "file_ordered_tokens" => run_file_ordered_tokens(root, &def),
            "manifest_non_empty" => run_manifest_non_empty(root, &def),
            "report_only" => (true, vec!["report-only".to_string()]),
            _ => (false, vec![format!("unsupported check kind: {kind}")]),
        };
        let status = if ok { "pass" } else { "fail" }.to_string();
        let reason = if ok {
            None
        } else if kind == "file_tokens" {
            Some("contract.token_missing".to_string())
        } else if kind == "manifest_non_empty" {
            Some("manifest.empty".to_string())
        } else if kind == "file_ordered_tokens" {
            Some("contract.order_violation".to_string())
        } else {
            Some("check.unsupported".to_string())
        };
        let duration_ms = c0.elapsed().as_millis() as i64;
        checks.push(CriticalCheckResult {
            check_id: check_id.clone(),
            status: status.clone(),
            duration_ms,
            reason_token: reason.clone(),
            details: details.clone(),
        });
        traces.push(json!({
            "check_id": check_id,
            "kind": kind,
            "status": status,
            "duration_ms": duration_ms,
            "reason_token": reason,
            "details": details,
        }));
        if fail_fast && !ok {
            break;
        }
    }

    // run SLO checks at end (from manifest)
    let total_ms = t0.elapsed().as_millis() as i64;
    for def in parse_manifest(root, manifest_ref, &profile).unwrap_or_default() {
        let kind = def
            .get(&YamlValue::String("kind".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if kind != "slo_budget" {
            continue;
        }
        let check_id = def
            .get(&YamlValue::String("id".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("runtime.critical_gate_slo_sub_10s_required")
            .to_string();
        let c0 = Instant::now();
        let (ok, details) = run_slo(total_ms, &def);
        checks.push(CriticalCheckResult {
            check_id,
            status: if ok { "pass" } else { "fail" }.to_string(),
            duration_ms: c0.elapsed().as_millis() as i64,
            reason_token: if ok {
                None
            } else {
                Some("slo.exceeded".to_string())
            },
            details,
        });
    }

    let failed: Vec<&CriticalCheckResult> = checks.iter().filter(|c| c.status != "pass").collect();
    let status = if failed.is_empty() { "pass" } else { "fail" };
    let first_failure_check_id = failed.first().map(|c| c.check_id.clone());
    let finished = now_iso_utc_fallback();

    let summary = json!({
        "version": 1,
        "status": status,
        "runner_impl": "rust",
        "profile": format!("governance.profile.{}", profile),
        "started_at": started,
        "finished_at": finished,
        "total_duration_ms": total_ms,
        "first_failure_check_id": first_failure_check_id,
        "failed_check_count": failed.len(),
        "checks": checks.iter().map(|c| {
            json!({
                "check_id": c.check_id,
                "status": c.status,
                "duration_ms": c.duration_ms,
                "reason_token": c.reason_token,
                "details": c.details,
            })
        }).collect::<Vec<_>>()
    });

    let trace = json!({
        "version": 1,
        "status": status,
        "total_duration_ms": total_ms,
        "events": traces,
    });

    let out_abs = resolve(root, &out_path);
    let trace_abs = resolve(root, &trace_out_path);
    let summary_abs = resolve(root, &summary_out_path);
    if let Some(parent) = out_abs.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Some(parent) = trace_abs.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Some(parent) = summary_abs.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(
        &out_abs,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string())
        ),
    );
    let _ = fs::write(
        &trace_abs,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&trace).unwrap_or_else(|_| "{}".to_string())
        ),
    );

    let mut md = String::new();
    md.push_str("# Critical Gate Summary\n\n");
    md.push_str(&format!("- status: `{}`\n", status));
    md.push_str(&format!("- total_duration_ms: `{}`\n", total_ms));
    md.push_str(&format!(
        "- first_failure_check_id: `{}`\n\n",
        first_failure_check_id.unwrap_or_default()
    ));
    md.push_str("| check_id | status | duration_ms | reason_token |\n");
    md.push_str("|---|---|---|---|\n");
    for c in &checks {
        md.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` |\n",
            c.check_id,
            c.status,
            c.duration_ms,
            c.reason_token.clone().unwrap_or_default()
        ));
    }
    let _ = fs::write(&summary_abs, md);

    println!("wrote {}", out_abs.display());
    println!("wrote {}", trace_abs.display());
    println!("wrote {}", summary_abs.display());

    if status == "pass" {
        0
    } else {
        1
    }
}

pub fn run_governance_native(root: &Path, forwarded: &[String]) -> i32 {
    run_governance_profile_native(root, forwarded, "full", "governance", &[])
}

pub fn run_governance_heavy_native(root: &Path, forwarded: &[String]) -> i32 {
    run_governance_profile_native(
        root,
        forwarded,
        "full",
        "governance-heavy",
        &[
            "runtime.chain",
            "library.",
            "normalization.mapping_ast_only",
            "normalization.virtual_root_paths_only",
        ],
    )
}
