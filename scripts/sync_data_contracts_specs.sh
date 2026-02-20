#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

MODE=""
TAG=""
SOURCE=""
ALLOW_REF=0

usage() {
  cat <<USAGE
Usage:
  scripts/sync_data_contracts_specs.sh --check [--source <path-or-url>]
  scripts/sync_data_contracts_specs.sh --tag <tag-or-ref> [--source <path-or-url>] [--write] [--allow-ref]

Options:
  --check           Verify lock + manifest + snapshot consistency.
  --tag <value>     Upstream tag or ref to pin (required for write mode).
  --source <value>  Optional upstream source for ref-resolution checks.
  --write           Compatibility no-op when --tag is provided.
  --allow-ref       Allow non-tag refs for sync mode.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check)
      MODE="check"
      shift
      ;;
    --tag)
      [[ $# -ge 2 ]] || { echo "ERROR: --tag requires a value" >&2; usage >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    --source)
      [[ $# -ge 2 ]] || { echo "ERROR: --source requires a value" >&2; usage >&2; exit 2; }
      SOURCE="$2"
      shift 2
      ;;
    --allow-ref)
      ALLOW_REF=1
      shift
      ;;
    --write)
      shift
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

if [[ -n "$TAG" && -z "$MODE" ]]; then
  MODE="write"
fi

if [[ "$MODE" == "check" ]]; then
  cmd=(cargo xtask spec check)
  [[ -n "$SOURCE" ]] && cmd+=(--source "$SOURCE")
  "${cmd[@]}"
  exit $?
fi

if [[ "$MODE" == "write" ]]; then
  [[ -n "$TAG" ]] || { echo "ERROR: --tag is required for write mode" >&2; usage >&2; exit 2; }
  cmd=(cargo xtask spec sync --tag "$TAG")
  [[ -n "$SOURCE" ]] && cmd+=(--source "$SOURCE")
  [[ "$ALLOW_REF" -eq 1 ]] && cmd+=(--allow-ref)
  "${cmd[@]}"
  exit $?
fi

echo "ERROR: choose --check or --tag <value>" >&2
usage >&2
exit 2
