# Contributing

## Local Setup

Required local tools:

- `cargo`
- `git`

Core setup check:

```sh
cargo xtask build
cargo xtask test
```

## Required Local Checks

Before opening a PR, run the canonical verification flow:

```sh
cargo xtask verify all
```

This runs build, tests, pinned snapshot integrity checks (`data-contracts` and
`data-contracts-runner`), Rust case registry validation, and compatibility
verification.

## Updating Pinned Upstream Compatibility Snapshot

Canonical specs are owned by `data-contracts`. To bump this runner against a
new upstream version:

```sh
cargo xtask spec sync --tag <upstream-tag> --source <path-or-url>
cargo xtask verify all
```

If you intentionally pin a non-tag ref for local iteration, add `--allow-ref`.

Review and commit all resulting changes together:

- `/specs/upstream/data_contracts_lock_v1.yaml`
- `/specs/upstream/data-contracts.manifest.sha256`
- `/specs/upstream/data-contracts/**`
- any implementation updates required for compatibility

## Updating Pinned Runner-Specific Snapshot

Canonical runner-specific specs are owned by `data-contracts-runner`. To bump this
runner against a new `data-contracts-runner` version:

```sh
cargo xtask runner-spec sync --tag <runner-spec-tag> --source <path-or-url>
cargo xtask verify all
```

If you intentionally pin a non-tag ref for local iteration, add `--allow-ref`.

Review and commit all resulting changes together:

- `/specs/upstream/dc_runner_spec_lock_v1.yaml`
- `/specs/upstream/data-contracts-runner.manifest.sha256`
- `/specs/upstream/data-contracts-runner/**`
- `/specs/impl/rust/runner_spec_registry_v1.yaml` (when updated)

## Pull Request Expectations

- Keep scope focused and behaviorally coherent.
- Preserve stable runner interface semantics and exit codes.
- Update docs/spec references when behavior or workflows change.
- Use conventional-commit PR titles so release automation can classify version bumps.

Required PR title format:

```text
<type>(optional-scope): summary
```

Allowed types:

- `feat`
- `fix`
- `perf`
- `refactor`
- `docs`
- `chore`
- `test`
- `build`
- `ci`

## Compatibility Invariants Checklist

Do not regress these invariants:

- Stable public command surface in `dc-runner` (compatibility alias `scripts/dc-runner-rust` is non-canonical)
- Stable exit code semantics (`0/1/2`)
- Rust-first required-lane execution (no Python runtime dependency)
- Compatibility checks remain pinned to `/specs/upstream/data-contracts/`
- Rust case registry must resolve against `/specs/upstream/data-contracts-runner/`

## Reference Docs

- `/docs/architecture.md`
- `/docs/commands.md`
- `/docs/compatibility.md`
- `/docs/release.md`
