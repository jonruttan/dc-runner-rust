use std::path::Path;
use std::process::Command;

use crate::ports::process::ProcessRunner;

pub struct StdProcessRunner;

impl ProcessRunner for StdProcessRunner {
    fn run_capture(&self, program: &str, args: &[String], root: &Path) -> Result<i32, String> {
        let status = Command::new(program)
            .args(args)
            .current_dir(root)
            .status()
            .map_err(|e| format!("failed to run {program}: {e}"))?;
        Ok(status.code().unwrap_or(1))
    }
}
