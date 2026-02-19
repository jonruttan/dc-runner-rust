use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::cli::args::OutputFormat;

#[derive(Debug, Clone)]
pub struct SpecCase {
    pub spec_file: String,
    pub case_id: String,
}

#[derive(Debug, Clone)]
pub struct RunAllSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub failed_refs: Vec<String>,
}

fn walk_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, out);
            continue;
        }
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".spec.md"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

pub fn normalize_spec_ref(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('/') || trimmed.starts_with('#') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

pub fn list_specs(root: &Path, path_filter: Option<&str>) -> Result<Vec<SpecCase>, String> {
    let search_root = path_filter
        .map(|p| root.join(p.trim_start_matches('/')))
        .unwrap_or_else(|| root.join("specs"));

    let mut files = Vec::<PathBuf>::new();
    walk_files(&search_root, &mut files);
    files.sort();

    let mut cases = Vec::<SpecCase>::new();
    for file in files {
        let text = fs::read_to_string(&file)
            .map_err(|e| format!("failed to read spec file {}: {e}", file.display()))?;
        for block in crate::domain::refs::extract_spec_test_blocks(&text) {
            if let Some(id) = crate::domain::refs::block_id(&block) {
                let rel = file
                    .strip_prefix(root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");
                cases.push(SpecCase {
                    spec_file: format!("/{rel}"),
                    case_id: id,
                });
            }
        }
    }

    Ok(cases)
}

pub fn print_specs(cases: &[SpecCase], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            if cases.is_empty() {
                println!("No spec cases found.");
                return;
            }
            println!("Discovered {} spec cases:", cases.len());
            for case in cases {
                println!("- {}#{}", case.spec_file, case.case_id);
            }
        }
        OutputFormat::Json => {
            let payload = json!(
                cases
                    .iter()
                    .map(|c| json!({"ref": format!("{}#{}", c.spec_file, c.case_id)}))
                    .collect::<Vec<_>>()
            );
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "[]".to_string())
            );
        }
    }
}

pub fn run_all_specs<F>(cases: &[SpecCase], fail_fast: bool, mut run_one: F) -> RunAllSummary
where
    F: FnMut(&str) -> i32,
{
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failed_refs = Vec::<String>::new();

    for case in cases {
        let spec_ref = format!("{}#{}", case.spec_file, case.case_id);
        let code = run_one(&spec_ref);
        if code == 0 {
            passed += 1;
        } else {
            failed += 1;
            failed_refs.push(spec_ref.clone());
            if fail_fast {
                break;
            }
        }
    }

    RunAllSummary {
        total: cases.len(),
        passed,
        failed,
        failed_refs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ref_adds_root_slash() {
        assert_eq!(normalize_spec_ref("specs/a.spec.md#ID"), "/specs/a.spec.md#ID");
        assert_eq!(normalize_spec_ref("/specs/a.spec.md#ID"), "/specs/a.spec.md#ID");
    }
}
