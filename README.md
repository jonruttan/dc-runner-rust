# dc-runner-rust

Rust implementation of the required Data Contracts runner lane.

## What This Project Is

`dc-runner-rust` provides the stable Rust runner interface used by Data Contracts
gates and automation. It preserves command-surface and exit-code semantics while
executing the required lane without cross-runtime delegation.

## What Is Data Contracts?

[Data Contracts](https://github.com/jonruttan/data-contracts) is the canonical
contracts/specifications project for the runner ecosystem. It owns normative
contracts, schemas, conformance surfaces, and governance policy.

This repository does not redefine those canonical contracts. Instead, it
implements the Rust required lane and verifies compatibility against pinned
upstream snapshots.

## Responsibility Boundary

- `data-contracts` owns canonical global runner specs/contracts and their evolution.
- `data-contracts-runner` owns canonical runner-specific implementation specs.
- `dc-runner-rust` owns the Rust implementation and compatibility verification
  against pinned upstream versions.

## Stable Interface Contract

- Canonical runner entrypoint: Rust CLI (`dc-runner`)
- Published crates.io package: `dc-runner-cli`
- Stable exit code semantics:
  - `0` success
  - `1` runtime/tool failure
  - `2` invalid usage/config
- Required lane runtime remains Rust-first.

## Install

### Cargo install (recommended)

```sh
cargo install dc-runner-cli --locked
```

### GitHub release binary (macOS/Linux)

```sh
PLATFORM="darwin-arm64" # or linux-x86_64
VERSION="v0.2.0"        # set desired release tag
BASE="https://github.com/jonruttan/dc-runner-rust/releases/download/${VERSION}"
curl -fL "${BASE}/dc-runner-${PLATFORM}" -o dc-runner
curl -fL "${BASE}/dc-runner-${PLATFORM}.sha256" -o dc-runner.sha256
shasum -a 256 -c dc-runner.sha256
chmod +x dc-runner
sudo mv dc-runner /usr/local/bin/dc-runner
```

### Verify install

```sh
dc-runner --help
dc-runner governance --help
dc-runner governance critical --help
```

## Quickstart Commands

Most common runner CLI commands:

```sh
dc-runner specs run-all
dc-runner specs list
dc-runner schema check
dc-runner schema lint
dc-runner schema format
dc-runner docs generate-check
dc-runner docs lint
dc-runner docs build-check
dc-runner governance run
```

## Bundle-first Scaffold (Canonical)

Canonical scaffold/install flow uses bundle ID plus explicit bundle version from
`jonruttan/data-contracts-bundles` releases:

```sh
dc-runner project scaffold --project-root ./my-project --bundle-id data-contracts-lang-project-scaffold --bundle-version 1.0.0
```

External URL scaffold mode is available for emergency/manual recovery only and
is non-canonical:

```sh
dc-runner project scaffold --project-root ./my-project --bundle-url <url> --sha256 <hex> --allow-external
```

Optional bundler command group (feature-gated build):

```sh
dc-runner bundler resolve
dc-runner bundler package
dc-runner bundler check
```

## Spec Source Mode

`dc-runner` ships with an embedded `data-contracts` spec snapshot and resolves
spec refs from the bundled payload by default.

- Default: `bundled`
- Override per command: `--spec-source bundled|workspace|auto`
- Override via env: `DC_RUNNER_SPEC_SOURCE=bundled|workspace|auto`

Examples:

```sh
dc-runner --spec-source bundled entrypoints list --format json
dc-runner --spec-source workspace docs generate-check
DC_RUNNER_SPEC_SOURCE=auto dc-runner governance run
```

For full command help:

```sh
dc-runner --help
```

For advanced profiling/liveness flags:

```sh
dc-runner help-advanced
```

Source-run maintainer equivalent:

```sh
cargo run -q -p dc-runner-cli -- --help
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

## Release publishing behavior

Release artifacts are published by the `Release` GitHub Action when a release commit lands on `main`
from the release-please flow.

- `main` changes to release files trigger a check:
  - `dc-runner-cli/Cargo.toml`
  - `dc-runner-cli/CHANGELOG.md`
  - `.release-please-manifest.json`
- The workflow publishes only when the commit matches a release commit pattern or when manually forced.
- Manual force is available via `workflow_dispatch` (`force_publish: true`) for emergencies.

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

Pinned runner-specific artifacts (`data-contracts-runner`):

- `/specs/upstream/dc_runner_spec_lock_v1.yaml`
- `/specs/upstream/data-contracts-runner.manifest.sha256`
- `/specs/upstream/data-contracts-runner/`

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
./scripts/verify_upstream_compat.sh --strict --runner-bin "$(command -v dc-runner)"
./scripts/sync_runner_specs.sh --check
./scripts/verify_runner_specs.sh
```

## Documentation Map

- Architecture: `/docs/architecture.md`
- Install: `/docs/install.md`
- Commands: `/docs/commands.md`
- Compatibility: `/docs/compatibility.md`
- Release operations: `/docs/release.md`
- Contributor workflow: `/CONTRIBUTING.md`

## Crate Publishing

- crates.io package: `dc-runner-cli`
- release orchestration: `/.github/workflows/release-please.yml` (opens/updates Release PR from conventional commits on `main`)
- publish executor: `/.github/workflows/release.yml` (runs on `vX.Y.Z` tags created by the release PR merge)
- CI publish auth: `CRATES_IO_TOKEN` repository secret

## Specs Map

- Local runner-owned implementation specs:
  - `/specs/impl/rust/runner_spec_registry_v1.yaml`
  - `/specs/impl/rust/jobs/` (Phase 1 compatibility copies)
- Upstream pinned compatibility snapshot:
  - `/specs/upstream/data-contracts/`
  - `/specs/upstream/data-contracts-runner/`

## Source Ownership

Rust-specific implementation specs are canonically owned in `data-contracts-runner` and
consumed here via pinned vendored snapshot at `/specs/upstream/data-contracts-runner/`.
