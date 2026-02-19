# dc-runner-rust

Required Rust runner for Data Contracts.

## Scope

- Owns the required, blocking runner lane for `data-contracts`.
- Provides stable subcommand surface consumed by Data Contracts gates.
- Preserves adapter contract semantics and exit codes (`0` success, `1` runtime/tool failure, `2` usage/config error).

## Requirements

- Rust stable toolchain
- `cargo`

## Build and Test

```sh
cargo build --manifest-path spec_runner_cli/Cargo.toml
cargo test --manifest-path spec_runner_cli/Cargo.toml
```

## Run

```sh
./runner_adapter.sh critical-gate
./runner_adapter.sh governance
```

## Contract

- Runtime execution for the required lane stays Rust-first and Python-free.
- `runner_adapter.sh` is the stable command surface consumed by upstream gates.
- Subcommand and exit-code compatibility is treated as a breaking interface.

## Upstream Spec Pinning

Canonical Data Contracts runner specs live in `data-contracts`.
This repository vendors a pinned compatibility snapshot under:

- `specs/upstream/data-contracts/`
- `specs/upstream/data-contracts.manifest.sha256`
- `specs/upstream/data_contracts_lock_v1.yaml`

Update snapshot (manual bump flow):

```sh
make spec-sync TAG=<upstream-tag> SOURCE=<path-or-url>
```

Validate lock + snapshot integrity:

```sh
make spec-sync-check
```

Run compatibility verification against pinned snapshot:

```sh
make compat-check
```

## Release artifacts

Release binaries are consumed by Data Contracts lock file and resolver.

## Contributing

See `CONTRIBUTING.md` for local workflow and quality checks.

## Implementation Specs

Runner-owned implementation contracts live in:

- `specs/impl/rust/jobs/`
