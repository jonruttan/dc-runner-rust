pub fn command_spec_ref(subcommand: &str) -> Option<&'static str> {
    match subcommand {
        "validate-report" => Some(
            "/specs/05_libraries/domain/conformance_core.spec.md#LIB-DOMAIN-CONFORMANCE-001-000C-DOMAIN-CONFORMANCE-VALIDATE-REPORT-ERRORS",
        ),
        "schema-check" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-012",
            )
        }
        "schema-lint" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-013",
            )
        }
        "schema-format" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-014",
            )
        }
        "docs-lint" => Some(
            "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-006",
        ),
        "docs-generate" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-007",
            )
        }
        "docs-generate-check" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-008",
            )
        }
        "docs-build" => Some(
            "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-009",
        ),
        "docs-build-check" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-010",
            )
        }
        "docs-graph" => Some(
            "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_command_jobs.spec.md#DCCONF-JOB-011",
        ),
        "runner-certify" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/runner_certification_core.spec.md#DCCONF-RCERT-001",
            )
        }
        "spec-lang-stdlib-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-017",
            )
        }
        "spec-lang-stdlib-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-018",
            )
        }
        "contract-assertions-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-005",
            )
        }
        "contract-assertions-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-006",
            )
        }
        "conformance-purpose-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-001",
            )
        }
        "conformance-purpose-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-002",
            )
        }
        "spec-portability-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-003",
            )
        }
        "spec-portability-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-004",
            )
        }
        "spec-lang-adoption-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-007",
            )
        }
        "spec-lang-adoption-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-008",
            )
        }
        "runner-independence-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-009",
            )
        }
        "runner-independence-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-010",
            )
        }
        "python-dependency-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-011",
            )
        }
        "python-dependency-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-012",
            )
        }
        "docs-operability-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-013",
            )
        }
        "docs-operability-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-014",
            )
        }
        "objective-scorecard-json" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-015",
            )
        }
        "objective-scorecard-md" => {
            Some(
                "/specs/upstream/data-contracts/specs/03_conformance/cases/core/report_job_contracts.spec.md#DCCONF-JOB-REP-016",
            )
        }
        "conformance-parity" => {
            Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-002")
        }
        "perf-smoke" => Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-003"),
        "bundler-resolve" => Some(
            "/specs/upstream/data-contracts-library/specs/05_libraries/bundle_tooling/bundler_job_contracts.spec.md#DCLIB-BUNDLE-JOB-001",
        ),
        "bundler-package" => Some(
            "/specs/upstream/data-contracts-library/specs/05_libraries/bundle_tooling/bundler_job_contracts.spec.md#DCLIB-BUNDLE-JOB-002",
        ),
        "bundler-check" => Some(
            "/specs/upstream/data-contracts-library/specs/05_libraries/bundle_tooling/bundler_job_contracts.spec.md#DCLIB-BUNDLE-JOB-003",
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_spec_ref_has_validate_report_mapping() {
        let got = command_spec_ref("validate-report");
        assert!(got.is_some());
        assert!(got.expect("mapping").contains('#'));
    }

    #[test]
    fn command_spec_ref_does_not_keep_removed_bundle_placeholders() {
        for command in ["bundle-list", "bundle-inspect", "bundle-install"] {
            assert_eq!(command_spec_ref(command), None, "unexpected mapping kept for {command}");
        }
    }
}
