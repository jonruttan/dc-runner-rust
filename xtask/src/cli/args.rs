use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about = "Rust-native task runner for dc-runner-rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Build,
    Test,
    Smoke,
    Spec(SpecArgs),
    RunnerSpec(RunnerSpecArgs),
    Compat(CompatArgs),
    Verify(VerifyArgs),

    #[command(name = "spec-sync", hide = true)]
    LegacySpecSync(LegacySpecSyncArgs),
    #[command(name = "spec-sync-check", hide = true)]
    LegacySpecSyncCheck(LegacySourceOnlyArgs),
    #[command(name = "compat-check", hide = true)]
    LegacyCompatCheck(LegacySourceOnlyArgs),
    #[command(name = "runner-spec-sync", hide = true)]
    LegacyRunnerSpecSync(LegacySpecSyncArgs),
    #[command(name = "runner-spec-check", hide = true)]
    LegacyRunnerSpecCheck(LegacySourceOnlyArgs),
}

#[derive(Debug, Args)]
pub struct SpecArgs {
    #[command(subcommand)]
    pub command: SpecCommand,
}

#[derive(Debug, Subcommand)]
pub enum SpecCommand {
    Sync(SyncArgs),
    Check(CheckArgs),
}

#[derive(Debug, Args)]
pub struct RunnerSpecArgs {
    #[command(subcommand)]
    pub command: RunnerSpecCommand,
}

#[derive(Debug, Subcommand)]
pub enum RunnerSpecCommand {
    Sync(SyncArgs),
    Check(CheckArgs),
}

#[derive(Debug, Args)]
pub struct CompatArgs {
    #[command(subcommand)]
    pub command: CompatCommand,
}

#[derive(Debug, Subcommand)]
pub enum CompatCommand {
    Check(CheckArgs),
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    #[command(subcommand)]
    pub command: Option<VerifyCommand>,
}

#[derive(Debug, Subcommand)]
pub enum VerifyCommand {
    All(CheckArgs),
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    #[arg(long)]
    pub tag: String,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub allow_ref: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct CheckArgs {
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct LegacySpecSyncArgs {
    pub tag: String,
    pub source: Option<String>,
}

#[derive(Debug, Args)]
pub struct LegacySourceOnlyArgs {
    pub source: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_canonical_spec_sync() {
        let cli = Cli::try_parse_from(["xtask", "spec", "sync", "--tag", "v0.2.0", "--allow-ref"])
            .expect("parse");
        match cli.command {
            Command::Spec(args) => match args.command {
                SpecCommand::Sync(sync) => {
                    assert_eq!(sync.tag, "v0.2.0");
                    assert!(sync.allow_ref);
                }
                _ => panic!("wrong command"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parses_legacy_alias() {
        let cli = Cli::try_parse_from(["xtask", "spec-sync", "v0.2.0"]).expect("parse");
        assert!(matches!(cli.command, Command::LegacySpecSync(_)));
    }

    #[test]
    fn parses_runner_spec_legacy_alias() {
        let cli = Cli::try_parse_from(["xtask", "runner-spec-sync", "v0.1.0"]).expect("parse");
        assert!(matches!(cli.command, Command::LegacyRunnerSpecSync(_)));
    }
}
