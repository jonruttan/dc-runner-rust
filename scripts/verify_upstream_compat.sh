#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SYNC_SCRIPT="${ROOT_DIR}/scripts/sync_data_contracts_specs.sh"
LOCK_FILE="${ROOT_DIR}/specs/upstream/data_contracts_lock_v1.yaml"
SNAP_ROOT="${ROOT_DIR}/specs/upstream/data-contracts"
RUNNER_BIN="${ROOT_DIR}/runner_adapter.sh"
STRICT=1
SOURCE=""

usage() {
  cat <<USAGE
Usage:
  scripts/verify_upstream_compat.sh [--strict] [--runner-bin <path>] [--source <path-or-url>]

Options:
  --strict            Enforce all compatibility checks (default).
  --runner-bin <path> Runner adapter path (default: ./runner_adapter.sh).
  --source <value>    Optional upstream source to verify lock tag/ref resolution.
USAGE
}

assert_file() {
  local rel="$1"
  local path="$SNAP_ROOT/$rel"
  if [[ ! -f "$path" ]]; then
    echo "ERROR: required compatibility file missing: $rel" >&2
    return 1
  fi
}

assert_runner_supports_subcommand() {
  local cmd="$1"
  if command -v rg >/dev/null 2>&1; then
    if rg -q "(^|[|[:space:]])${cmd}([|)[:space:]]|$)" "$RUNNER_BIN"; then
      return 0
    fi
  else
    if grep -Eq "(^|[|[:space:]])${cmd}([|)[:space:]]|$)" "$RUNNER_BIN"; then
      return 0
    fi
  fi
  {
    echo "ERROR: runner adapter missing required subcommand: ${cmd}" >&2
  }
  return 1
}

assert_exit_code() {
  local expected="$1"
  shift
  set +e
  "$@" >/tmp/dc_runner_verify.out 2>/tmp/dc_runner_verify.err
  local ec=$?
  set -e
  if [[ "$ec" -ne "$expected" ]]; then
    echo "ERROR: expected exit ${expected}, got ${ec} for command: $*" >&2
    sed -n '1,20p' /tmp/dc_runner_verify.err >&2 || true
    sed -n '1,20p' /tmp/dc_runner_verify.out >&2 || true
    return 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict)
      STRICT=1
      shift
      ;;
    --runner-bin)
      RUNNER_BIN="$2"
      shift 2
      ;;
    --source)
      SOURCE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

[[ -x "$RUNNER_BIN" ]] || { echo "ERROR: runner bin not executable: $RUNNER_BIN" >&2; exit 1; }
[[ -x "$SYNC_SCRIPT" ]] || { echo "ERROR: sync script not executable: $SYNC_SCRIPT" >&2; exit 1; }
[[ -f "$LOCK_FILE" ]] || { echo "ERROR: lock file missing: $LOCK_FILE" >&2; exit 1; }

if [[ -n "$SOURCE" ]]; then
  "$SYNC_SCRIPT" --check --source "$SOURCE"
else
  "$SYNC_SCRIPT" --check
fi

assert_file "specs/contract/index.md"
assert_file "specs/contract/policy_v1.yaml"
assert_file "specs/contract/traceability_v1.yaml"
assert_file "specs/schema/index.md"
assert_file "specs/schema/runner_certification_registry_v1.yaml"
assert_file "specs/schema/dc_runner_rust_lock_v1.yaml"
assert_file "specs/governance/index.md"
assert_file "specs/governance/check_sets_v1.yaml"
assert_file "specs/governance/cases/core/index.md"

# Parse required interface commands from upstream contract and ensure adapter surface includes them.
CONTRACT_FILE="$SNAP_ROOT/specs/contract/12_runner_interface.md"
[[ -f "$CONTRACT_FILE" ]] || { echo "ERROR: missing runner interface contract file" >&2; exit 1; }

REQUIRED_CMDS=()
while IFS= read -r cmd; do
  [[ -n "$cmd" ]] && REQUIRED_CMDS+=("$cmd")
done < <(
  awk '
    /Required subcommands:/ { in_block=1; next }
    in_block && /CI expectation:/ { exit }
    in_block && /^- `/ {
      gsub(/^- `/, "", $0)
      gsub(/`.*$/, "", $0)
      print $0
    }
  ' "$CONTRACT_FILE"
)

if [[ "${#REQUIRED_CMDS[@]}" -eq 0 ]]; then
  echo "ERROR: failed to extract required subcommands from contract" >&2
  exit 1
fi

for cmd in "${REQUIRED_CMDS[@]}"; do
  assert_runner_supports_subcommand "$cmd"
done

# Exit-code contract probes (representative success/failure/usage).
assert_exit_code 0 "$RUNNER_BIN" style-check
assert_exit_code 1 "$RUNNER_BIN" job-run --ref "#DOES_NOT_EXIST"
assert_exit_code 2 "$RUNNER_BIN" __unknown_subcommand__

# Required lane must not rely on direct python runner execution.
if command -v rg >/dev/null 2>&1; then
  if rg -n "python(3)?[[:space:]]+" "$RUNNER_BIN" >/dev/null 2>&1; then
    echo "ERROR: runner adapter appears to execute python directly" >&2
    exit 1
  fi
else
  if grep -En "python(3)?[[:space:]]+" "$RUNNER_BIN" >/dev/null 2>&1; then
    echo "ERROR: runner adapter appears to execute python directly" >&2
    exit 1
  fi
fi

if [[ "$STRICT" -eq 1 ]]; then
  :
fi

echo "OK: upstream compatibility verification passed"
