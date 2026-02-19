# Contributing

## Development Setup

```sh
make build
make test
```

## Local Verification

Before opening a PR, run:

```sh
cargo build --locked --manifest-path spec_runner_cli/Cargo.toml
cargo test --locked --manifest-path spec_runner_cli/Cargo.toml
make smoke
make spec-sync-check
make compat-check
```

## Updating Upstream Compatibility Snapshot

Canonical specs are owned by `data-contracts`. To bump this runner to a newer
upstream version:

```sh
make spec-sync TAG=<upstream-tag> SOURCE=<path-or-url>
make verify
```

Commit all lock/snapshot/manifest changes with your runner updates.

## Compatibility Rules

- Preserve `runner_adapter.sh` subcommand interface semantics.
- Preserve adapter exit codes: `0` success, `1` runtime/tool failure, `2` usage/config error.
- Keep required-lane runtime execution Rust-first and Python-free.

## Pull Requests

- Keep changes focused and include tests when behavior changes.
- Update `CHANGELOG.md` for user-visible behavior changes.
