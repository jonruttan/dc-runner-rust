#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="${ROOT_DIR}/target/debug/spec_runner_cli"

echo "WARN: runner_adapter.sh is deprecated and will be removed in a future release; use Rust runner binary instead." >&2

if [[ ! -x "${BIN_PATH}" ]]; then
  cargo build --manifest-path "${ROOT_DIR}/spec_runner_cli/Cargo.toml" >&2
fi

exec "${BIN_PATH}" "$@"
