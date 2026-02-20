use serde_json::json;

use crate::error::AppResult;
use crate::output;
use crate::runner_spec_registry;
use crate::sync;

pub fn sync(tag: &str, source: Option<&str>, allow_ref: bool, json_mode: bool) -> AppResult<()> {
    let report = sync::runner_spec_sync(tag, source, allow_ref)?;
    if report.non_tag_ref && !json_mode {
        output::warn(&format!(
            "'{}' resolved as non-tag git ref; release workflow should use immutable tags.",
            report.tag
        ));
    }
    output::ok(
        "runner-spec sync",
        &format!(
            "wrote lock + snapshot + manifest for {} ({})",
            report.tag, report.commit
        ),
        json_mode,
        json!({
            "tag": report.tag,
            "commit": report.commit,
            "file_count": report.file_count,
            "manifest_sha256": report.manifest_hash,
            "non_tag_ref": report.non_tag_ref,
        }),
    );
    Ok(())
}

pub fn check(source: Option<&str>, json_mode: bool) -> AppResult<()> {
    let report = sync::runner_spec_check(source)?;
    let registry_case_count = runner_spec_registry::validate_runner_spec_registry()?;
    output::ok(
        "runner-spec check",
        "runner-spec snapshot, lock, and rust registry are consistent",
        json_mode,
        json!({
            "commit": report.commit,
            "file_count": report.file_count,
            "manifest_sha256": report.manifest_hash,
            "registry_case_count": registry_case_count,
        }),
    );
    Ok(())
}
