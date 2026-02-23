use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "dc_runner_cli",
    bin_name = "spec_runner_cli",
    version,
    about = "Rust Data Contracts runner",
    long_about = "Rust-first Data Contracts runner with compatibility commands and user-friendly specs workflows.",
    after_help = crate::cli::help::GLOBAL_EXAMPLES,
    arg_required_else_help = true,
    propagate_version = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub verbose: u8,

    #[arg(long = "profile-level", global = true, hide = true)]
    pub profile_level: Option<String>,
    #[arg(long = "profile-out", global = true, hide = true)]
    pub profile_out: Option<String>,
    #[arg(long = "profile-summary-out", global = true, hide = true)]
    pub profile_summary_out: Option<String>,
    #[arg(long = "profile-heartbeat-ms", global = true, hide = true)]
    pub profile_heartbeat_ms: Option<String>,
    #[arg(long = "profile-stall-threshold-ms", global = true, hide = true)]
    pub profile_stall_threshold_ms: Option<String>,
    #[arg(long = "liveness-level", global = true, hide = true)]
    pub liveness_level: Option<String>,
    #[arg(long = "liveness-stall-ms", global = true, hide = true)]
    pub liveness_stall_ms: Option<String>,
    #[arg(long = "liveness-min-events", global = true, hide = true)]
    pub liveness_min_events: Option<String>,
    #[arg(long = "liveness-hard-cap-ms", global = true, hide = true)]
    pub liveness_hard_cap_ms: Option<String>,
    #[arg(long = "liveness-kill-grace-ms", global = true, hide = true)]
    pub liveness_kill_grace_ms: Option<String>,

    #[command(subcommand)]
    pub command: CommandGroup,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CommandGroup {
    /// Specs workflows
    Specs(SpecsCommand),
    /// Quality checks
    Quality(QualityCommand),
    /// Governance checks
    #[command(name = "governance", visible_alias = "gov")]
    Governance(GovernanceCommand),
    /// Docs workflows
    Docs(DocsCommand),
    /// Reporting workflows
    Reports(ReportsCommand),
    /// CI workflows
    Ci(CiCommand),
    /// Project workflows
    Project(ProjectCommand),

    #[command(name = "style-check", hide = true)]
    StyleCheck(PassthroughArgs),
    #[command(name = "lint", hide = true)]
    Lint(PassthroughArgs),
    #[command(name = "typecheck", hide = true)]
    Typecheck(PassthroughArgs),
    #[command(name = "compilecheck", hide = true)]
    Compilecheck(PassthroughArgs),
    #[command(name = "conformance-purpose-json", hide = true)]
    ConformancePurposeJson(PassthroughArgs),
    #[command(name = "conformance-purpose-md", hide = true)]
    ConformancePurposeMd(PassthroughArgs),
    #[command(name = "runner-independence-json", hide = true)]
    RunnerIndependenceJson(PassthroughArgs),
    #[command(name = "runner-independence-md", hide = true)]
    RunnerIndependenceMd(PassthroughArgs),
    #[command(name = "python-dependency-json", hide = true)]
    PythonDependencyJson(PassthroughArgs),
    #[command(name = "python-dependency-md", hide = true)]
    PythonDependencyMd(PassthroughArgs),
    #[command(name = "ci-gate-summary", hide = true)]
    CiGateSummary(PassthroughArgs),
    #[command(name = "docs-generate", hide = true)]
    DocsGenerate(PassthroughArgs),
    #[command(name = "docs-generate-check", hide = true)]
    DocsGenerateCheck(PassthroughArgs),
    #[command(name = "conformance-parity", hide = true)]
    ConformanceParity(PassthroughArgs),
    #[command(name = "runner-certify", hide = true)]
    RunnerCertify(PassthroughArgs),
    #[command(name = "test-core", hide = true)]
    TestCore(PassthroughArgs),
    #[command(name = "test-full", hide = true)]
    TestFull(PassthroughArgs),
    #[command(name = "job-run", hide = true)]
    JobRun(PassthroughArgs),

    #[command(name = "spec-eval", hide = true)]
    SpecEval(PassthroughArgs),
    #[command(name = "critical-gate", hide = true)]
    CriticalGate(PassthroughArgs),
    #[command(name = "governance-broad-native", hide = true)]
    GovernanceBroadNative(PassthroughArgs),
    #[command(name = "spec-ref", hide = true)]
    SpecRef(PassthroughArgs),
    #[command(name = "validate-report", hide = true)]
    ValidateReport(PassthroughArgs),
    #[command(name = "governance-heavy", hide = true)]
    GovernanceHeavy(PassthroughArgs),
    #[command(name = "spec-lang-lint", hide = true)]
    SpecLangLint(PassthroughArgs),
    #[command(name = "spec-lang-format", hide = true)]
    SpecLangFormat(PassthroughArgs),
    #[command(name = "migrate-contract-step-imports-v1", hide = true)]
    MigrateContractStepImportsV1(PassthroughArgs),
    #[command(name = "migrate-case-doc-metadata-v1", hide = true)]
    MigrateCaseDocMetadataV1(PassthroughArgs),
    #[command(name = "migrate-library-docs-metadata-v1", hide = true)]
    MigrateLibraryDocsMetadataV1(PassthroughArgs),
    #[command(name = "migrate-case-domain-prefix-v1", hide = true)]
    MigrateCaseDomainPrefixV1(PassthroughArgs),
    #[command(name = "normalize-check", hide = true)]
    NormalizeCheck(PassthroughArgs),
    #[command(name = "normalize-fix", hide = true)]
    NormalizeFix(PassthroughArgs),
    #[command(name = "schema-registry-check", hide = true)]
    SchemaRegistryCheck(PassthroughArgs),
    #[command(name = "schema-registry-build", hide = true)]
    SchemaRegistryBuild(PassthroughArgs),
    #[command(name = "schema-docs-check", hide = true)]
    SchemaDocsCheck(PassthroughArgs),
    #[command(name = "schema-docs-build", hide = true)]
    SchemaDocsBuild(PassthroughArgs),
    #[command(name = "spec-portability-json", hide = true)]
    SpecPortabilityJson(PassthroughArgs),
    #[command(name = "spec-portability-md", hide = true)]
    SpecPortabilityMd(PassthroughArgs),
    #[command(name = "spec-lang-adoption-json", hide = true)]
    SpecLangAdoptionJson(PassthroughArgs),
    #[command(name = "spec-lang-adoption-md", hide = true)]
    SpecLangAdoptionMd(PassthroughArgs),
    #[command(name = "docs-operability-json", hide = true)]
    DocsOperabilityJson(PassthroughArgs),
    #[command(name = "docs-operability-md", hide = true)]
    DocsOperabilityMd(PassthroughArgs),
    #[command(name = "contract-assertions-json", hide = true)]
    ContractAssertionsJson(PassthroughArgs),
    #[command(name = "contract-assertions-md", hide = true)]
    ContractAssertionsMd(PassthroughArgs),
    #[command(name = "objective-scorecard-json", hide = true)]
    ObjectiveScorecardJson(PassthroughArgs),
    #[command(name = "objective-scorecard-md", hide = true)]
    ObjectiveScorecardMd(PassthroughArgs),
    #[command(name = "spec-lang-stdlib-json", hide = true)]
    SpecLangStdlibJson(PassthroughArgs),
    #[command(name = "spec-lang-stdlib-md", hide = true)]
    SpecLangStdlibMd(PassthroughArgs),
    #[command(name = "ci-cleanroom", hide = true)]
    CiCleanroom(PassthroughArgs),
    #[command(name = "perf-smoke", hide = true)]
    PerfSmoke(PassthroughArgs),
    #[command(name = "docs-build", hide = true)]
    DocsBuild(PassthroughArgs),
    #[command(name = "docs-build-check", hide = true)]
    DocsBuildCheck(PassthroughArgs),
    #[command(name = "docs-lint", hide = true)]
    DocsLint(PassthroughArgs),
    #[command(name = "docs-graph", hide = true)]
    DocsGraph(PassthroughArgs),
    #[command(name = "service-plugin-check", hide = true)]
    ServicePluginCheck(PassthroughArgs),
    #[command(name = "help-advanced")]
    HelpAdvanced,
}

#[derive(Debug, Clone, Args)]
pub struct PassthroughArgs {
    #[arg(value_name = "ARGS", trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct SpecsCommand {
    #[command(subcommand)]
    pub command: SpecsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SpecsSubcommand {
    /// List discovered spec cases
    List {
        #[arg(long)]
        path: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Run a single spec case by ref (/specs/...#ID)
    Run {
        #[arg(long)]
        r#ref: String,
    },
    /// Run all spec cases under specs root
    RunAll {
        #[arg(long)]
        root: Option<String>,
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "continue_on_fail")]
        fail_fast: bool,
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "fail_fast")]
        continue_on_fail: bool,
    },
    /// Run style/spec hygiene checks
    Check,
}

#[derive(Debug, Clone, Args)]
pub struct QualityCommand {
    #[command(subcommand)]
    pub command: QualitySubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum QualitySubcommand {
    Lint,
    Typecheck,
    Compilecheck,
    StyleCheck,
    TestCore,
    TestFull,
}

#[derive(Debug, Clone, Args)]
pub struct GovernanceCommand {
    #[command(subcommand)]
    pub command: GovernanceSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum GovernanceSubcommand {
    Run,
    Heavy,
    Broad,
    Critical,
}

#[derive(Debug, Clone, Args)]
pub struct DocsCommand {
    #[command(subcommand)]
    pub command: DocsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DocsSubcommand {
    Generate,
    GenerateCheck,
    Build,
    BuildCheck,
    Lint,
    Graph,
}

#[derive(Debug, Clone, Args)]
pub struct ReportsCommand {
    #[command(subcommand)]
    pub command: ReportsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReportsSubcommand {
    ConformancePurposeJson,
    ConformancePurposeMd,
    RunnerIndependenceJson,
    RunnerIndependenceMd,
    PythonDependencyJson,
    PythonDependencyMd,
    SpecPortabilityJson,
    SpecPortabilityMd,
    SpecLangAdoptionJson,
    SpecLangAdoptionMd,
    DocsOperabilityJson,
    DocsOperabilityMd,
    ContractAssertionsJson,
    ContractAssertionsMd,
    ObjectiveScorecardJson,
    ObjectiveScorecardMd,
    SpecLangStdlibJson,
    SpecLangStdlibMd,
}

#[derive(Debug, Clone, Args)]
pub struct CiCommand {
    #[command(subcommand)]
    pub command: CiSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CiSubcommand {
    GateSummary,
    Cleanroom,
    ConformanceParity,
    RunnerCertify,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub command: ProjectSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProjectSubcommand {
    Scaffold {
        #[arg(long = "project-root")]
        project_root: String,
        #[arg(long = "bundle-id")]
        bundle_id: Option<String>,
        #[arg(long = "bundle-version")]
        bundle_version: Option<String>,
        #[arg(long = "bundle-url")]
        bundle_url: Option<String>,
        #[arg(long = "sha256")]
        sha256: Option<String>,
        #[arg(long = "allow-external", action = ArgAction::SetTrue)]
        allow_external: bool,
        #[arg(long = "runner")]
        runner: Option<String>,
        #[arg(long = "var")]
        vars: Vec<String>,
        #[arg(long = "overwrite", action = ArgAction::SetTrue)]
        overwrite: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
