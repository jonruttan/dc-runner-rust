use serde_json::json;

use crate::error::AppResult;
use crate::output;
use crate::sync;

pub fn sync(tag: &str, source: Option<&str>, allow_ref: bool, json_mode: bool) -> AppResult<()> {
    let report = sync::spec_sync(tag, source, allow_ref)?;
    if report.non_tag_ref && !json_mode {
        output::warn(&format!(
            "'{}' resolved as non-tag git ref; release workflow should use immutable tags.",
            report.tag
        ));
    }
    output::ok(
        "spec sync",
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
    let report = sync::spec_sync_check(source)?;
    output::ok(
        "spec check",
        "upstream snapshot and lock are consistent",
        json_mode,
        json!({
            "commit": report.commit,
            "file_count": report.file_count,
            "manifest_sha256": report.manifest_hash,
        }),
    );
    Ok(())
}
