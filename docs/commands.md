# Commands

## Runner Quickstart

Use the installed runner CLI first:

```sh
dc-runner --help
```

Common commands:

```sh
dc-runner specs run-all
dc-runner specs list
dc-runner specs run --ref /specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-001
dc-runner schema check
dc-runner schema lint
dc-runner schema format
dc-runner docs generate
dc-runner docs generate-check
dc-runner docs build
dc-runner docs build-check
dc-runner docs lint
dc-runner docs graph
```

## Scaffold from Canonical Bundles

Use bundle ID plus explicit version for canonical installs:

```sh
dc-runner project scaffold --project-root ./my-project --bundle-id data-contracts-lang-project-scaffold --bundle-version 1.0.0
```

External URL mode is non-canonical and intended only for emergency fallback:

```sh
dc-runner project scaffold --project-root ./my-project --bundle-url <url> --sha256 <hex> --allow-external
```

Optional bundler commands (only when compiled with `--features bundler`):

```sh
dc-runner bundler resolve
dc-runner bundler package
dc-runner bundler check
```

### Spec source selection

`dc-runner` resolves spec refs from the embedded snapshot by default.

```sh
dc-runner --spec-source bundled entrypoints list
dc-runner --spec-source workspace docs generate-check
DC_RUNNER_SPEC_SOURCE=auto dc-runner governance run
```

## Spec Entrypoints (diagnostic)

Entrypoints remain available for diagnostics/governance validation:

```sh
dc-runner entrypoints list
dc-runner entrypoints list --format json
dc-runner entrypoints run docs-generate-check
```

Advanced runtime flags (profiling/liveness) are intentionally hidden from
default help:

```sh
dc-runner help-advanced
```

Source-run maintainer equivalent:

```sh
cargo run -q -p dc-runner-cli -- --help
```

## Maintainer Command Reference

### Build and test

```sh
cargo xtask build
cargo xtask test
```

### Smoke

```sh
cargo xtask smoke
```

### Full verification

```sh
cargo xtask verify all
```

### Upstream compatibility snapshot update

```sh
cargo xtask spec sync --tag <upstream-tag> --source <path-or-url>
```

Non-tag refs are rejected unless explicitly enabled:

```sh
cargo xtask spec sync --tag <commit-or-branch> --source <path-or-url> --allow-ref
```

### Upstream snapshot integrity check

```sh
cargo xtask spec check
```

Optional explicit source resolution check:

```sh
cargo xtask spec check --source https://github.com/jonruttan/data-contracts.git
```

### Runner compatibility check

```sh
cargo xtask compat check
```

### Runner-specific snapshot update (`data-contracts-runner`)

```sh
cargo xtask runner-spec sync --tag <runner-spec-tag> --source <path-or-url>
```

Non-tag refs are rejected unless explicitly enabled:

```sh
cargo xtask runner-spec sync --tag <commit-or-branch> --source <path-or-url> --allow-ref
```

### Runner-specific snapshot and registry check

```sh
cargo xtask runner-spec check
```

### Shell compatibility wrappers (thin, xtask-backed)

```sh
./scripts/sync_data_contracts_specs.sh --check [--source <path-or-url>]
./scripts/sync_data_contracts_specs.sh --tag <tag-or-ref> [--source <path-or-url>] [--allow-ref]
./scripts/verify_upstream_compat.sh --strict --runner-bin <path> [--source <path-or-url>]
./scripts/sync_runner_specs.sh --check [--source <path-or-url>]
./scripts/sync_runner_specs.sh --tag <tag-or-ref> [--source <path-or-url>] [--allow-ref]
./scripts/verify_runner_specs.sh [--source <path-or-url>]
```

## Exit Behavior

Runner command contract (canonical Rust CLI):

- `0`: success
- `1`: runtime/tool failure
- `2`: usage/config error

Compatibility command surface remains callable for contract checks (`job-run`,
`style-check`, `ci-gate-summary`, and other required interface commands), but
is hidden from default top-level help to reduce cognitive load.

Script/check failures return non-zero and should be treated as merge-blocking
for required-lane flows.

## Troubleshooting

| Symptom | Likely Cause | Action |
|---|---|---|
| `spec check` fails manifest drift | Snapshot changed without lock/manifest update | Re-run `cargo xtask spec sync --tag ...` and commit lock+manifest+snapshot |
| `runner-spec check` fails manifest drift | Runner-specific snapshot changed without lock/manifest update | Re-run `cargo xtask runner-spec sync --tag ...` and commit lock+manifest+snapshot |
| `runner-spec check` fails registry validation | Rust case registry IDs or paths drifted from vendored source | Fix `/specs/impl/rust/runner_spec_registry_v1.yaml` to match vendored `data-contracts-runner` |
| `compat check` fails missing required subcommand | Runner surface drifted from upstream contract | Compare `dc-runner` behavior against `/specs/upstream/data-contracts/specs/02_contracts/12_runner_interface.md` |
| `compat check` fails lock tag resolution with `--source` | Upstream ref/tag changed or unavailable | Verify upstream tag exists or use local source path |
| `cargo xtask verify all` fails in build/test | Rust compile/test regression | Fix code/test failures before snapshot updates |
