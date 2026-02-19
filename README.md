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
implements the Rust required lane and verifies compatibility against a pinned
upstream snapshot.

## Responsibility Boundary

- `data-contracts` owns canonical runner specs/contracts and their evolution.
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
cargo xtask verify
```

## Upstream Snapshot Workflow

Pinned upstream compatibility artifacts:

- `/specs/upstream/data_contracts_lock_v1.yaml`
- `/specs/upstream/data-contracts.manifest.sha256`
- `/specs/upstream/data-contracts/`

Update pinned snapshot:

```sh
cargo xtask spec-sync --tag <upstream-tag> --source <path-or-url>
```

Validate lock/snapshot integrity:

```sh
cargo xtask spec-sync-check
```

Run compatibility verification:

```sh
cargo xtask compat-check
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
  - `/specs/impl/rust/jobs/`
- Upstream pinned compatibility snapshot:
  - `/specs/upstream/data-contracts/`

## Source Moved From data-contracts

Implementation job specs under `/specs/impl/rust/jobs/` are canonical in this
repository. Any legacy copies from `data-contracts/specs/impl/rust/jobs/` are
deprecated and removed from control-plane scope.
