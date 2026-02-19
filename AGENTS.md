# AGENTS.md

Project-specific instructions for AI agents working in `dc-runner-rust`.

- Preserve runner command interface semantics and exit codes (`0/1/2`).
- Keep Rust lane deterministic and Python-free in runtime execution.
- Treat this repository as the required blocking lane for Data Contracts.
