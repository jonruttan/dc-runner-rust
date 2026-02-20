use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn manifest_entries(snapshot_root: &Path) -> Result<Vec<(String, String)>> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(snapshot_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let rel = path
            .strip_prefix(snapshot_root)
            .with_context(|| format!("failed to strip prefix for {}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(path).with_context(|| format!("failed reading {}", path.display()))?;
        entries.push((rel, sha256_bytes(&bytes)));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

pub fn write_manifest(snapshot_root: &Path, manifest_file: &Path) -> Result<()> {
    if let Some(parent) = manifest_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }
    let mut out = Vec::new();
    for (rel, hash) in manifest_entries(snapshot_root)? {
        out.extend_from_slice(format!("{hash}  {rel}\n").as_bytes());
    }
    fs::write(manifest_file, out)
        .with_context(|| format!("failed writing {}", manifest_file.display()))
}

pub fn verify_manifest(snapshot_root: &Path, manifest_file: &Path) -> Result<(u64, String)> {
    let mut tmp = tempfile::NamedTempFile::new().context("failed creating temp manifest")?;
    for (rel, hash) in manifest_entries(snapshot_root)? {
        writeln!(tmp, "{hash}  {rel}").context("failed writing temp manifest")?;
    }
    let computed = fs::read(tmp.path()).context("failed reading temp manifest")?;
    let existing = fs::read(manifest_file)
        .with_context(|| format!("failed reading {}", manifest_file.display()))?;
    if computed != existing {
        bail!("snapshot manifest drift detected. Run spec sync update.");
    }

    let file_count = existing
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .count() as u64;
    let manifest_hash = sha256_bytes(&existing);
    Ok((file_count, manifest_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_sorted() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("a")).expect("mkdir");
        std::fs::write(dir.path().join("b.txt"), "b").expect("write b");
        std::fs::write(dir.path().join("a/a.txt"), "a").expect("write a");
        let entries = manifest_entries(dir.path()).expect("entries");
        assert_eq!(entries[0].0, "a/a.txt");
        assert_eq!(entries[1].0, "b.txt");
    }
}
