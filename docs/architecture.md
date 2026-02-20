# Architecture

## System Boundary

Text boundary model:

- Data Contracts upstream (`https://github.com/jonruttan/data-contracts`):
  canonical contracts/specs/schemas/governance definitions.
- Runner spec upstream (`https://github.com/jonruttan/dc-runner-spec`):
  canonical runner-owned implementation specs.
- This repository (`dc-runner-rust`): Rust required-lane implementation and
  compatibility verification logic.

## Artifact Ownership

Local runner-owned artifacts:

- `/runner_adapter.sh` (temporary compatibility shim)
- `/spec_runner_cli/**`
- `/specs/impl/rust/jobs/**`
- `/specs/impl/rust/runner_spec_registry_v1.yaml`
- `/xtask/**`

Pinned upstream compatibility artifacts:

- `/specs/upstream/data_contracts_lock_v1.yaml`
- `/specs/upstream/data-contracts.manifest.sha256`
- `/specs/upstream/data-contracts/**`
- `/specs/upstream/dc_runner_spec_lock_v1.yaml`
- `/specs/upstream/dc-runner-spec.manifest.sha256`
- `/specs/upstream/dc-runner-spec/**`

## Execution Model

Runtime flow:

1. Caller invokes `/runner_adapter.sh <subcommand>`.
2. Shim delegates to the Rust CLI binary.
3. `/spec_runner_cli` handles subcommand execution.
4. CLI runs checks/gates and returns stable exit semantics.

## Compatibility Verification Model

The compatibility model is lock-driven and deterministic:

1. Lock files record upstream repo/tag/commit and integrity metadata.
2. Vendored snapshots hold curated upstream compatibility and runner-specific surfaces.
3. Manifests track deterministic per-file checksums.
4. Rust registry maps local case IDs to vendored runner-specific files.
5. `cargo xtask` verification enforces lock/snapshot/manifest coherence, registry
   integrity, and runner interface compatibility.

## Change Impact Matrix

- Rust runtime behavior changes:
  - update implementation
  - run `/cargo xtask verify all`
  - update docs if interface/operations changed
- Upstream compatibility version bump:
  - run `/cargo xtask spec sync --tag <tag> --source <source>`
  - run `/cargo xtask runner-spec sync --tag <tag> --source <source>`
  - run `/cargo xtask verify all`
  - commit lock + snapshot + manifest (+ registry updates when needed)
- Runner interface or command semantics changes:
  - preserve compatibility guarantees, or treat as explicit breaking change
  - update `/README.md`, `/CONTRIBUTING.md`, and command docs
