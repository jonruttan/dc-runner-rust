use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

fn to_unix(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn emit_snapshot(source_root: &Path, prefix: &str, out_name: &str) {
    println!("cargo:rerun-if-changed={}", source_root.display());

    let mut files = Vec::<PathBuf>::new();
    collect_files(&source_root, &mut files);
    files.sort();

    let mut entries = Vec::<(String, String)>::new();
    let mut hasher = Sha256::new();
    for file in files {
        let rel = file
            .strip_prefix(&source_root)
            .expect("strip prefix for source root");
        let key = format!("{}/{}", prefix.trim_end_matches('/'), to_unix(rel));
        let text = fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", file.display()));
        hasher.update(key.as_bytes());
        hasher.update(b"\n");
        hasher.update(text.as_bytes());
        hasher.update(b"\n");
        entries.push((key, text));
    }

    let snapshot_sha256 = format!("{:x}", hasher.finalize());
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_path = out_dir.join(out_name);
    let mut out = fs::File::create(&out_path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", out_path.display()));

    writeln!(
        out,
        "pub const SNAPSHOT_SOURCE_ROOT: &str = {:?};",
        to_unix(&source_root)
    )
    .expect("write source root");
    writeln!(
        out,
        "pub const SNAPSHOT_SHA256: &str = {:?};",
        snapshot_sha256
    )
    .expect("write sha");
    writeln!(out, "pub static FILES: &[(&str, &str)] = &[").expect("write header");
    for (key, text) in entries {
        writeln!(out, "    ({:?}, {:?}),", key, text).expect("write row");
    }
    writeln!(out, "];").expect("write footer");
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let core_candidates = [
        manifest_dir.join("specs/upstream/data-contracts"),
        manifest_dir.join("../specs/upstream/data-contracts"),
    ];
    let core_root = core_candidates
        .iter()
        .find(|p| p.exists() && p.is_dir())
        .cloned()
        .expect("missing data-contracts snapshot source for embedding");
    emit_snapshot(
        &core_root,
        "specs/upstream/data-contracts",
        "embedded_data_contracts.rs",
    );

    if env::var("CARGO_FEATURE_BUNDLER").is_ok() {
        let bundler_candidates = [
            manifest_dir.join("specs/upstream/data-contracts-library"),
            manifest_dir.join("../specs/upstream/data-contracts-library"),
        ];
        let bundler_root = bundler_candidates
            .iter()
            .find(|p| p.exists() && p.is_dir())
            .cloned()
            .expect("missing data-contracts-library snapshot source for bundler embedding");
        emit_snapshot(
            &bundler_root,
            "specs/upstream/data-contracts-library",
            "embedded_data_contracts_library.rs",
        );
    }
}
