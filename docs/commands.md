# Commands

## Maintainer Command Reference

### Build and test

```sh
make build
make test
```

### Smoke

```sh
make smoke
```

### Full verification

```sh
make verify
```

### Upstream compatibility snapshot update

```sh
make spec-sync TAG=<upstream-tag> SOURCE=<path-or-url>
```

### Upstream snapshot integrity check

```sh
make spec-sync-check
```

Optional explicit source resolution check:

```sh
make spec-sync-check SOURCE=https://github.com/jonruttan/data-contracts.git
```

### Runner compatibility check

```sh
make compat-check
```

## Exit Behavior

Runner command contract (via `/runner_adapter.sh`):

- `0`: success
- `1`: runtime/tool failure
- `2`: usage/config error

Script/check failures return non-zero and should be treated as merge-blocking
for required-lane flows.

## Troubleshooting

| Symptom | Likely Cause | Action |
|---|---|---|
| `spec-sync-check` fails manifest drift | Snapshot changed without lock/manifest update | Re-run `make spec-sync TAG=...` and commit lock+manifest+snapshot |
| `compat-check` fails missing required subcommand | Adapter surface drifted from upstream contract | Compare `/runner_adapter.sh` against required subcommands in `/specs/upstream/data-contracts/specs/contract/12_runner_interface.md` |
| `compat-check` fails lock tag resolution with `SOURCE=` | Upstream ref/tag changed or unavailable | Verify upstream tag exists or use local source path |
| `make verify` fails in build/test | Rust compile/test regression | Fix code/test failures before snapshot updates |
