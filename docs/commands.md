# Commands

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
cargo xtask verify
```

### Upstream compatibility snapshot update

```sh
cargo xtask spec-sync --tag <upstream-tag> --source <path-or-url>
```

### Upstream snapshot integrity check

```sh
cargo xtask spec-sync-check
```

Optional explicit source resolution check:

```sh
cargo xtask spec-sync-check --source https://github.com/jonruttan/data-contracts.git
```

### Runner compatibility check

```sh
cargo xtask compat-check
```

## Exit Behavior

Runner command contract (canonical Rust CLI with temporary `/runner_adapter.sh` shim):

- `0`: success
- `1`: runtime/tool failure
- `2`: usage/config error

Script/check failures return non-zero and should be treated as merge-blocking
for required-lane flows.

## Troubleshooting

| Symptom | Likely Cause | Action |
|---|---|---|
| `spec-sync-check` fails manifest drift | Snapshot changed without lock/manifest update | Re-run `cargo xtask spec-sync --tag ...` and commit lock+manifest+snapshot |
| `compat-check` fails missing required subcommand | Runner surface drifted from upstream contract | Compare `spec_runner_cli` behavior against `/specs/upstream/data-contracts/specs/contract/12_runner_interface.md` |
| `compat-check` fails lock tag resolution with `SOURCE=` | Upstream ref/tag changed or unavailable | Verify upstream tag exists or use local source path |
| `cargo xtask verify` fails in build/test | Rust compile/test regression | Fix code/test failures before snapshot updates |
