pub const DEFAULT_SOURCE: &str = "https://github.com/jonruttan/data-contracts.git";
pub const LOCK_FILE: &str = "specs/upstream/data_contracts_lock_v1.yaml";
pub const SNAP_ROOT: &str = "specs/upstream/data-contracts";
pub const MANIFEST_FILE: &str = "specs/upstream/data-contracts.manifest.sha256";
pub const DEFAULT_RUNNER_SPEC_SOURCE: &str =
    "https://github.com/jonruttan/data-contracts-library.git";
pub const RUNNER_SPEC_LOCK_FILE: &str = "specs/upstream/dc_runner_spec_lock_v1.yaml";
pub const RUNNER_SPEC_SNAP_ROOT: &str = "specs/upstream/data-contracts-library";
pub const RUNNER_SPEC_MANIFEST_FILE: &str = "specs/upstream/data-contracts-library.manifest.sha256";

pub const INCLUDE_PATTERNS: &[&str] = &[
    "specs/index.md",
    "specs/00_core/current.md",
    "specs/02_contracts/**",
    "specs/01_schema/**",
    "specs/03_conformance/**",
    "specs/04_governance/index.md",
    "specs/04_governance/check_*.yaml",
    "specs/04_governance/cases/core/**",
    "specs/04_governance/metrics/**",
];

pub const EXCLUDE_PATTERNS: &[&str] = &["**/pending/**", "**/reviews/**", "**/snapshots/**"];
pub const RUNNER_SPEC_EXCLUDE_PATTERNS: &[&str] =
    &["**/pending/**", "**/reviews/**", "**/snapshots/**"];

pub const REQUIRED_SNAPSHOT_FILES: &[&str] = &[
    "specs/index.md",
    "specs/00_core/current.md",
    "specs/02_contracts/index.md",
    "specs/02_contracts/policy_v1.yaml",
    "specs/02_contracts/traceability_v1.yaml",
    "specs/01_schema/index.md",
    "specs/01_schema/runner_certification_registry_v1.yaml",
    "specs/01_schema/dc_runner_rust_lock_v1.yaml",
    "specs/04_governance/index.md",
    "specs/04_governance/check_sets_v1.yaml",
    "specs/04_governance/check_prefix_map_v1.yaml",
    "specs/04_governance/cases/core/index.md",
];

pub const RUNNER_SPEC_INCLUDE_PATTERNS: &[&str] = &[
    "specs/index.md",
    "specs/07_runner_behavior/contract_sets/**",
    "specs/05_libraries/policy/**",
    "specs/07_runner_behavior/impl/shared/**",
    "specs/07_runner_behavior/impl/rust/index.md",
    "specs/07_runner_behavior/impl/rust/jobs/**",
    "specs/07_runner_behavior/impl/python/index.md",
    "specs/07_runner_behavior/impl/php/index.md",
];

pub const REQUIRED_RUNNER_SPEC_FILES: &[&str] = &[
    "specs/index.md",
    "specs/07_runner_behavior/contract_sets/index.md",
    "specs/07_runner_behavior/impl/rust/index.md",
    "specs/07_runner_behavior/impl/rust/jobs/script_jobs.spec.md",
];

pub const REQUIRED_SUBCOMMAND_FALLBACK: &[&str] = &[
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
];
