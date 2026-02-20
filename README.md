# dc-runner-rust

Rust implementation of the required Data Contracts runner lane.

## What This Project Is

`dc-runner-rust` provides the stable Rust runner interface used by Data Contracts
gates and automation. It preserves command-surface and exit-code semantics while
executing the required lane without Python runtime delegation.

## What Is Data Contracts?

[Data Contracts](https://github.com/jonruttan/data-contracts) is the canonical
contracts/specifications project for the runner ecosystem. It owns normative
contracts, schemas, conformance surfaces, and governance policy.

This repository does not redefine those canonical contracts. Instead, it
implements the Rust required lane and verifies compatibility against pinned
upstream snapshots.

## Responsibility Boundary

- `data-contracts` owns canonical global runner specs/contracts and their evolution.
- `dc-runner-spec` owns canonical runner-specific implementation specs.
- `dc-runner-rust` owns the Rust implementation and compatibility verification
  against pinned upstream versions.

## Stable Interface Contract

- Canonical runner entrypoint: Rust CLI (`spec_runner_cli`)
- Published crates.io package: `dc_runner_cli`
- Compatibility shim (one-release deprecation): `/runner_adapter.sh`
- Stable exit code semantics:
  - `0` success
  - `1` runtime/tool failure
  - `2` invalid usage/config
- Required lane runtime remains Rust-first and Python-free.

## Quickstart

Most common runner CLI commands:

```sh
cargo run -q -p dc_runner_cli
cargo run -q -p dc_runner_cli -- specs run-all
cargo run -q -p dc_runner_cli -- specs list
```

For full command help:

```sh
cargo run -q -p dc_runner_cli -- --help
```

For advanced profiling/liveness flags:

```sh
cargo run -q -p dc_runner_cli -- help-advanced
```

Build and test:

```sh
cargo xtask build
cargo xtask test
```

Smoke:

```sh
cargo xtask smoke
```

Full local verification:

```sh
cargo xtask verify all
```

Emit local status exchange artifact:

```sh
./scripts/emit_runner_status_report.sh
```

## Upstream Snapshot Workflows

Pinned upstream compatibility artifacts:

- `/specs/upstream/data_contracts_lock_v1.yaml`
- `/specs/upstream/data-contracts.manifest.sha256`
- `/specs/upstream/data-contracts/`

Update pinned snapshot:

```sh
cargo xtask spec sync --tag <upstream-tag> --source <path-or-url>
```

For local development only, non-tag refs require `--allow-ref`:

```sh
cargo xtask spec sync --tag <commit-or-branch> --source <path-or-url> --allow-ref
```

Validate lock/snapshot integrity:

```sh
cargo xtask spec check
```

Run compatibility verification:

```sh
cargo xtask compat check
```

Pinned runner-specific artifacts (`dc-runner-spec`):

- `/specs/upstream/dc_runner_spec_lock_v1.yaml`
- `/specs/upstream/dc-runner-spec.manifest.sha256`
- `/specs/upstream/dc-runner-spec/`

Update runner-specific snapshot:

```sh
cargo xtask runner-spec sync --tag <runner-spec-tag> --source <path-or-url>
```

Validate runner-specific lock/snapshot integrity and Rust case registry:

```sh
cargo xtask runner-spec check
```

Shell compatibility wrappers (xtask-backed):

```sh
./scripts/sync_data_contracts_specs.sh --check
./scripts/verify_upstream_compat.sh --strict --runner-bin ./runner_adapter.sh
./scripts/sync_runner_specs.sh --check
./scripts/verify_runner_specs.sh
```

## Documentation Map

- Architecture: `/docs/architecture.md`
- Commands: `/docs/commands.md`
- Compatibility: `/docs/compatibility.md`
- Release operations: `/docs/release.md`
- Contributor workflow: `/CONTRIBUTING.md`

## Crate Publishing

- crates.io package: `dc_runner_cli`
- publish flow: GitHub tag `vX.Y.Z` (must match crate version in `/spec_runner_cli/Cargo.toml`)
- CI publish auth: `CRATES_IO_TOKEN` repository secret

## Specs Map

- Local runner-owned implementation specs:
  - `/specs/impl/rust/runner_spec_registry_v1.yaml`
  - `/specs/impl/rust/jobs/` (Phase 1 compatibility copies)
- Upstream pinned compatibility snapshot:
  - `/specs/upstream/data-contracts/`
  - `/specs/upstream/dc-runner-spec/`

## Source Ownership

Rust-specific implementation specs are canonically owned in `dc-runner-spec` and
consumed here via pinned vendored snapshot at `/specs/upstream/dc-runner-spec/`.
