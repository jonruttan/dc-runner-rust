use std::path::Path;

pub fn dispatch(root: &Path, subcommand: &str, forwarded: &[String]) -> i32 {
    match subcommand {
        "spec-eval" => super::run_spec_eval_native(root, forwarded),
        "job-run" => super::run_job_run_native(root, forwarded),
        "critical-gate" => super::run_critical_gate_native(root, forwarded),
        "governance-broad-native" => super::run_governance_broad_native(root, forwarded),
        "spec-ref" => {
            if forwarded.len() != 1 {
                eprintln!("usage: spec-ref <subcommand>");
                2
            } else {
                super::run_spec_ref_print(&forwarded[0])
            }
        }
        "validate-report" => super::run_validate_report_native(root, forwarded),
        "governance" => super::run_governance_native(root, forwarded),
        "governance-heavy" => super::run_governance_heavy_native(root, forwarded),
        "style-check" => super::run_style_check_native(root, forwarded),
        "spec-lang-lint" => super::run_spec_lang_lint_native(root, forwarded),
        "spec-lang-format" => super::run_spec_lang_format_native(root, forwarded),
        "migrate-contract-step-imports-v1" => super::run_spec_lang_format_native(root, forwarded),
        "migrate-case-doc-metadata-v1" => super::run_migrate_case_doc_metadata_v1(root, forwarded),
        "migrate-library-docs-metadata-v1" => super::run_migrate_library_docs_metadata_v1(root, forwarded),
        "migrate-case-domain-prefix-v1" => super::run_migrate_case_domain_prefix_v1(root, forwarded),
        "normalize-check" => super::run_normalize_mode(root, forwarded, false),
        "normalize-fix" => super::run_normalize_mode(root, forwarded, true),
        "schema-registry-check" => super::run_job_for_command(root, "schema-registry-check", forwarded),
        "schema-registry-build" => super::run_job_for_command(root, "schema-registry-build", forwarded),
        "schema-docs-check" => super::run_schema_docs_native(root, forwarded, true),
        "schema-docs-build" => super::run_schema_docs_native(root, forwarded, false),
        "lint" => super::run_lint_native(root, forwarded),
        "typecheck" => super::run_typecheck_native(root, forwarded),
        "compilecheck" => super::run_compilecheck_native(root, forwarded),
        "conformance-purpose-json" => super::run_job_for_command(root, "conformance-purpose-json", forwarded),
        "conformance-purpose-md" => super::run_job_for_command(root, "conformance-purpose-md", forwarded),
        "spec-portability-json" => super::run_job_for_command(root, "spec-portability-json", forwarded),
        "spec-portability-md" => super::run_job_for_command(root, "spec-portability-md", forwarded),
        "spec-lang-adoption-json" => super::run_job_for_command(root, "spec-lang-adoption-json", forwarded),
        "spec-lang-adoption-md" => super::run_job_for_command(root, "spec-lang-adoption-md", forwarded),
        "runner-independence-json" => super::run_job_for_command(root, "runner-independence-json", forwarded),
        "runner-independence-md" => super::run_job_for_command(root, "runner-independence-md", forwarded),
        "python-dependency-json" => super::run_job_for_command(root, "python-dependency-json", forwarded),
        "python-dependency-md" => super::run_job_for_command(root, "python-dependency-md", forwarded),
        "docs-operability-json" => super::run_job_for_command(root, "docs-operability-json", forwarded),
        "docs-operability-md" => super::run_job_for_command(root, "docs-operability-md", forwarded),
        "contract-assertions-json" => super::run_job_for_command(root, "contract-assertions-json", forwarded),
        "contract-assertions-md" => super::run_job_for_command(root, "contract-assertions-md", forwarded),
        "objective-scorecard-json" => super::run_job_for_command(root, "objective-scorecard-json", forwarded),
        "objective-scorecard-md" => super::run_job_for_command(root, "objective-scorecard-md", forwarded),
        "spec-lang-stdlib-json" => super::run_job_for_command(root, "spec-lang-stdlib-json", forwarded),
        "spec-lang-stdlib-md" => super::run_job_for_command(root, "spec-lang-stdlib-md", forwarded),
        "ci-gate-summary" => super::run_ci_gate_summary_native(root, forwarded),
        "ci-cleanroom" => super::run_cmd(&super::script(root, "ci_cleanroom.sh"), &super::with_forwarded(vec![], forwarded), root),
        "perf-smoke" => super::run_job_for_command(root, "perf-smoke", forwarded),
        "docs-generate" => super::run_docs_generate_native(root, forwarded, false),
        "docs-generate-check" => super::run_docs_generate_native(root, forwarded, true),
        "docs-build" => super::run_job_for_command(root, "docs-build", forwarded),
        "docs-build-check" => super::run_job_for_command(root, "docs-build-check", forwarded),
        "docs-lint" => super::run_docs_lint_native(root, forwarded),
        "docs-graph" => super::run_job_for_command(root, "docs-graph", forwarded),
        "conformance-parity" => super::run_job_for_command(root, "conformance-parity", forwarded),
        "test-core" => super::run_tests_native(root, forwarded),
        "test-full" => super::run_tests_native(root, forwarded),
        "runner-certify" => super::run_runner_certify_native(root, forwarded),
        _ => {
            eprintln!("ERROR: unsupported runner adapter subcommand: {subcommand}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn dispatch_rejects_unknown_subcommand() {
        let code = dispatch(Path::new("."), "unknown-subcommand", &[]);
        assert_eq!(code, 2);
    }

    #[test]
    fn dispatch_spec_ref_requires_single_argument() {
        let code = dispatch(Path::new("."), "spec-ref", &[]);
        assert_eq!(code, 2);
    }
}
