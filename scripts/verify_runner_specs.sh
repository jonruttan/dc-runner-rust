#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

SOURCE=""

usage() {
  cat <<USAGE
Usage:
  scripts/verify_runner_specs.sh [--source <path-or-url>]
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
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

cmd=(cargo xtask runner-spec check)
[[ -n "$SOURCE" ]] && cmd+=(--source "$SOURCE")
"${cmd[@]}"

# Parity guard with python/php repos: block unexpected local executable specs.
if find "$ROOT_DIR/specs/impl/rust" -type f -name '*.spec.md' ! -path '*/jobs/*' 2>/dev/null | grep -q .; then
  echo "ERROR: unexpected local rust spec cases detected outside specs/impl/rust/jobs; runner-specific specs should be consumed from specs/upstream/dc-runner-spec" >&2
  exit 1
fi

echo "OK: runner-specific spec ingestion is consistent"
