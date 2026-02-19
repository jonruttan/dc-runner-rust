# dc-runner-rust

Required Rust runner for Data Contracts.

## Scope

- Owns the required, blocking runner lane for `data-contracts`.
- Provides stable subcommand surface consumed by Data Contracts gates.

## Build

```sh
cargo build --manifest-path spec_runner_cli/Cargo.toml
```

## Run

```sh
./runner_adapter.sh critical-gate
./runner_adapter.sh governance
```

## Release artifacts

Release binaries are consumed by Data Contracts lock file and resolver.

## Implementation Specs

Runner-owned implementation contracts live in:

- `specs/impl/rust/jobs/`
