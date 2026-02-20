use serde_json::json;

use crate::compat;
use crate::error::AppResult;
use crate::output;

pub fn check(source: Option<&str>, json_mode: bool) -> AppResult<()> {
    let report = compat::compat_check(source)?;
    output::ok(
        "compat check",
        "upstream compatibility verification passed",
        json_mode,
        json!({
            "required_subcommands_count": report.required_subcommands_count,
        }),
    );
    Ok(())
}
