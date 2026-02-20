pub const DEFAULT_SOURCE: &str = "https://github.com/jonruttan/data-contracts.git";
pub const LOCK_FILE: &str = "specs/upstream/data_contracts_lock_v1.yaml";
pub const SNAP_ROOT: &str = "specs/upstream/data-contracts";
pub const MANIFEST_FILE: &str = "specs/upstream/data-contracts.manifest.sha256";
pub const DEFAULT_RUNNER_SPEC_SOURCE: &str = "https://github.com/jonruttan/dc-runner-spec.git";
pub const RUNNER_SPEC_LOCK_FILE: &str = "specs/upstream/dc_runner_spec_lock_v1.yaml";
pub const RUNNER_SPEC_SNAP_ROOT: &str = "specs/upstream/dc-runner-spec";
pub const RUNNER_SPEC_MANIFEST_FILE: &str = "specs/upstream/dc-runner-spec.manifest.sha256";

pub const INCLUDE_PATTERNS: &[&str] = &[
    "specs/index.md",
    "specs/current.md",
    "specs/contract/**",
    "specs/schema/**",
    "specs/conformance/**",
    "specs/governance/index.md",
    "specs/governance/check_*.yaml",
    "specs/governance/cases/core/**",
    "specs/governance/metrics/**",
];

pub const EXCLUDE_PATTERNS: &[&str] = &["**/pending/**", "**/reviews/**", "**/snapshots/**"];
pub const RUNNER_SPEC_EXCLUDE_PATTERNS: &[&str] =
    &["**/pending/**", "**/reviews/**", "**/snapshots/**"];

pub const REQUIRED_SNAPSHOT_FILES: &[&str] = &[
    "specs/index.md",
    "specs/current.md",
    "specs/contract/index.md",
    "specs/contract/policy_v1.yaml",
    "specs/contract/traceability_v1.yaml",
    "specs/schema/index.md",
    "specs/schema/runner_certification_registry_v1.yaml",
    "specs/schema/dc_runner_rust_lock_v1.yaml",
    "specs/governance/index.md",
    "specs/governance/check_sets_v1.yaml",
    "specs/governance/check_prefix_map_v1.yaml",
    "specs/governance/cases/core/index.md",
];

pub const RUNNER_SPEC_INCLUDE_PATTERNS: &[&str] = &[
    "specs/index.md",
    "specs/impl/rust/index.md",
    "specs/impl/rust/jobs/**",
    "specs/impl/python/index.md",
    "specs/impl/php/index.md",
];

pub const REQUIRED_RUNNER_SPEC_FILES: &[&str] = &[
    "specs/index.md",
    "specs/impl/rust/index.md",
    "specs/impl/rust/jobs/script_jobs.spec.md",
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
