use std::process::Command;

use anyhow::Result;

use crate::git::run_inherited;

pub fn run() -> Result<()> {
    run_inherited(Command::new("cargo").args([
        "test",
        "--manifest-path",
        "spec_runner_cli/Cargo.toml",
    ]))
}
