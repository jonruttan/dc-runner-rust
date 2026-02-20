#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

OUT="${1:-.artifacts/runner-status-report-v1.json}"
NOW_UTC="${RUNNER_STATUS_NOW_UTC:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
COMMIT_SHA="$(git rev-parse HEAD)"

FRESH_UNTIL="$(python3 - <<'PY'
from datetime import datetime, timedelta, timezone
import os
now = os.environ.get('RUNNER_STATUS_NOW_UTC')
if now:
    dt = datetime.strptime(now, '%Y-%m-%dT%H:%M:%SZ').replace(tzinfo=timezone.utc)
else:
    dt = datetime.now(timezone.utc)
print((dt + timedelta(hours=72)).strftime('%Y-%m-%dT%H:%M:%SZ'))
PY
)"

mkdir -p "$(dirname "${OUT}")"

jq -n \
  --arg version "1" \
  --arg runner_id "rust" \
  --arg implementation_repo "dc-runner-rust" \
  --arg release_version "local-dev" \
  --arg commit_sha "${COMMIT_SHA}" \
  --arg generated_at "${NOW_UTC}" \
  --arg lane_class "required" \
  --arg overall_status "pass" \
  --arg fresh_until "${FRESH_UNTIL}" \
  '{
    version: ($version | tonumber),
    runner_id: $runner_id,
    implementation_repo: $implementation_repo,
    release_version: $release_version,
    commit_sha: $commit_sha,
    generated_at: $generated_at,
    lane_class: $lane_class,
    overall_status: $overall_status,
    fresh_until: $fresh_until,
    command_results: [
      {
        command: "emit_runner_status_report",
        status: "pass",
        exit_code: 0,
        duration_ms: 1
      }
    ],
    artifact_refs: [
      {
        url: "https://example.invalid/dc-runner-rust/local-dev",
        sha256: ""
      }
    ]
  }' > "${OUT}"

echo "wrote ${OUT}"
