# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows SemVer.

## [Unreleased]


## [0.2.6] - 2026-02-24

### Added
- Add full spec state lifecycle management commands under `dc-runner specs`:
  `refresh`, `status`, `versions`, `use`, `rollback`, `verify`, `clean`, `info`, and `prune`.
- Add release behavior documentation clarifying main-merge auto-publish path.

### Fixed
- Fix spec state tests for local workspace fixtures under main-merge runtime defaults.

## [0.2.3] - 2026-02-24

### Added
- Publication readiness baseline (`.gitignore`, contribution guide, crate metadata).
- Upstream Data Contracts spec pinning + vendored compatibility snapshot workflow:
  lock file, manifest, sync/check tooling, compatibility verification, and CI gates.

### Changed
- Migrated operational tooling to Rust-native `cargo xtask` commands.
- Replaced shell script/Makefile operational path with Rust `xtask` implementation.
- Kept `/runner_adapter.sh` as a temporary compatibility shim that delegates to Rust CLI.
- Renamed publishable crate package to `dc_runner_cli` (binary name remains `spec_runner_cli`).
- Added tag-driven crates.io publish flow using `CRATES_IO_TOKEN` with strict tag/version matching.
- Refactored CLI composition to a layered module layout (`app`, `domain`, `ports`, `infra`, `services`) and reduced `main.rs` to a thin bootstrap.
- Added unit tests for extracted domain/service helpers and integration tests for contract-required dispatch and critical command exit semantics.
