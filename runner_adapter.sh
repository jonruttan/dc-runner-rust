#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="${DC_RUNNER_BIN:-${ROOT_DIR}/target/debug/spec_runner_cli}"

if [[ ! -x "${BIN_PATH}" ]]; then
  cargo build --manifest-path "${ROOT_DIR}/spec_runner_cli/Cargo.toml" >&2
fi

exec "${BIN_PATH}" "$@"
