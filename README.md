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

## Release artifacts

Release binaries are consumed by Data Contracts lock file and resolver.

## Contributing

See `CONTRIBUTING.md` for local workflow and quality checks.

## Implementation Specs

Runner-owned implementation contracts live in:

- `specs/impl/rust/jobs/`
