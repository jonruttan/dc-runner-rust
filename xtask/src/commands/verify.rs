use serde_json::json;

use crate::commands;
use crate::error::AppResult;
use crate::output;

pub fn all(source: Option<&str>, json_mode: bool) -> AppResult<()> {
    commands::build::run()?;
    commands::test::run()?;
    commands::spec::check(source, json_mode)?;
    commands::runner_spec::check(None, json_mode)?;
    commands::compat::check(source, json_mode)?;
    output::ok(
        "verify all",
        "build + test + spec check + runner-spec check + compat check completed",
        json_mode,
        json!({ "sequence": ["build", "test", "spec check", "runner-spec check", "compat check"] }),
    );
    Ok(())
}
