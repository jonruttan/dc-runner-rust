#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

STRICT=1
SOURCE=""
RUNNER_BIN="${ROOT_DIR}/runner_adapter.sh"

usage() {
  cat <<USAGE
Usage:
  scripts/verify_upstream_compat.sh [--strict] [--runner-bin <path>] [--source <path-or-url>]

Options:
  --strict            Enforce all compatibility checks (default and required mode).
  --runner-bin <path> Runner adapter path to validate exists and is executable.
  --source <value>    Optional upstream source for lock tag/ref verification.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict)
      STRICT=1
      shift
      ;;
    --runner-bin)
      [[ $# -ge 2 ]] || { echo "ERROR: --runner-bin requires a value" >&2; usage >&2; exit 2; }
      RUNNER_BIN="$2"
      shift 2
      ;;
    --source)
      [[ $# -ge 2 ]] || { echo "ERROR: --source requires a value" >&2; usage >&2; exit 2; }
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

if [[ "$STRICT" -ne 1 ]]; then
  echo "ERROR: only --strict mode is supported" >&2
  exit 2
fi

if [[ ! -x "$RUNNER_BIN" ]]; then
  echo "ERROR: runner bin not executable: $RUNNER_BIN" >&2
  exit 1
fi

spec_cmd=(cargo xtask spec check)
compat_cmd=(cargo xtask compat check)
if [[ -n "$SOURCE" ]]; then
  spec_cmd+=(--source "$SOURCE")
  compat_cmd+=(--source "$SOURCE")
fi

"${spec_cmd[@]}"
"${compat_cmd[@]}"

echo "OK: upstream compatibility verification passed"
