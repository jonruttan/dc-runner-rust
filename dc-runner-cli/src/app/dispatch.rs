use std::path::Path;

pub fn dispatch(root: &Path, subcommand: &str, forwarded: &[String]) -> i32 {
    match subcommand {
        "entrypoints-list" => super::run_entrypoints_list_native(root, forwarded),
        "entrypoints-run" => super::run_entrypoints_run_native(root, forwarded),
        "spec-eval" => super::run_spec_eval_native(root, forwarded),
        "job-run" => super::run_job_run_native(root, forwarded),
        "critical-gate" => super::run_registered_entry_command(root, "critical-gate", forwarded),
        "governance-broad-native" => {
            super::run_registered_entry_command(root, "governance-broad-native", forwarded)
        }
        "spec-ref" => {
            if forwarded.len() != 1 {
                eprintln!("usage: spec-ref <subcommand>");
                2
            } else {
                super::run_spec_ref_print(&forwarded[0])
            }
        }
        "bundle-list" => super::run_registered_entry_command(root, "bundle-list", forwarded),
        "bundle-inspect" => super::run_registered_entry_command(root, "bundle-inspect", forwarded),
        "bundle-install" => super::run_registered_entry_command(root, "bundle-install", forwarded),
        "validate-report" => super::run_validate_report_native(root, forwarded),
        "governance" => super::run_registered_entry_command(root, "governance", forwarded),
        "governance-heavy" => super::run_governance_heavy_native(root, forwarded),
        "style-check" => super::run_style_check_native(root, forwarded),
        "specs-list" => super::run_specs_list_native(root, forwarded),
        "specs-run" => super::run_specs_run_native(root, forwarded),
        "specs-run-all" => super::run_specs_run_all_native(root, forwarded),
        "specs-check" => super::run_specs_check_native(root, forwarded),
        "specs-refresh" => super::run_specs_refresh_native(root, forwarded),
        "specs-status" => super::run_specs_status_native(root, forwarded),
        "specs-versions" => super::run_specs_versions_native(root, forwarded),
        "specs-use" => super::run_specs_use_native(root, forwarded),
        "specs-rollback" => super::run_specs_rollback_native(root, forwarded),
        "specs-verify" => super::run_specs_verify_native(root, forwarded),
        "specs-clean" => super::run_specs_clean_native(root, forwarded),
        "specs-info" => super::run_specs_info_native(root, forwarded),
        "specs-prune" => super::run_specs_prune_native(root, forwarded),
        "help-advanced" => super::run_help_advanced_native(forwarded),
        "spec-lang-lint" => super::run_spec_lang_lint_native(root, forwarded),
        "spec-lang-format" => super::run_spec_lang_format_native(root, forwarded),
        "migrate-contract-step-imports-v1" => super::run_spec_lang_format_native(root, forwarded),
        "migrate-case-doc-metadata-v1" => super::run_migrate_case_doc_metadata_v1(root, forwarded),
        "migrate-library-docs-metadata-v1" => {
            super::run_migrate_library_docs_metadata_v1(root, forwarded)
        }
        "migrate-case-domain-prefix-v1" => {
            super::run_migrate_case_domain_prefix_v1(root, forwarded)
        }
        "normalize-check" => super::run_normalize_mode(root, forwarded, false),
        "normalize-fix" => super::run_normalize_mode(root, forwarded, true),
        "schema-check" => super::run_registered_entry_command(root, "schema-check", forwarded),
        "schema-lint" => super::run_registered_entry_command(root, "schema-lint", forwarded),
        "schema-format" => super::run_registered_entry_command(root, "schema-format", forwarded),
        #[cfg(feature = "bundler")]
        "bundler-resolve" => {
            super::run_registered_entry_command(root, "bundler-resolve", forwarded)
        }
        #[cfg(feature = "bundler")]
        "bundler-package" => {
            super::run_registered_entry_command(root, "bundler-package", forwarded)
        }
        #[cfg(feature = "bundler")]
        "bundler-check" => super::run_registered_entry_command(root, "bundler-check", forwarded),
        "lint" => super::run_lint_native(root, forwarded),
        "typecheck" => super::run_typecheck_native(root, forwarded),
        "compilecheck" => super::run_compilecheck_native(root, forwarded),
        "conformance-purpose-json" => {
            super::run_job_for_command(root, "conformance-purpose-json", forwarded)
        }
        "conformance-purpose-md" => {
            super::run_job_for_command(root, "conformance-purpose-md", forwarded)
        }
        "spec-portability-json" => {
            super::run_job_for_command(root, "spec-portability-json", forwarded)
        }
        "spec-portability-md" => super::run_job_for_command(root, "spec-portability-md", forwarded),
        "spec-lang-adoption-json" => {
            super::run_job_for_command(root, "spec-lang-adoption-json", forwarded)
        }
        "spec-lang-adoption-md" => {
            super::run_job_for_command(root, "spec-lang-adoption-md", forwarded)
        }
        "runner-independence-json" => {
            super::run_job_for_command(root, "runner-independence-json", forwarded)
        }
        "runner-independence-md" => {
            super::run_job_for_command(root, "runner-independence-md", forwarded)
        }
        "python-dependency-json" => {
            super::run_job_for_command(root, "python-dependency-json", forwarded)
        }
        "python-dependency-md" => {
            super::run_job_for_command(root, "python-dependency-md", forwarded)
        }
        "docs-operability-json" => {
            super::run_job_for_command(root, "docs-operability-json", forwarded)
        }
        "docs-operability-md" => super::run_job_for_command(root, "docs-operability-md", forwarded),
        "contract-assertions-json" => {
            super::run_job_for_command(root, "contract-assertions-json", forwarded)
        }
        "contract-assertions-md" => {
            super::run_job_for_command(root, "contract-assertions-md", forwarded)
        }
        "objective-scorecard-json" => {
            super::run_job_for_command(root, "objective-scorecard-json", forwarded)
        }
        "objective-scorecard-md" => {
            super::run_job_for_command(root, "objective-scorecard-md", forwarded)
        }
        "spec-lang-stdlib-json" => {
            super::run_job_for_command(root, "spec-lang-stdlib-json", forwarded)
        }
        "spec-lang-stdlib-md" => super::run_job_for_command(root, "spec-lang-stdlib-md", forwarded),
        "ci-gate-summary" => super::run_ci_gate_summary_native(root, forwarded),
        "ci-cleanroom" => super::run_cmd(
            &super::script(root, "ci_cleanroom.sh"),
            &super::with_forwarded(vec![], forwarded),
            root,
        ),
        "perf-smoke" => super::run_job_for_command(root, "perf-smoke", forwarded),
        "docs-generate" => super::run_registered_entry_command(root, "docs-generate", forwarded),
        "docs-generate-check" => {
            super::run_registered_entry_command(root, "docs-generate-check", forwarded)
        }
        "docs-build" => super::run_registered_entry_command(root, "docs-build", forwarded),
        "docs-build-check" => {
            super::run_registered_entry_command(root, "docs-build-check", forwarded)
        }
        "docs-lint" => super::run_registered_entry_command(root, "docs-lint", forwarded),
        "docs-graph" => super::run_registered_entry_command(root, "docs-graph", forwarded),
        "conformance-parity" => super::run_job_for_command(root, "conformance-parity", forwarded),
        "test-core" => super::run_tests_native(root, forwarded),
        "test-full" => super::run_tests_native(root, forwarded),
        "runner-certify" => super::run_runner_certify_native(root, forwarded),
        "service-plugin-check" => super::run_service_plugin_check_native(root, forwarded),
        "project-scaffold" => super::run_project_scaffold_native(root, forwarded),
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
    fn dispatch_targets_bundle_list_via_dispatch() {
        let code = dispatch(Path::new("."), "bundle-list", &[]);
        assert_eq!(code, 2);
    }

    #[test]
    fn dispatch_targets_bundle_inspect_via_dispatch_rejects_forwarded_args() {
        let forwarded = vec![
            "--bundle-id".to_string(),
            "core".to_string(),
            "--bundle-version".to_string(),
            "1.0.0".to_string(),
        ];
        let code = dispatch(Path::new("."), "bundle-inspect", &forwarded);
        assert_eq!(code, 2);
    }

    #[test]
    fn dispatch_targets_bundle_install_via_dispatch_rejects_forwarded_args() {
        let forwarded = vec![
            "--bundle-id".to_string(),
            "core".to_string(),
            "--bundle-version".to_string(),
            "1.0.0".to_string(),
        ];
        let code = dispatch(Path::new("."), "bundle-install", &forwarded);
        assert_eq!(code, 2);
    }

    #[test]
    fn dispatch_spec_ref_requires_single_argument() {
        let code = dispatch(Path::new("."), "spec-ref", &[]);
        assert_eq!(code, 2);
    }
}
