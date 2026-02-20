mod cli;
mod commands;
mod compat;
mod constants;
mod error;
mod git;
mod lockfile;
mod manifest;
mod output;
mod runner_spec_registry;
mod sync;

use clap::Parser;
use serde_json::json;

use crate::cli::{
    CheckArgs, Cli, Command, CompatCommand, LegacySourceOnlyArgs, RunnerSpecCommand, SpecCommand,
    SyncArgs, VerifyCommand,
};
use crate::error::AppResult;

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let code = err.exit_code();
            let _ = err.print();
            std::process::exit(code);
        }
    };

    let json_mode = command_json_mode(&cli.command);
    if let Err(error) = run(cli) {
        output::err(
            "xtask",
            &error.message(),
            json_mode,
            json!({ "exit_code": error.exit_code() }),
        );
        std::process::exit(error.exit_code());
    }
}

fn command_json_mode(command: &Command) -> bool {
    match command {
        Command::Spec(spec) => match &spec.command {
            SpecCommand::Sync(args) => args.json,
            SpecCommand::Check(args) => args.json,
        },
        Command::RunnerSpec(spec) => match &spec.command {
            RunnerSpecCommand::Sync(args) => args.json,
            RunnerSpecCommand::Check(args) => args.json,
        },
        Command::Compat(compat) => match &compat.command {
            CompatCommand::Check(args) => args.json,
        },
        Command::Verify(verify) => match &verify.command {
            Some(VerifyCommand::All(args)) => args.json,
            None => false,
        },
        _ => false,
    }
}

fn run(cli: Cli) -> AppResult<()> {
    match cli.command {
        Command::Build => {
            commands::build::run()?;
            Ok(())
        }
        Command::Test => {
            commands::test::run()?;
            Ok(())
        }
        Command::Smoke => {
            commands::smoke::run()?;
            Ok(())
        }
        Command::Spec(spec) => match spec.command {
            SpecCommand::Sync(SyncArgs {
                tag,
                source,
                allow_ref,
                json,
            }) => commands::spec::sync(&tag, source.as_deref(), allow_ref, json),
            SpecCommand::Check(CheckArgs { source, json }) => {
                commands::spec::check(source.as_deref(), json)
            }
        },
        Command::RunnerSpec(spec) => match spec.command {
            RunnerSpecCommand::Sync(SyncArgs {
                tag,
                source,
                allow_ref,
                json,
            }) => commands::runner_spec::sync(&tag, source.as_deref(), allow_ref, json),
            RunnerSpecCommand::Check(CheckArgs { source, json }) => {
                commands::runner_spec::check(source.as_deref(), json)
            }
        },
        Command::Compat(compat) => match compat.command {
            CompatCommand::Check(CheckArgs { source, json }) => {
                commands::compat::check(source.as_deref(), json)
            }
        },
        Command::Verify(verify) => match verify.command {
            Some(VerifyCommand::All(CheckArgs { source, json })) => {
                commands::verify::all(source.as_deref(), json)
            }
            None => {
                deprecated_alias("verify", "verify all [--source <path-or-url>]");
                commands::verify::all(None, false)
            }
        },
        Command::LegacySpecSync(args) => {
            deprecated_alias(
                "spec-sync",
                "spec sync --tag <tag> [--source <path-or-url>]",
            );
            commands::spec::sync(&args.tag, args.source.as_deref(), true, false)
        }
        Command::LegacySpecSyncCheck(LegacySourceOnlyArgs { source }) => {
            deprecated_alias("spec-sync-check", "spec check [--source <path-or-url>]");
            commands::spec::check(source.as_deref(), false)
        }
        Command::LegacyCompatCheck(LegacySourceOnlyArgs { source }) => {
            deprecated_alias("compat-check", "compat check [--source <path-or-url>]");
            commands::compat::check(source.as_deref(), false)
        }
        Command::LegacyRunnerSpecSync(args) => {
            deprecated_alias(
                "runner-spec-sync",
                "runner-spec sync --tag <tag> [--source <path-or-url>]",
            );
            commands::runner_spec::sync(&args.tag, args.source.as_deref(), true, false)
        }
        Command::LegacyRunnerSpecCheck(LegacySourceOnlyArgs { source }) => {
            deprecated_alias(
                "runner-spec-check",
                "runner-spec check [--source <path-or-url>]",
            );
            commands::runner_spec::check(source.as_deref(), false)
        }
    }
}

fn deprecated_alias(alias: &str, canonical: &str) {
    output::warn(&format!(
        "'{alias}' is deprecated and will be removed next minor release; use '{canonical}'"
    ));
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;

    #[test]
    fn usage_errors_map_to_exit_code_two() {
        let err = AppError::usage("bad args");
        assert_eq!(err.exit_code(), 2);
    }
}
