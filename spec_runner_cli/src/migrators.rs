use serde_yaml::{Mapping, Value as YamlValue};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const START_FENCE: &str = "```yaml contract-spec\n";
const END_FENCE: &str = "\n```";

#[derive(Default)]
struct CommonOpts {
    check: bool,
    write: bool,
    paths: Vec<String>,
}

fn parse_common_opts(args: &[String], default_path: &str) -> Result<CommonOpts, String> {
    let mut out = CommonOpts::default();
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => out.check = true,
            "--write" => out.write = true,
            "--root" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    return Err("missing value for --root".to_string());
                };
                out.paths.push(path.clone());
            }
            other if other.starts_with("--") => {
                return Err(format!("unsupported argument: {other}"));
            }
            raw => out.paths.push(raw.to_string()),
        }
        i += 1;
    }
    if out.check == out.write {
        return Err("choose exactly one of --check or --write".to_string());
    }
    if out.paths.is_empty() {
        out.paths.push(default_path.to_string());
    }
    Ok(out)
}

fn walk_spec_files(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = Vec::new();
    for raw in paths {
        let path = PathBuf::from(raw);
        if path.is_file() {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".spec.md"))
                .unwrap_or(false)
            {
                files.push(path);
            }
            continue;
        }
        if !path.exists() {
            continue;
        }
        let mut stack = vec![path];
        while let Some(cur) = stack.pop() {
            let rd = fs::read_dir(&cur)
                .map_err(|e| format!("failed reading directory {}: {e}", cur.display()))?;
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.is_file()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.ends_with(".spec.md"))
                        .unwrap_or(false)
                {
                    files.push(p);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

fn key(name: &str) -> YamlValue {
    YamlValue::String(name.to_string())
}

fn as_mapping_mut(v: &mut YamlValue) -> Option<&mut Mapping> {
    if let YamlValue::Mapping(m) = v {
        Some(m)
    } else {
        None
    }
}

fn as_mapping(v: &YamlValue) -> Option<&Mapping> {
    if let YamlValue::Mapping(m) = v {
        Some(m)
    } else {
        None
    }
}

fn as_sequence_mut(v: &mut YamlValue) -> Option<&mut Vec<YamlValue>> {
    if let YamlValue::Sequence(seq) = v {
        Some(seq)
    } else {
        None
    }
}

fn as_sequence(v: &YamlValue) -> Option<&Vec<YamlValue>> {
    if let YamlValue::Sequence(seq) = v {
        Some(seq)
    } else {
        None
    }
}

fn yaml_string(v: &YamlValue) -> Option<String> {
    v.as_str().map(|s| s.to_string())
}

fn dump_yaml(value: &YamlValue) -> Result<String, String> {
    let raw = serde_yaml::to_string(value).map_err(|e| format!("failed to render yaml: {e}"))?;
    let stripped = raw.strip_prefix("---\n").unwrap_or(&raw);
    Ok(stripped.trim_end_matches('\n').to_string())
}

fn rewrite_contract_spec_blocks<F>(text: &str, mut transform: F) -> Result<(String, bool), String>
where
    F: FnMut(&mut YamlValue) -> Result<bool, String>,
{
    let mut out = String::with_capacity(text.len());
    let mut idx = 0usize;
    let mut changed = false;
    while let Some(rel_start) = text[idx..].find(START_FENCE) {
        let start = idx + rel_start;
        out.push_str(&text[idx..start]);
        let block_start = start + START_FENCE.len();
        let Some(rel_end) = text[block_start..].find(END_FENCE) else {
            return Err("unterminated contract-spec fence".to_string());
        };
        let block_end = block_start + rel_end;
        let block = &text[block_start..block_end];
        let mut parsed: YamlValue = serde_yaml::from_str(block)
            .map_err(|e| format!("failed parsing contract-spec yaml block: {e}"))?;
        let block_changed = transform(&mut parsed)?;
        if block_changed {
            changed = true;
            out.push_str(START_FENCE);
            out.push_str(&dump_yaml(&parsed)?);
            out.push_str(END_FENCE);
        } else {
            out.push_str(START_FENCE);
            out.push_str(block);
            out.push_str(END_FENCE);
        }
        idx = block_end + END_FENCE.len();
    }
    out.push_str(&text[idx..]);
    Ok((out, changed))
}

fn infer_domain_from_path(path: &Path) -> Option<String> {
    let rel = path.to_string_lossy().to_string();
    let parts: Vec<&str> = rel.split('/').collect();
    let pos = parts.iter().position(|p| *p == "libraries")?;
    let candidate = parts.get(pos + 1)?.trim();
    if candidate.is_empty() {
        None
    } else {
        Some(candidate.to_string())
    }
}

fn normalize_domain(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("domain must be a non-empty string".to_string());
    }
    Ok(value.to_string())
}

fn normalize_export_symbol(domain: Option<&str>, raw_as: &str) -> Result<String, String> {
    let name = raw_as.trim();
    if name.is_empty() {
        return Err("harness.exports[].as must be a non-empty string".to_string());
    }
    match domain {
        None => Ok(name.to_string()),
        Some(d) => {
            let prefix = format!("{d}.");
            if name.starts_with(&prefix) {
                Ok(name.to_string())
            } else {
                Ok(format!("{prefix}{name}"))
            }
        }
    }
}

fn rewrite_var_nodes(node: &mut YamlValue, rename_map: &HashMap<String, String>) -> i64 {
    match node {
        YamlValue::Mapping(map) => {
            let mut rewrites = 0_i64;
            if let Some(var_raw) = map.get(&key("var")).and_then(yaml_string) {
                if let Some(new_name) = rename_map.get(&var_raw) {
                    map.insert(key("var"), YamlValue::String(new_name.clone()));
                    rewrites += 1;
                }
            }
            for (_k, value) in map.iter_mut() {
                rewrites += rewrite_var_nodes(value, rename_map);
            }
            rewrites
        }
        YamlValue::Sequence(seq) => seq
            .iter_mut()
            .map(|v| rewrite_var_nodes(v, rename_map))
            .sum(),
        _ => 0,
    }
}

fn rewrite_case_refs(case: &mut Mapping, rename_map: &HashMap<String, String>) -> i64 {
    let mut rewrites = 0_i64;
    if let Some(harness) = case.get_mut(&key("harness")).and_then(as_mapping_mut) {
        if let Some(use_seq) = harness.get_mut(&key("use")).and_then(as_sequence_mut) {
            for item in use_seq.iter_mut() {
                if let Some(use_item) = as_mapping_mut(item) {
                    if let Some(symbols) =
                        use_item.get_mut(&key("symbols")).and_then(as_sequence_mut)
                    {
                        for symbol in symbols.iter_mut() {
                            if let Some(raw) = symbol.as_str() {
                                if let Some(new_name) = rename_map.get(raw) {
                                    *symbol = YamlValue::String(new_name.clone());
                                    rewrites += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(spec_lang) = harness.get_mut(&key("spec_lang")).and_then(as_mapping_mut) {
            if let Some(exports) = spec_lang.get_mut(&key("exports")).and_then(as_sequence_mut) {
                for symbol in exports.iter_mut() {
                    if let Some(raw) = symbol.as_str() {
                        if let Some(new_name) = rename_map.get(raw) {
                            *symbol = YamlValue::String(new_name.clone());
                            rewrites += 1;
                        }
                    }
                }
            }
        }
    }

    if let Some(contract) = case.get_mut(&key("contract")).and_then(as_mapping_mut) {
        if let Some(steps) = contract.get_mut(&key("steps")).and_then(as_sequence_mut) {
            for step in steps.iter_mut() {
                if let Some(step_map) = as_mapping_mut(step) {
                    if let Some(assert_node) = step_map.get_mut(&key("assert")) {
                        rewrites += rewrite_var_nodes(assert_node, rename_map);
                    }
                }
            }
        }
    }
    rewrites
}

pub fn run_migrate_case_doc_metadata_v1(root: &Path, args: &[String]) -> i32 {
    let opts = match parse_common_opts(args, "specs") {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("ERROR: {msg}");
            return 2;
        }
    };
    let files = match walk_spec_files(&opts.paths) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("ERROR: {msg}");
            return 2;
        }
    };
    let mut changed_files: Vec<String> = Vec::new();
    for rel in files {
        let path = if rel.is_absolute() {
            rel
        } else {
            root.join(rel)
        };
        let raw = match fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: failed to read {}: {e}", path.display());
                return 1;
            }
        };
        let res = rewrite_contract_spec_blocks(&raw, |case| {
            let Some(m) = as_mapping_mut(case) else {
                return Ok(false);
            };
            if m.get(&key("type")).and_then(|v| v.as_str()) != Some("contract.export") {
                return Ok(false);
            }
            if m.get(&key("doc")).and_then(as_mapping).is_some() {
                return Ok(false);
            }
            let case_id = m
                .get(&key("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("<case>");
            let case_type = m
                .get(&key("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("contract.export");
            let mut doc = Mapping::new();
            doc.insert(
                key("summary"),
                YamlValue::String(format!("Case `{case_id}` for `{case_type}`.")),
            );
            doc.insert(
                key("description"),
                YamlValue::String(
                    "Auto-generated root doc metadata stub. Replace with authored reference text."
                        .to_string(),
                ),
            );
            doc.insert(
                key("audience"),
                YamlValue::String("spec-authors".to_string()),
            );
            doc.insert(key("since"), YamlValue::String("v1".to_string()));
            doc.insert(
                key("tags"),
                YamlValue::Sequence(vec![YamlValue::String(case_type.to_string())]),
            );
            doc.insert(key("see_also"), YamlValue::Sequence(vec![]));
            m.insert(key("doc"), YamlValue::Mapping(doc));
            Ok(true)
        });
        let (updated, changed) = match res {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("ERROR: {}: {msg}", path.display());
                return 1;
            }
        };
        if changed {
            changed_files.push(path.to_string_lossy().to_string());
            if opts.write {
                if let Err(e) = fs::write(&path, updated) {
                    eprintln!("ERROR: failed writing {}: {e}", path.display());
                    return 1;
                }
            }
        }
    }
    if opts.check {
        if changed_files.is_empty() {
            println!("OK: root doc metadata is current");
            0
        } else {
            println!("metadata migration required:");
            for file in changed_files {
                println!("- {file}");
            }
            1
        }
    } else {
        if changed_files.is_empty() {
            println!("no changes");
        } else {
            println!("updated files:");
            for file in changed_files {
                println!("- {file}");
            }
        }
        0
    }
}

pub fn run_migrate_library_docs_metadata_v1(root: &Path, args: &[String]) -> i32 {
    let opts = match parse_common_opts(args, "specs/libraries") {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("ERROR: {msg}");
            return 2;
        }
    };
    let files = match walk_spec_files(&opts.paths) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("ERROR: {msg}");
            return 2;
        }
    };
    let mut changed_files: Vec<String> = Vec::new();
    for rel in files {
        let path = if rel.is_absolute() {
            rel
        } else {
            root.join(rel)
        };
        let raw = match fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: failed to read {}: {e}", path.display());
                return 1;
            }
        };
        let rel_path = path.to_string_lossy().to_string();
        let rel_parts: Vec<&str> = rel_path.split('/').collect();
        let module = if rel_parts.len() > 3 {
            rel_parts[2].to_string()
        } else {
            "general".to_string()
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("case")
            .replace(".spec", "");
        let suffix = stem.replace('_', ".");
        let library_id = format!("{module}.{suffix}");

        let res = rewrite_contract_spec_blocks(&raw, |case| {
            let Some(m) = as_mapping_mut(case) else {
                return Ok(false);
            };
            if m.get(&key("type")).and_then(|v| v.as_str()) != Some("contract.export") {
                return Ok(false);
            }
            let mut changed = false;
            if m.get(&key("library")).and_then(as_mapping).is_none() {
                let mut lib = Mapping::new();
                lib.insert(key("id"), YamlValue::String(library_id.clone()));
                lib.insert(key("module"), YamlValue::String(module.clone()));
                lib.insert(key("stability"), YamlValue::String("alpha".to_string()));
                lib.insert(
                    key("owner"),
                    YamlValue::String("data-contracts".to_string()),
                );
                lib.insert(
                    key("tags"),
                    YamlValue::Sequence(vec![YamlValue::String(module.clone())]),
                );
                m.insert(key("library"), YamlValue::Mapping(lib));
                changed = true;
            }

            let Some(harness) = m.get_mut(&key("harness")).and_then(as_mapping_mut) else {
                return Ok(changed);
            };
            let Some(exports) = harness.get_mut(&key("exports")).and_then(as_sequence_mut) else {
                return Ok(changed);
            };
            for item in exports.iter_mut() {
                let Some(exp) = as_mapping_mut(item) else {
                    continue;
                };
                if exp.get(&key("doc")).and_then(as_mapping).is_some() {
                    continue;
                }
                let symbol = exp
                    .get(&key("as"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("<symbol>")
                    .to_string();
                let params: Vec<String> = exp
                    .get(&key("params"))
                    .and_then(as_sequence)
                    .map(|seq| {
                        seq.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let mut doc = Mapping::new();
                doc.insert(
                    key("summary"),
                    YamlValue::String(format!("Contract export for `{symbol}`.")),
                );
                doc.insert(
                    key("description"),
                    YamlValue::String(
                        "Auto-generated metadata stub. Replace with authored reference text."
                            .to_string(),
                    ),
                );
                let mut params_nodes: Vec<YamlValue> = Vec::new();
                for p in &params {
                    let mut node = Mapping::new();
                    node.insert(key("name"), YamlValue::String(p.clone()));
                    node.insert(key("type"), YamlValue::String("any".to_string()));
                    node.insert(key("required"), YamlValue::Bool(true));
                    node.insert(
                        key("description"),
                        YamlValue::String(format!("Input parameter `{p}`.")),
                    );
                    params_nodes.push(YamlValue::Mapping(node));
                }
                doc.insert(key("params"), YamlValue::Sequence(params_nodes));
                let mut returns = Mapping::new();
                returns.insert(key("type"), YamlValue::String("any".to_string()));
                returns.insert(
                    key("description"),
                    YamlValue::String("Result payload for this symbol.".to_string()),
                );
                doc.insert(key("returns"), YamlValue::Mapping(returns));
                let mut error = Mapping::new();
                error.insert(key("code"), YamlValue::String("SCHEMA_ERROR".to_string()));
                error.insert(
                    key("when"),
                    YamlValue::String(
                        "Input payload does not satisfy contract shape requirements.".to_string(),
                    ),
                );
                error.insert(key("category"), YamlValue::String("schema".to_string()));
                doc.insert(
                    key("errors"),
                    YamlValue::Sequence(vec![YamlValue::Mapping(error)]),
                );
                let mut input = Mapping::new();
                for p in &params {
                    input.insert(key(p), YamlValue::String(format!("<{p}>")));
                }
                let mut ex = Mapping::new();
                ex.insert(key("title"), YamlValue::String("Basic usage".to_string()));
                ex.insert(key("input"), YamlValue::Mapping(input));
                ex.insert(key("expected"), YamlValue::String("<result>".to_string()));
                ex.insert(
                    key("notes"),
                    YamlValue::String("Replace with a concrete scenario.".to_string()),
                );
                doc.insert(
                    key("examples"),
                    YamlValue::Sequence(vec![YamlValue::Mapping(ex)]),
                );
                let mut portability = Mapping::new();
                portability.insert(key("python"), YamlValue::Bool(true));
                portability.insert(key("php"), YamlValue::Bool(true));
                portability.insert(key("rust"), YamlValue::Bool(true));
                portability.insert(
                    key("notes"),
                    YamlValue::String("Confirm per-runtime behavior and caveats.".to_string()),
                );
                doc.insert(key("portability"), YamlValue::Mapping(portability));
                doc.insert(key("see_also"), YamlValue::Sequence(vec![]));
                doc.insert(key("since"), YamlValue::String("v1".to_string()));
                exp.insert(key("doc"), YamlValue::Mapping(doc));
                changed = true;
            }
            Ok(changed)
        });
        let (updated, changed) = match res {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("ERROR: {}: {msg}", path.display());
                return 1;
            }
        };
        if changed {
            changed_files.push(path.to_string_lossy().to_string());
            if opts.write {
                if let Err(e) = fs::write(&path, updated) {
                    eprintln!("ERROR: failed writing {}: {e}", path.display());
                    return 1;
                }
            }
        }
    }
    if opts.check {
        if changed_files.is_empty() {
            println!("OK: library metadata is current");
            0
        } else {
            println!("metadata migration required:");
            for file in changed_files {
                println!("- {file}");
            }
            1
        }
    } else {
        if changed_files.is_empty() {
            println!("no changes");
        } else {
            println!("updated files:");
            for file in changed_files {
                println!("- {file}");
            }
        }
        0
    }
}

pub fn run_migrate_case_domain_prefix_v1(root: &Path, args: &[String]) -> i32 {
    let mut check = false;
    let mut write = false;
    let mut explicit_domain: Option<String> = None;
    let mut paths: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => check = true,
            "--write" => write = true,
            "--domain" => {
                i += 1;
                let Some(raw) = args.get(i) else {
                    eprintln!("ERROR: missing value for --domain");
                    return 2;
                };
                match normalize_domain(raw) {
                    Ok(v) => explicit_domain = Some(v),
                    Err(msg) => {
                        eprintln!("ERROR: invalid --domain ({msg})");
                        return 2;
                    }
                }
            }
            other if other.starts_with("--") => {
                eprintln!("ERROR: unsupported argument: {other}");
                return 2;
            }
            raw => paths.push(raw.to_string()),
        }
        i += 1;
    }
    if check == write {
        eprintln!("ERROR: choose exactly one of --check or --write");
        return 2;
    }
    if paths.is_empty() {
        paths.push("specs".to_string());
    }
    let files = match walk_spec_files(&paths) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("ERROR: {msg}");
            return 2;
        }
    };

    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut planned: Vec<String> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();

    for rel in &files {
        let path = if rel.is_absolute() {
            rel.clone()
        } else {
            root.join(rel)
        };
        let text = match fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: failed to read {}: {e}", path.display());
                return 1;
            }
        };
        let res = rewrite_contract_spec_blocks(&text, |case| {
            let Some(m) = as_mapping(case) else {
                return Ok(false);
            };
            if m.get(&key("type")).and_then(|v| v.as_str()) != Some("contract.export") {
                return Ok(false);
            }
            if m.get(&key("domain")).is_some() {
                return Ok(false);
            }
            let inferred = explicit_domain
                .clone()
                .or_else(|| infer_domain_from_path(&path));
            let Some(domain) = inferred else {
                let case_id = m
                    .get(&key("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                unresolved.push(format!(
                    "{}: case {}: unable to infer domain (pass --domain)",
                    path.display(),
                    case_id
                ));
                return Ok(false);
            };
            let Some(harness) = m.get(&key("harness")).and_then(as_mapping) else {
                return Ok(false);
            };
            let Some(exports) = harness.get(&key("exports")).and_then(as_sequence) else {
                return Ok(false);
            };
            let case_id = m
                .get(&key("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            for item in exports {
                let Some(export_map) = as_mapping(item) else {
                    continue;
                };
                let Some(raw_as) = export_map.get(&key("as")).and_then(|v| v.as_str()) else {
                    continue;
                };
                let canonical = normalize_export_symbol(Some(&domain), raw_as)?;
                if canonical != raw_as {
                    rename_map.insert(raw_as.to_string(), canonical.clone());
                    planned.push(format!(
                        "{}: case {}: {} -> {} (domain={})",
                        path.display(),
                        case_id,
                        raw_as,
                        canonical,
                        domain
                    ));
                }
            }
            Ok(false)
        });
        if let Err(msg) = res {
            eprintln!("ERROR: {}: {msg}", path.display());
            return 1;
        }
    }

    if check {
        if !planned.is_empty() {
            println!("planned export renames:");
            for row in &planned {
                println!("- {row}");
            }
        }
        if !unresolved.is_empty() {
            println!("unresolved cases:");
            unresolved.sort();
            unresolved.dedup();
            for row in &unresolved {
                println!("- {row}");
            }
        }
        return if planned.is_empty() && unresolved.is_empty() {
            println!("OK: no domain-prefix migration required");
            0
        } else {
            1
        };
    }

    let mut updated_contracts = 0_i64;
    let mut rewritten_symbols = 0_i64;
    for rel in files {
        let path = if rel.is_absolute() {
            rel
        } else {
            root.join(rel)
        };
        let text = match fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: failed to read {}: {e}", path.display());
                return 1;
            }
        };
        let res = rewrite_contract_spec_blocks(&text, |case| {
            let Some(m) = as_mapping_mut(case) else {
                return Ok(false);
            };
            let mut changed = false;
            if m.get(&key("type")).and_then(|v| v.as_str()) == Some("contract.export")
                && m.get(&key("domain")).is_none()
            {
                let inferred = explicit_domain
                    .clone()
                    .or_else(|| infer_domain_from_path(&path));
                let Some(domain) = inferred else {
                    let case_id = m
                        .get(&key("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unknown>");
                    unresolved.push(format!(
                        "{}: case {}: unable to infer domain (pass --domain)",
                        path.display(),
                        case_id
                    ));
                    return Ok(false);
                };
                m.insert(key("domain"), YamlValue::String(domain));
                updated_contracts += 1;
                changed = true;
            }
            let rewrites = rewrite_case_refs(m, &rename_map);
            if rewrites > 0 {
                rewritten_symbols += rewrites;
                changed = true;
            }
            Ok(changed)
        });
        let (updated, changed) = match res {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("ERROR: {}: {msg}", path.display());
                return 1;
            }
        };
        if changed {
            if let Err(e) = fs::write(&path, updated) {
                eprintln!("ERROR: failed writing {}: {e}", path.display());
                return 1;
            }
        }
    }

    if !planned.is_empty() {
        println!("planned export renames:");
        for row in &planned {
            println!("- {row}");
        }
    }
    if !unresolved.is_empty() {
        println!("unresolved cases:");
        unresolved.sort();
        unresolved.dedup();
        for row in &unresolved {
            println!("- {row}");
        }
    }
    println!("updated contract.export domain blocks: {updated_contracts}");
    println!("rewritten symbol references: {rewritten_symbols}");
    if unresolved.is_empty() {
        0
    } else {
        1
    }
}
