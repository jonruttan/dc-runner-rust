use std::env;

use clap::error::ErrorKind;
use clap::Parser;

#[cfg(feature = "bundler")]
use crate::cli::args::BundlerSubcommand;
use crate::cli::args::{
    BundleSubcommand, CiSubcommand, Cli, CommandGroup, DocsSubcommand, EntrypointsSubcommand,
    GovernanceSubcommand, ProjectSubcommand, QualitySubcommand, ReportsSubcommand,
    SchemaSubcommand, SpecRefreshSourceOption, SpecSourceOption, SpecUseSourceOption,
    SpecsSubcommand,
};
use crate::cli::errors::CliError;

#[derive(Debug)]
pub struct ParsedEntry {
    pub subcommand: String,
    pub forwarded: Vec<String>,
}

fn looks_like_governance_group_subcommand(token: &str) -> bool {
    matches!(
        token,
        "run" | "heavy" | "broad" | "critical" | "--help" | "-h"
    )
}

fn apply_global_env(cli: &Cli) {
    if let Some(v) = &cli.spec_source {
        let value = match v {
            SpecSourceOption::Bundled => "bundled",
            SpecSourceOption::Workspace => "workspace",
            SpecSourceOption::Auto => "auto",
        };
        env::set_var("DC_RUNNER_SPEC_SOURCE", value);
    }
    if cli.verbose > 0 {
        env::set_var("SPEC_RUNNER_DEBUG", "1");
        env::set_var("SPEC_RUNNER_DEBUG_LEVEL", cli.verbose.to_string());
    }
    if let Some(v) = &cli.profile_level {
        env::set_var("SPEC_RUNNER_PROFILE_LEVEL", v);
    }
    if let Some(v) = &cli.profile_out {
        env::set_var("SPEC_RUNNER_PROFILE_OUT", v);
    }
    if let Some(v) = &cli.profile_summary_out {
        env::set_var("SPEC_RUNNER_PROFILE_SUMMARY_OUT", v);
    }
    if let Some(v) = &cli.profile_heartbeat_ms {
        env::set_var("SPEC_RUNNER_PROFILE_HEARTBEAT_MS", v);
    }
    if let Some(v) = &cli.profile_stall_threshold_ms {
        env::set_var("SPEC_RUNNER_PROFILE_STALL_THRESHOLD_MS", v);
    }
    if let Some(v) = &cli.liveness_level {
        env::set_var("SPEC_RUNNER_LIVENESS_LEVEL", v);
    }
    if let Some(v) = &cli.liveness_stall_ms {
        env::set_var("SPEC_RUNNER_LIVENESS_STALL_MS", v);
    }
    if let Some(v) = &cli.liveness_min_events {
        env::set_var("SPEC_RUNNER_LIVENESS_MIN_EVENTS", v);
    }
    if let Some(v) = &cli.liveness_hard_cap_ms {
        env::set_var("SPEC_RUNNER_LIVENESS_HARD_CAP_MS", v);
    }
    if let Some(v) = &cli.liveness_kill_grace_ms {
        env::set_var("SPEC_RUNNER_LIVENESS_KILL_GRACE_MS", v);
    }
}

fn map_passthrough(command: &str, args: Vec<String>) -> ParsedEntry {
    ParsedEntry {
        subcommand: command.to_string(),
        forwarded: args,
    }
}

fn from_cli(cli: Cli) -> ParsedEntry {
    match cli.command {
        CommandGroup::Specs(specs) => match specs.command {
            SpecsSubcommand::List { path, format } => {
                let mut forwarded = Vec::<String>::new();
                if let Some(p) = path {
                    forwarded.push("--path".to_string());
                    forwarded.push(p);
                }
                forwarded.push("--format".to_string());
                forwarded.push(match format {
                    crate::cli::args::OutputFormat::Text => "text".to_string(),
                    crate::cli::args::OutputFormat::Json => "json".to_string(),
                });
                map_passthrough("specs-list", forwarded)
            }
            SpecsSubcommand::Run { r#ref } => {
                map_passthrough("specs-run", vec!["--ref".to_string(), r#ref])
            }
            SpecsSubcommand::RunAll {
                root,
                fail_fast,
                continue_on_fail,
            } => {
                let mut forwarded = Vec::<String>::new();
                if let Some(p) = root {
                    forwarded.push("--root".to_string());
                    forwarded.push(p);
                }
                if fail_fast {
                    forwarded.push("--fail-fast".to_string());
                }
                if continue_on_fail {
                    forwarded.push("--continue-on-fail".to_string());
                }
                map_passthrough("specs-run-all", forwarded)
            }
            SpecsSubcommand::Check => map_passthrough("specs-check", vec![]),
            SpecsSubcommand::Refresh {
                source,
                version,
                bundle_id,
                force,
                check_only,
                skip_signature,
            } => {
                let mut forwarded = Vec::<String>::new();
                forwarded.push("--source".to_string());
                forwarded.push(match source {
                    SpecRefreshSourceOption::Remote => "remote".to_string(),
                    SpecRefreshSourceOption::Bundled => "bundled".to_string(),
                    SpecRefreshSourceOption::Workspace => "workspace".to_string(),
                });
                forwarded.push("--version".to_string());
                forwarded.push(version);
                if let Some(bundle_id) = bundle_id {
                    forwarded.push("--bundle-id".to_string());
                    forwarded.push(bundle_id);
                }
                if force {
                    forwarded.push("--force".to_string());
                }
                if check_only {
                    forwarded.push("--check-only".to_string());
                }
                if skip_signature {
                    forwarded.push("--skip-signature".to_string());
                }
                map_passthrough("specs-refresh", forwarded)
            }
            SpecsSubcommand::Status => map_passthrough("specs-status", vec![]),
            SpecsSubcommand::Versions => map_passthrough("specs-versions", vec![]),
            SpecsSubcommand::Use { target, source } => {
                let mut forwarded = Vec::<String>::new();
                forwarded.push(target);
                forwarded.push("--source".to_string());
                forwarded.push(match source {
                    SpecUseSourceOption::Version => "version".to_string(),
                    SpecUseSourceOption::Bundled => "bundled".to_string(),
                    SpecUseSourceOption::Workspace => "workspace".to_string(),
                });
                map_passthrough("specs-use", forwarded)
            }
            SpecsSubcommand::Rollback { to } => {
                let mut forwarded = Vec::<String>::new();
                if let Some(version) = to {
                    forwarded.push("--to".to_string());
                    forwarded.push(version);
                }
                map_passthrough("specs-rollback", forwarded)
            }
            SpecsSubcommand::Verify { source } => {
                map_passthrough("specs-verify", vec!["--source".to_string(), source])
            }
            SpecsSubcommand::Clean { keep, dry_run, yes } => {
                let mut forwarded = vec!["--keep".to_string(), keep.to_string()];
                if dry_run {
                    forwarded.push("--dry-run".to_string());
                }
                if yes {
                    forwarded.push("--yes".to_string());
                }
                map_passthrough("specs-clean", forwarded)
            }
            SpecsSubcommand::Info { selected_version } => {
                let mut forwarded = Vec::<String>::new();
                if let Some(v) = selected_version {
                    forwarded.push("--version".to_string());
                    forwarded.push(v);
                }
                map_passthrough("specs-info", forwarded)
            }
            SpecsSubcommand::Prune { expired } => {
                if expired {
                    map_passthrough("specs-prune", vec!["--expired".to_string()])
                } else {
                    map_passthrough("specs-prune", vec![])
                }
            }
        },
        CommandGroup::Entrypoints(entrypoints) => match entrypoints.command {
            EntrypointsSubcommand::List { format } => {
                let mut forwarded = Vec::<String>::new();
                forwarded.push("--format".to_string());
                forwarded.push(match format {
                    crate::cli::args::OutputFormat::Text => "text".to_string(),
                    crate::cli::args::OutputFormat::Json => "json".to_string(),
                });
                map_passthrough("entrypoints-list", forwarded)
            }
            EntrypointsSubcommand::Run { command_id } => {
                map_passthrough("entrypoints-run", vec![command_id])
            }
        },
        CommandGroup::Quality(q) => match q.command {
            QualitySubcommand::Lint { mode } => map_passthrough(
                "quality-lint",
                vec!["--input".to_string(), format!("mode={}", mode.as_str())],
            ),
            QualitySubcommand::Typecheck => map_passthrough("typecheck", vec![]),
            QualitySubcommand::Compilecheck => map_passthrough("compilecheck", vec![]),
            QualitySubcommand::StyleCheck => map_passthrough("style-check", vec![]),
            QualitySubcommand::TestCore => map_passthrough("test-core", vec![]),
            QualitySubcommand::TestFull => map_passthrough("test-full", vec![]),
        },
        CommandGroup::Governance(g) => match g.command {
            GovernanceSubcommand::Run => map_passthrough("governance", vec![]),
            GovernanceSubcommand::Heavy => map_passthrough("governance-heavy", vec![]),
            GovernanceSubcommand::Broad => map_passthrough("governance-broad-native", vec![]),
            GovernanceSubcommand::Critical => map_passthrough("critical-gate", vec![]),
        },
        CommandGroup::Docs(d) => match d.command {
            DocsSubcommand::Generate => map_passthrough("docs-generate", vec![]),
            DocsSubcommand::GenerateCheck => map_passthrough("docs-generate-check", vec![]),
            DocsSubcommand::Build => map_passthrough("docs-build", vec![]),
            DocsSubcommand::BuildCheck => map_passthrough("docs-build-check", vec![]),
            DocsSubcommand::Lint => map_passthrough("docs-lint", vec![]),
            DocsSubcommand::Graph => map_passthrough("docs-graph", vec![]),
        },
        CommandGroup::Schema(s) => match s.command {
            SchemaSubcommand::Check => map_passthrough("schema-check", vec![]),
            SchemaSubcommand::Lint => map_passthrough("schema-lint", vec![]),
            SchemaSubcommand::Format => map_passthrough("schema-format", vec![]),
        },
        #[cfg(feature = "bundler")]
        CommandGroup::Bundler(b) => match b.command {
            BundlerSubcommand::Resolve => map_passthrough("bundler-resolve", vec![]),
            BundlerSubcommand::Package => map_passthrough("bundler-package", vec![]),
            BundlerSubcommand::Check => map_passthrough("bundler-check", vec![]),
        },
        CommandGroup::Bundle(b) => match b.command {
            BundleSubcommand::List => map_passthrough("bundle-list", vec![]),
            BundleSubcommand::Info {
                bundle_id,
                bundle_version,
            } => {
                let mut forwarded = vec!["--bundle-id".to_string(), bundle_id];
                if let Some(v) = bundle_version {
                    forwarded.push("--bundle-version".to_string());
                    forwarded.push(v);
                }
                map_passthrough("bundle-info", forwarded)
            }
            BundleSubcommand::Inspect {
                bundle_id,
                bundle_version,
            } => {
                let mut forwarded = vec!["--bundle-id".to_string(), bundle_id];
                if let Some(v) = bundle_version {
                    forwarded.push("--bundle-version".to_string());
                    forwarded.push(v);
                }
                map_passthrough("bundle-info", forwarded)
            }
            BundleSubcommand::Install {
                project_lock,
                out,
                bundle_id,
                bundle_version,
                install_dir,
            } => {
                let mut forwarded = Vec::<String>::new();
                if let Some(v) = project_lock {
                    forwarded.push("--project-lock".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = out {
                    forwarded.push("--out".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = bundle_id {
                    forwarded.push("--bundle-id".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = bundle_version {
                    forwarded.push("--bundle-version".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = install_dir {
                    forwarded.push("--install-dir".to_string());
                    forwarded.push(v);
                }
                map_passthrough("bundle-install", forwarded)
            }
            BundleSubcommand::InstallCheck { project_lock, out } => {
                let mut forwarded = vec!["--project-lock".to_string(), project_lock];
                if let Some(v) = out {
                    forwarded.push("--out".to_string());
                    forwarded.push(v);
                }
                map_passthrough("bundle-install-check", forwarded)
            }
            BundleSubcommand::Bootstrap { lock, out } => {
                let mut forwarded = vec!["--lock".to_string(), lock];
                if let Some(v) = out {
                    forwarded.push("--out".to_string());
                    forwarded.push(v);
                }
                map_passthrough("bundle-bootstrap", forwarded)
            }
            BundleSubcommand::BootstrapCheck { lock, out } => {
                let mut forwarded = vec!["--lock".to_string(), lock];
                if let Some(v) = out {
                    forwarded.push("--out".to_string());
                    forwarded.push(v);
                }
                map_passthrough("bundle-bootstrap-check", forwarded)
            }
            BundleSubcommand::Outdated {
                project_lock,
                format,
            } => {
                let mut forwarded = vec!["--project-lock".to_string(), project_lock];
                if let Some(v) = format {
                    forwarded.push("--format".to_string());
                    forwarded.push(v);
                }
                map_passthrough("bundle-outdated", forwarded)
            }
            BundleSubcommand::Upgrade {
                project_lock,
                dry_run,
            } => {
                let mut forwarded = vec!["--project-lock".to_string(), project_lock];
                if dry_run {
                    forwarded.push("--dry-run".to_string());
                }
                map_passthrough("bundle-upgrade", forwarded)
            }
            BundleSubcommand::Run {
                bundle_id,
                bundle_version,
                entrypoint,
                args,
            } => {
                let mut forwarded = vec![
                    "--bundle-id".to_string(),
                    bundle_id,
                    "--bundle-version".to_string(),
                    bundle_version,
                    "--entrypoint".to_string(),
                    entrypoint,
                ];
                for arg in args {
                    forwarded.push("--arg".to_string());
                    forwarded.push(arg);
                }
                map_passthrough("bundle-run", forwarded)
            }
            BundleSubcommand::Scaffold {
                project_root,
                bundle_id,
                bundle_version,
                bundle_url,
                sha256,
                allow_external,
                runner,
                vars,
                overwrite,
            } => {
                let mut forwarded = vec!["--project-root".to_string(), project_root];
                if let Some(v) = bundle_id {
                    forwarded.push("--bundle-id".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = bundle_version {
                    forwarded.push("--bundle-version".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = bundle_url {
                    forwarded.push("--bundle-url".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = sha256 {
                    forwarded.push("--sha256".to_string());
                    forwarded.push(v);
                }
                if allow_external {
                    forwarded.push("--allow-external".to_string());
                }
                if let Some(v) = runner {
                    forwarded.push("--runner".to_string());
                    forwarded.push(v);
                }
                for entry in vars {
                    forwarded.push("--var".to_string());
                    forwarded.push(entry);
                }
                if overwrite {
                    forwarded.push("--overwrite".to_string());
                }
                map_passthrough("bundle-scaffold", forwarded)
            }
        },
        CommandGroup::Reports(r) => match r.command {
            ReportsSubcommand::ConformancePurposeJson => {
                map_passthrough("conformance-purpose-json", vec![])
            }
            ReportsSubcommand::ConformancePurposeMd => {
                map_passthrough("conformance-purpose-md", vec![])
            }
            ReportsSubcommand::RunnerIndependenceJson => {
                map_passthrough("runner-independence-json", vec![])
            }
            ReportsSubcommand::RunnerIndependenceMd => {
                map_passthrough("runner-independence-md", vec![])
            }
            ReportsSubcommand::PythonDependencyJson => {
                map_passthrough("python-dependency-json", vec![])
            }
            ReportsSubcommand::PythonDependencyMd => {
                map_passthrough("python-dependency-md", vec![])
            }
            ReportsSubcommand::SpecPortabilityJson => {
                map_passthrough("spec-portability-json", vec![])
            }
            ReportsSubcommand::SpecPortabilityMd => map_passthrough("spec-portability-md", vec![]),
            ReportsSubcommand::SpecLangAdoptionJson => {
                map_passthrough("spec-lang-adoption-json", vec![])
            }
            ReportsSubcommand::SpecLangAdoptionMd => {
                map_passthrough("spec-lang-adoption-md", vec![])
            }
            ReportsSubcommand::DocsOperabilityJson => {
                map_passthrough("docs-operability-json", vec![])
            }
            ReportsSubcommand::DocsOperabilityMd => map_passthrough("docs-operability-md", vec![]),
            ReportsSubcommand::ContractAssertionsJson => {
                map_passthrough("contract-assertions-json", vec![])
            }
            ReportsSubcommand::ContractAssertionsMd => {
                map_passthrough("contract-assertions-md", vec![])
            }
            ReportsSubcommand::ObjectiveScorecardJson => {
                map_passthrough("objective-scorecard-json", vec![])
            }
            ReportsSubcommand::ObjectiveScorecardMd => {
                map_passthrough("objective-scorecard-md", vec![])
            }
            ReportsSubcommand::SpecLangStdlibJson => {
                map_passthrough("spec-lang-stdlib-json", vec![])
            }
            ReportsSubcommand::SpecLangStdlibMd => map_passthrough("spec-lang-stdlib-md", vec![]),
        },
        CommandGroup::Ci(c) => match c.command {
            CiSubcommand::GateSummary => map_passthrough("ci-gate-summary", vec![]),
            CiSubcommand::Cleanroom => map_passthrough("ci-cleanroom", vec![]),
            CiSubcommand::ConformanceParity => map_passthrough("conformance-parity", vec![]),
            CiSubcommand::RunnerCertify => map_passthrough("runner-certify", vec![]),
        },
        CommandGroup::Project(p) => match p.command {
            ProjectSubcommand::Scaffold {
                project_root,
                bundle_id,
                bundle_version,
                bundle_url,
                sha256,
                allow_external,
                runner,
                vars,
                overwrite,
            } => {
                let mut forwarded = vec!["--project-root".to_string(), project_root];
                if let Some(v) = bundle_id {
                    forwarded.push("--bundle-id".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = bundle_version {
                    forwarded.push("--bundle-version".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = bundle_url {
                    forwarded.push("--bundle-url".to_string());
                    forwarded.push(v);
                }
                if let Some(v) = sha256 {
                    forwarded.push("--sha256".to_string());
                    forwarded.push(v);
                }
                if allow_external {
                    forwarded.push("--allow-external".to_string());
                }
                if let Some(v) = runner {
                    forwarded.push("--runner".to_string());
                    forwarded.push(v);
                }
                for v in vars {
                    forwarded.push("--var".to_string());
                    forwarded.push(v);
                }
                if overwrite {
                    forwarded.push("--overwrite".to_string());
                }
                map_passthrough("project-scaffold", forwarded)
            }
        },
        CommandGroup::StyleCheck(x) => map_passthrough("style-check", x.args),
        CommandGroup::Lint(x) => map_passthrough(
            "lint",
            vec!["--input".to_string(), format!("mode={}", x.mode.as_str())],
        ),
        CommandGroup::Typecheck(x) => map_passthrough("typecheck", x.args),
        CommandGroup::Compilecheck(x) => map_passthrough("compilecheck", x.args),
        CommandGroup::ConformancePurposeJson(x) => {
            map_passthrough("conformance-purpose-json", x.args)
        }
        CommandGroup::ConformancePurposeMd(x) => map_passthrough("conformance-purpose-md", x.args),
        CommandGroup::RunnerIndependenceJson(x) => {
            map_passthrough("runner-independence-json", x.args)
        }
        CommandGroup::RunnerIndependenceMd(x) => map_passthrough("runner-independence-md", x.args),
        CommandGroup::PythonDependencyJson(x) => map_passthrough("python-dependency-json", x.args),
        CommandGroup::PythonDependencyMd(x) => map_passthrough("python-dependency-md", x.args),
        CommandGroup::CiGateSummary(x) => map_passthrough("ci-gate-summary", x.args),
        CommandGroup::DocsGenerate(x) => map_passthrough("docs-generate", x.args),
        CommandGroup::DocsGenerateCheck(x) => map_passthrough("docs-generate-check", x.args),
        CommandGroup::ConformanceParity(x) => map_passthrough("conformance-parity", x.args),
        CommandGroup::RunnerCertify(x) => map_passthrough("runner-certify", x.args),
        CommandGroup::TestCore(x) => map_passthrough("test-core", x.args),
        CommandGroup::TestFull(x) => map_passthrough("test-full", x.args),
        CommandGroup::JobRun(x) => map_passthrough("job-run", x.args),
        CommandGroup::SpecEval(x) => map_passthrough("spec-eval", x.args),
        CommandGroup::CriticalGate(x) => map_passthrough("critical-gate", x.args),
        CommandGroup::GovernanceBroadNative(x) => {
            map_passthrough("governance-broad-native", x.args)
        }
        CommandGroup::SpecRef(x) => map_passthrough("spec-ref", x.args),
        CommandGroup::ValidateReport(x) => map_passthrough("validate-report", x.args),
        CommandGroup::GovernanceHeavy(x) => map_passthrough("governance-heavy", x.args),
        CommandGroup::SpecLangLint(x) => map_passthrough("spec-lang-lint", x.args),
        CommandGroup::SpecLangFormat(x) => map_passthrough("spec-lang-format", x.args),
        CommandGroup::MigrateContractStepImportsV1(x) => {
            map_passthrough("migrate-contract-step-imports-v1", x.args)
        }
        CommandGroup::MigrateCaseDocMetadataV1(x) => {
            map_passthrough("migrate-case-doc-metadata-v1", x.args)
        }
        CommandGroup::MigrateLibraryDocsMetadataV1(x) => {
            map_passthrough("migrate-library-docs-metadata-v1", x.args)
        }
        CommandGroup::MigrateCaseDomainPrefixV1(x) => {
            map_passthrough("migrate-case-domain-prefix-v1", x.args)
        }
        CommandGroup::NormalizeCheck(x) => map_passthrough("normalize-check", x.args),
        CommandGroup::NormalizeFix(x) => map_passthrough("normalize-fix", x.args),
        CommandGroup::SpecPortabilityJson(x) => map_passthrough("spec-portability-json", x.args),
        CommandGroup::SpecPortabilityMd(x) => map_passthrough("spec-portability-md", x.args),
        CommandGroup::SpecLangAdoptionJson(x) => map_passthrough("spec-lang-adoption-json", x.args),
        CommandGroup::SpecLangAdoptionMd(x) => map_passthrough("spec-lang-adoption-md", x.args),
        CommandGroup::DocsOperabilityJson(x) => map_passthrough("docs-operability-json", x.args),
        CommandGroup::DocsOperabilityMd(x) => map_passthrough("docs-operability-md", x.args),
        CommandGroup::ContractAssertionsJson(x) => {
            map_passthrough("contract-assertions-json", x.args)
        }
        CommandGroup::ContractAssertionsMd(x) => map_passthrough("contract-assertions-md", x.args),
        CommandGroup::ObjectiveScorecardJson(x) => {
            map_passthrough("objective-scorecard-json", x.args)
        }
        CommandGroup::ObjectiveScorecardMd(x) => map_passthrough("objective-scorecard-md", x.args),
        CommandGroup::SpecLangStdlibJson(x) => map_passthrough("spec-lang-stdlib-json", x.args),
        CommandGroup::SpecLangStdlibMd(x) => map_passthrough("spec-lang-stdlib-md", x.args),
        CommandGroup::CiCleanroom(x) => map_passthrough("ci-cleanroom", x.args),
        CommandGroup::PerfSmoke(x) => map_passthrough("perf-smoke", x.args),
        CommandGroup::DocsBuild(x) => map_passthrough("docs-build", x.args),
        CommandGroup::DocsBuildCheck(x) => map_passthrough("docs-build-check", x.args),
        CommandGroup::DocsLint(x) => map_passthrough("docs-lint", x.args),
        CommandGroup::DocsGraph(x) => map_passthrough("docs-graph", x.args),
        CommandGroup::ServicePluginCheck(x) => map_passthrough("service-plugin-check", x.args),
        CommandGroup::HelpAdvanced => map_passthrough("help-advanced", vec![]),
    }
}

pub fn parse_entry(args: &[String]) -> Result<ParsedEntry, i32> {
    if args.len() <= 1 {
        println!("dc-runner quick start:");
        println!("  dc-runner specs run-all");
        println!("  dc-runner specs list");
        println!("  dc-runner schema check");
        println!("  dc-runner docs generate-check");
        println!("  dc-runner --help");
        return Err(0);
    }
    if args.iter().any(|arg| arg == "--help-advanced")
        || (args.len() > 1 && args[1] == "help-advanced")
    {
        println!("{}", crate::cli::help::ADVANCED_HELP);
        return Err(0);
    }

    // Preserve required compatibility command semantics for `governance` while also
    // exposing a grouped `governance` UX command surface.
    if args.len() > 1 && args[1] == "governance" {
        let next = args.get(2).map(|s| s.as_str()).unwrap_or("");
        if args.len() == 2
            || (next.starts_with('-') && next != "--help" && next != "-h")
            || !looks_like_governance_group_subcommand(next)
        {
            return Ok(map_passthrough("governance", args[2..].to_vec()));
        }
    }

    let cli = match Cli::try_parse_from(args) {
        Ok(v) => v,
        Err(err) => {
            let code = match err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
                _ => CliError::Usage("invalid arguments".to_string()).exit_code(),
            };
            let _ = err.print();
            if code == 2 {
                eprintln!("Try `dc-runner --help`.");
            }
            return Err(code);
        }
    };

    apply_global_env(&cli);
    Ok(from_cli(cli))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parse_entry_reads_subcommand_and_forwarded_args() {
        let args = argv(&["dc-runner", "job-run", "--ref", "#CASE"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "job-run");
        assert_eq!(
            parsed.forwarded,
            vec!["--ref".to_string(), "#CASE".to_string()]
        );
    }

    #[test]
    fn parse_entry_supports_specs_run() {
        let args = argv(&[
            "dc-runner",
            "specs",
            "run",
            "--ref",
            "/specs/example.spec.md#CASE-1",
        ]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "specs-run");
    }

    #[test]
    fn parse_entry_supports_specs_refresh_with_bundle_id() {
        let args = argv(&[
            "dc-runner",
            "specs",
            "refresh",
            "--source",
            "remote",
            "--version",
            "latest",
            "--bundle-id",
            "data-contracts-lang-project-scaffold",
            "--force",
        ]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "specs-refresh");
        assert!(parsed.forwarded.contains(&"--bundle-id".to_string()));
        assert!(parsed
            .forwarded
            .contains(&"data-contracts-lang-project-scaffold".to_string()));
    }

    #[test]
    fn parse_entry_supports_bundle_list() {
        let args = argv(&["dc-runner", "bundle", "list"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "bundle-list");
        assert!(parsed.forwarded.is_empty());
    }

    #[test]
    fn parse_entry_supports_bundle_info() {
        let args = argv(&[
            "dc-runner",
            "bundle",
            "info",
            "--bundle-id",
            "core",
            "--bundle-version",
            "1.0.0",
        ]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "bundle-info");
        assert!(parsed.forwarded.contains(&"--bundle-id".to_string()));
        assert!(parsed.forwarded.contains(&"--bundle-version".to_string()));
    }

    #[test]
    fn parse_entry_supports_bundle_install_lock_driven() {
        let args = argv(&[
            "dc-runner",
            "bundle",
            "install",
            "--project-lock",
            "bundles.lock.yaml",
            "--out",
            "/tmp/bundles",
        ]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "bundle-install");
        assert!(parsed.forwarded.contains(&"--project-lock".to_string()));
        assert!(parsed.forwarded.contains(&"--out".to_string()));
    }

    #[test]
    fn parse_entry_supports_quality_lint_strict_default() {
        let args = argv(&["dc-runner", "quality", "lint"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "quality-lint");
        assert_eq!(
            parsed.forwarded,
            vec!["--input".to_string(), "mode=strict".to_string()]
        );
    }

    #[test]
    fn parse_entry_supports_quality_lint_pedantic() {
        let args = argv(&["dc-runner", "quality", "lint", "--mode", "pedantic"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "quality-lint");
        assert_eq!(
            parsed.forwarded,
            vec!["--input".to_string(), "mode=pedantic".to_string()]
        );
    }

    #[test]
    fn parse_entry_supports_hidden_lint_alias() {
        let args = argv(&["dc-runner", "lint", "--mode", "pedantic"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "lint");
        assert_eq!(
            parsed.forwarded,
            vec!["--input".to_string(), "mode=pedantic".to_string()]
        );
    }

    #[test]
    fn parse_entry_rejects_bundle_info_missing_required_bundle_id() {
        let args = argv(&["dc-runner", "bundle", "info"]);
        let code = parse_entry(&args).expect_err("should fail");
        assert_eq!(code, 2);
    }

    #[test]
    fn parse_entry_rejects_bundle_install_missing_required_version() {
        let args = argv(&["dc-runner", "bundle", "install", "--bundle-id", "core"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "bundle-install");
    }

    #[test]
    fn parse_entry_rejects_unknown_subcommand() {
        let args = argv(&["dc-runner", "does-not-exist"]);
        let code = parse_entry(&args).expect_err("should fail");
        assert_eq!(code, 2);
    }

    #[test]
    fn parse_entry_no_args_returns_quickstart_exit_zero() {
        let args = argv(&["dc-runner"]);
        let code = parse_entry(&args).expect_err("quickstart should exit");
        assert_eq!(code, 0);
    }

    #[test]
    fn parse_entry_supports_governance_group_name() {
        let args = argv(&["dc-runner", "governance", "run"]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "governance");
    }

    #[test]
    fn parse_entry_supports_project_scaffold() {
        let args = argv(&[
            "dc-runner",
            "project",
            "scaffold",
            "--project-root",
            "/tmp/demo",
            "--bundle-id",
            "core",
            "--bundle-version",
            "1.0.0",
        ]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "project-scaffold");
        assert!(parsed.forwarded.contains(&"--bundle-id".to_string()));
    }

    #[test]
    fn parse_entry_supports_bundle_scaffold() {
        let args = argv(&[
            "dc-runner",
            "bundle",
            "scaffold",
            "--project-root",
            "/tmp/demo",
            "--bundle-id",
            "core",
            "--bundle-version",
            "1.0.0",
        ]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "bundle-scaffold");
        assert!(parsed.forwarded.contains(&"--bundle-id".to_string()));
    }

    #[test]
    fn parse_entry_supports_bundle_run() {
        let args = argv(&[
            "dc-runner",
            "bundle",
            "run",
            "--bundle-id",
            "data-contracts-library-review-workflow",
            "--bundle-version",
            "1.0.0",
            "--entrypoint",
            "run",
            "--arg",
            "foo",
        ]);
        let parsed = parse_entry(&args).expect("parse");
        assert_eq!(parsed.subcommand, "bundle-run");
        assert!(parsed.forwarded.contains(&"--entrypoint".to_string()));
        assert!(parsed.forwarded.contains(&"--arg".to_string()));
    }
}
