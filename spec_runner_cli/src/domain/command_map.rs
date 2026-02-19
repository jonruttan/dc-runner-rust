pub fn command_spec_ref(subcommand: &str) -> Option<&'static str> {
    match subcommand {
        "validate-report" => Some(
            "/specs/libraries/domain/conformance_core.spec.md#LIB-DOMAIN-CONFORMANCE-001-000C-DOMAIN-CONFORMANCE-VALIDATE-REPORT-ERRORS",
        ),
        "schema-registry-build" => {
            Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-004")
        }
        "schema-registry-check" => {
            Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-005")
        }
        "docs-lint" => Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-006"),
        "docs-generate" => {
            Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-007")
        }
        "docs-generate-check" => {
            Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-008")
        }
        "docs-build" => Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-009"),
        "docs-build-check" => {
            Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-010")
        }
        "docs-graph" => Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-011"),
        "runner-certify" => {
            Some(
                "/specs/upstream/data-contracts/specs/conformance/cases/core/runner_certification_core.spec.md#DCCONF-RCERT-001",
            )
        }
        "spec-lang-stdlib-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-017")
        }
        "spec-lang-stdlib-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-018")
        }
        "contract-assertions-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-005")
        }
        "contract-assertions-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-006")
        }
        "conformance-purpose-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-001")
        }
        "conformance-purpose-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-002")
        }
        "spec-portability-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-003")
        }
        "spec-portability-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-004")
        }
        "spec-lang-adoption-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-007")
        }
        "spec-lang-adoption-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-008")
        }
        "runner-independence-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-009")
        }
        "runner-independence-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-010")
        }
        "python-dependency-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-011")
        }
        "python-dependency-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-012")
        }
        "docs-operability-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-013")
        }
        "docs-operability-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-014")
        }
        "objective-scorecard-json" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-015")
        }
        "objective-scorecard-md" => {
            Some("/specs/impl/rust/jobs/report_jobs.spec.md#DCIMPL-RUST-JOB-REP-016")
        }
        "conformance-parity" => {
            Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-002")
        }
        "perf-smoke" => Some("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-003"),
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
}
