# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows SemVer.

## [Unreleased]

### Added
- Publication readiness baseline (`.gitignore`, contribution guide, crate metadata).
- Upstream Data Contracts spec pinning + vendored compatibility snapshot workflow:
  lock file, manifest, sync/check tooling, compatibility verification, and CI gates.

### Changed
- Migrated operational tooling to Rust-native `cargo xtask` commands.
- Replaced shell script/Makefile operational path with Rust `xtask` implementation.
- Kept `/runner_adapter.sh` as a temporary compatibility shim that delegates to Rust CLI.
