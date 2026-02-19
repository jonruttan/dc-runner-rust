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

- Public runner entrypoint: `/runner_adapter.sh`
- Stable exit code semantics:
  - `0` success
  - `1` runtime/tool failure
  - `2` invalid usage/config
- Required lane runtime remains Rust-first and Python-free.

## Quickstart

Build and test:

```sh
make build
make test
```

Smoke:

```sh
make smoke
```

Full local verification:

```sh
make verify
```

## Upstream Snapshot Workflow

Pinned upstream compatibility artifacts:

- `/specs/upstream/data_contracts_lock_v1.yaml`
- `/specs/upstream/data-contracts.manifest.sha256`
- `/specs/upstream/data-contracts/`

Update pinned snapshot:

```sh
make spec-sync TAG=<upstream-tag> SOURCE=<path-or-url>
```

Validate lock/snapshot integrity:

```sh
make spec-sync-check
```

Run compatibility verification:

```sh
make compat-check
```

## Documentation Map

- Architecture: `/docs/architecture.md`
- Commands: `/docs/commands.md`
- Compatibility: `/docs/compatibility.md`
- Release operations: `/docs/release.md`
- Contributor workflow: `/CONTRIBUTING.md`

## Specs Map

- Local runner-owned implementation specs:
  - `/specs/impl/rust/jobs/`
- Upstream pinned compatibility snapshot:
  - `/specs/upstream/data-contracts/`
