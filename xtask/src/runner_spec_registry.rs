use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

const REGISTRY_PATH: &str = "specs/impl/rust/runner_spec_registry_v1.yaml";
const VENDORED_ROOT: &str = "specs/upstream/data-contracts-library";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    version: u32,
    cases: Vec<CaseEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseEntry {
    id: String,
    path: String,
    purpose: String,
    owner: String,
    status: String,
    replaced_by: Option<String>,
}

fn repo_root() -> Result<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("failed to resolve repository root"))
}

pub fn validate_runner_spec_registry() -> Result<usize> {
    let root = repo_root()?;
    let registry_path = root.join(REGISTRY_PATH);
    let vendored_root = root.join(VENDORED_ROOT);

    if !registry_path.is_file() {
        bail!("runner spec registry missing: {}", registry_path.display());
    }
    if !vendored_root.is_dir() {
        bail!("runner spec snapshot missing: {}", vendored_root.display());
    }

    let txt = fs::read_to_string(&registry_path)
        .with_context(|| format!("failed reading {}", registry_path.display()))?;
    let registry: Registry =
        serde_yaml::from_str(&txt).context("failed parsing runner spec registry YAML")?;

    if registry.version != 1 {
        bail!("runner spec registry version must be 1");
    }

    let mut seen = HashSet::new();
    for case in &registry.cases {
        if !seen.insert(case.id.clone()) {
            bail!("duplicate rust spec case id in registry: {}", case.id);
        }
        if case.id.trim().is_empty()
            || case.path.trim().is_empty()
            || case.purpose.trim().is_empty()
            || case.owner.trim().is_empty()
        {
            bail!("registry case fields must be non-empty for id: {}", case.id);
        }

        match case.status.as_str() {
            "active" | "migrating" => {}
            "deprecated" => {
                if case.replaced_by.as_deref().unwrap_or("").trim().is_empty() {
                    bail!(
                        "deprecated rust spec case must include replaced_by: {}",
                        case.id
                    );
                }
            }
            other => bail!("invalid rust spec case status '{}': {}", other, case.id),
        }

        let rel = case.path.trim_start_matches('/');
        let path = vendored_root.join(rel);
        if !path.is_file() {
            bail!(
                "registry path does not exist in vendored runner spec: {}",
                case.path
            );
        }

        assert_case_id_present(&path, &case.id)?;
    }

    Ok(registry.cases.len())
}

fn assert_case_id_present(path: &Path, case_id: &str) -> Result<()> {
    let txt =
        fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))?;
    let expected = format!("id: {case_id}");
    if !txt.contains(&expected) {
        bail!(
            "registry id '{}' not found in target spec file {}",
            case_id,
            path.display()
        );
    }
    Ok(())
}
