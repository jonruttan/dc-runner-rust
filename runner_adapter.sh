#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${ROOT_DIR}"

RUST_CLI_MANIFEST="${ROOT_DIR}/spec_runner_cli/Cargo.toml"
RUST_CLI_TARGET=""
RUST_CLI_BIN=""
RUST_PREFERRED_TARGET=""
HOST_BIN_LOCAL="${ROOT_DIR}/spec_runner_cli/target/debug/spec_runner_cli"
HOST_BIN_ROOT="${ROOT_DIR}/target/debug/spec_runner_cli"

is_debug_enabled() {
  local val="${SPEC_RUNNER_DEBUG:-}"
  [[ "${val}" == "1" || "${val}" == "true" || "${val}" == "yes" ]]
}

debug_level() {
  local lvl="${SPEC_RUNNER_DEBUG_LEVEL:-0}"
  if [[ "${lvl}" =~ ^[0-9]+$ ]]; then
    echo "${lvl}"
  elif is_debug_enabled; then
    echo 1
  else
    echo 0
  fi
}

debug_log() {
  if [[ "$(debug_level)" -ge 1 ]]; then
    echo "[runner_adapter debug] $*" >&2
  fi
}

debug_log_at() {
  local level="$1"
  shift
  if [[ "$(debug_level)" -ge "${level}" ]]; then
    echo "[runner_adapter debug:${level}] $*" >&2
  fi
}

# Prefer native Apple Silicon binaries when available to avoid Rosetta/runtime hangs.
if [[ "$(uname -s)" == "Darwin" && "$(uname -m)" == "arm64" ]]; then
  ARM_TARGET="aarch64-apple-darwin"
  ARM_BIN_LOCAL="${ROOT_DIR}/spec_runner_cli/target/${ARM_TARGET}/debug/spec_runner_cli"
  ARM_BIN_ROOT="${ROOT_DIR}/target/${ARM_TARGET}/debug/spec_runner_cli"
  RUST_CLI_TARGET="${ARM_TARGET}"
  RUST_PREFERRED_TARGET="${ARM_TARGET}"
  if [[ -x "${ARM_BIN_ROOT}" ]]; then
    RUST_CLI_BIN="${ARM_BIN_ROOT}"
  elif [[ -x "${ARM_BIN_LOCAL}" ]]; then
    RUST_CLI_BIN="${ARM_BIN_LOCAL}"
  else
    RUST_CLI_BIN="${ARM_BIN_ROOT}"
  fi
fi

# Default host-target binary resolution for non-ARM or when target-specific binary is unset.
if [[ -z "${RUST_CLI_BIN}" ]]; then
  if [[ -x "${HOST_BIN_ROOT}" ]]; then
    RUST_CLI_BIN="${HOST_BIN_ROOT}"
  elif [[ -x "${HOST_BIN_LOCAL}" ]]; then
    RUST_CLI_BIN="${HOST_BIN_LOCAL}"
  else
    RUST_CLI_BIN="${HOST_BIN_ROOT}"
  fi
fi

debug_log "root=${ROOT_DIR}"
debug_log_at 2 "manifest=${RUST_CLI_MANIFEST}"
debug_log "target=${RUST_CLI_TARGET:-default-host}"
debug_log_at 2 "bin=${RUST_CLI_BIN}"

has_rust_target_installed() {
  local target="$1"
  [[ -n "${target}" ]] || return 1
  rustup target list --installed 2>/dev/null | grep -qx "${target}"
}

strict_rust_target_mode() {
  local raw="${SPEC_RUNNER_RUST_TARGET_STRICT:-0}"
  [[ "${raw}" == "1" || "${raw}" == "true" || "${raw}" == "yes" ]]
}

resolve_rust_target_mode() {
  if [[ -z "${RUST_CLI_TARGET}" ]]; then
    return 0
  fi
  if has_rust_target_installed "${RUST_CLI_TARGET}"; then
    return 0
  fi
  if strict_rust_target_mode; then
    echo "ERROR: missing Rust target '${RUST_CLI_TARGET}'. Install with: rustup target add ${RUST_CLI_TARGET}" >&2
    return 2
  fi
  echo "[runner_adapter] preferred target missing; using host target (${RUST_CLI_TARGET} unavailable)." >&2
  RUST_CLI_TARGET=""
  if [[ -x "${HOST_BIN_ROOT}" ]]; then
    RUST_CLI_BIN="${HOST_BIN_ROOT}"
  elif [[ -x "${HOST_BIN_LOCAL}" ]]; then
    RUST_CLI_BIN="${HOST_BIN_LOCAL}"
  else
    RUST_CLI_BIN="${HOST_BIN_ROOT}"
  fi
}

ensure_rust_cli_bin() {
  if [[ -x "${RUST_CLI_BIN}" ]]; then
    return 0
  fi
  if ! resolve_rust_target_mode; then
    return 2
  fi
  if [[ -n "${RUST_CLI_TARGET}" ]]; then
    debug_log "building rust cli target=${RUST_CLI_TARGET}"
    cargo build --quiet --manifest-path "${RUST_CLI_MANIFEST}" --target "${RUST_CLI_TARGET}" || return 1
    RUST_CLI_BIN="${ROOT_DIR}/target/${RUST_CLI_TARGET}/debug/spec_runner_cli"
  else
    debug_log "building rust cli host-target"
    cargo build --quiet --manifest-path "${RUST_CLI_MANIFEST}" || return 1
    if [[ -x "${HOST_BIN_ROOT}" ]]; then
      RUST_CLI_BIN="${HOST_BIN_ROOT}"
    elif [[ -x "${HOST_BIN_LOCAL}" ]]; then
      RUST_CLI_BIN="${HOST_BIN_LOCAL}"
    else
      echo "ERROR: rust cli build completed but binary not found at expected host paths." >&2
      return 1
    fi
  fi
  if [[ ! -x "${RUST_CLI_BIN}" ]]; then
    echo "ERROR: rust cli binary missing or not executable: ${RUST_CLI_BIN}" >&2
    return 1
  fi
}

run_rust_subcommand() {
  local cmd="$1"
  shift
  debug_log "run_rust_subcommand cmd=${cmd} args=[$*]"
  if ! ensure_rust_cli_bin; then
    return 2
  fi
  debug_log "using binary ${RUST_CLI_BIN}"
  "${RUST_CLI_BIN}" "${cmd}" "$@"
}

run_rust_subcommand_with_retry() {
  local cmd="$1"
  local attempts="$2"
  shift 2

  if [[ ! "${attempts}" =~ ^[0-9]+$ ]] || [[ "${attempts}" -lt 1 ]]; then
    echo "ERROR: invalid retry attempt count '${attempts}' for ${cmd}; expected integer >= 1" >&2
    return 2
  fi

  local attempt=1
  local exit_code=0
  while (( attempt <= attempts )); do
    if run_rust_subcommand "${cmd}" "$@"; then
      return 0
    fi
    exit_code=$?
    if [[ "${exit_code}" -ge 128 ]] && (( attempt < attempts )); then
      echo "WARN: transient ${cmd} termination (exit ${exit_code}) on attempt ${attempt}/${attempts}; retrying." >&2
      attempt=$((attempt + 1))
      sleep 1
      continue
    fi
    break
  done

  if [[ "${exit_code}" -ge 128 ]]; then
    echo "ERROR: ${cmd} terminated by signal-like exit code ${exit_code}. Set SPEC_RUNNER_DEBUG=1 for more context and retry." >&2
  fi
  return "${exit_code}"
}

exec_rust_subcommand() {
  local cmd="$1"
  shift
  debug_log "exec_rust_subcommand cmd=${cmd} args=[$*]"
  if ! ensure_rust_cli_bin; then
    exit 2
  fi
  debug_log "exec binary ${RUST_CLI_BIN}"
  exec "${RUST_CLI_BIN}" "${cmd}" "$@"
}

run_with_timeout() {
  local timeout_seconds="$1"
  local label="$2"
  local env_var_name="$3"
  shift 3

  if [[ ! "${timeout_seconds}" =~ ^[0-9]+$ ]] || [[ "${timeout_seconds}" -le 0 ]]; then
    echo "ERROR: invalid timeout '${timeout_seconds}' for ${env_var_name}; expected positive integer seconds" >&2
    return 2
  fi

  local timeout_flag
  timeout_flag="$(mktemp -t spec_runner_timeout.XXXXXX)"
  rm -f "${timeout_flag}"

  "$@" &
  local cmd_pid=$!
  (
    sleep "${timeout_seconds}"
    if kill -0 "${cmd_pid}" 2>/dev/null; then
      : > "${timeout_flag}"
      kill -TERM "${cmd_pid}" 2>/dev/null || true
      sleep 5
      kill -KILL "${cmd_pid}" 2>/dev/null || true
    fi
  ) &
  local watchdog_pid=$!

  local cmd_status=0
  if ! wait "${cmd_pid}"; then
    cmd_status=$?
  fi
  kill "${watchdog_pid}" 2>/dev/null || true
  wait "${watchdog_pid}" 2>/dev/null || true

  if [[ -f "${timeout_flag}" ]]; then
    rm -f "${timeout_flag}"
    echo "ERROR: '${label}' timed out after ${timeout_seconds}s. Override with ${env_var_name}=<seconds>." >&2
    return 124
  fi

  rm -f "${timeout_flag}"
  return "${cmd_status}"
}

run_with_timeout_env() {
  local env_var_name="$1"
  local default_seconds="$2"
  local label="$3"
  shift 3

  local timeout_seconds="${!env_var_name:-${default_seconds}}"
  run_with_timeout "${timeout_seconds}" "${label}" "${env_var_name}" "$@"
}

subcommand="${1:-}"
while [[ $# -gt 0 ]]; do
  case "${1:-}" in
    --verbose|-v)
      export SPEC_RUNNER_DEBUG=1
      export SPEC_RUNNER_DEBUG_LEVEL=1
      shift
      ;;
    -vv)
      export SPEC_RUNNER_DEBUG=1
      export SPEC_RUNNER_DEBUG_LEVEL=2
      shift
      ;;
    -vvv)
      export SPEC_RUNNER_DEBUG=1
      export SPEC_RUNNER_DEBUG_LEVEL=3
      shift
      ;;
    --profile-level)
      export SPEC_RUNNER_PROFILE_LEVEL="${2:-}"
      shift 2
      ;;
    --profile-out)
      export SPEC_RUNNER_PROFILE_OUT="${2:-}"
      shift 2
      ;;
    --profile-summary-out)
      export SPEC_RUNNER_PROFILE_SUMMARY_OUT="${2:-}"
      shift 2
      ;;
    --profile-heartbeat-ms)
      export SPEC_RUNNER_PROFILE_HEARTBEAT_MS="${2:-}"
      shift 2
      ;;
    --profile-stall-threshold-ms)
      export SPEC_RUNNER_PROFILE_STALL_THRESHOLD_MS="${2:-}"
      shift 2
      ;;
    --liveness-level)
      export SPEC_RUNNER_LIVENESS_LEVEL="${2:-}"
      shift 2
      ;;
    --liveness-stall-ms)
      export SPEC_RUNNER_LIVENESS_STALL_MS="${2:-}"
      shift 2
      ;;
    --liveness-min-events)
      export SPEC_RUNNER_LIVENESS_MIN_EVENTS="${2:-}"
      shift 2
      ;;
    --liveness-hard-cap-ms)
      export SPEC_RUNNER_LIVENESS_HARD_CAP_MS="${2:-}"
      shift 2
      ;;
    --liveness-kill-grace-ms)
      export SPEC_RUNNER_LIVENESS_KILL_GRACE_MS="${2:-}"
      shift 2
      ;;
    --fail-fast)
      export SPEC_RUNNER_FAIL_FAST=1
      shift
      ;;
    --continue-on-fail)
      export SPEC_RUNNER_FAIL_FAST=0
      shift
      ;;
    --profile-on-fail)
      export SPEC_RUNNER_PROFILE_ON_FAIL="${2:-}"
      shift 2
      ;;
    *)
      break
      ;;
  esac
done
subcommand="${1:-}"
if [[ -z "${subcommand}" ]]; then
  echo "ERROR: missing runner adapter subcommand" >&2
  exit 2
fi
shift
debug_log "subcommand=${subcommand} forwarded=[$*]"

case "${subcommand}" in
  spec-eval|spec-ref|validate-report|job-run|critical-gate|governance-broad-native|style-check|spec-lang-lint|spec-lang-format|schema-registry-check|schema-registry-build|schema-docs-check|schema-docs-build|lint|typecheck|compilecheck|conformance-purpose-json|conformance-purpose-md|spec-portability-json|spec-portability-md|spec-lang-adoption-json|spec-lang-adoption-md|runner-independence-json|runner-independence-md|python-dependency-json|python-dependency-md|docs-operability-json|docs-operability-md|contract-assertions-json|contract-assertions-md|objective-scorecard-json|objective-scorecard-md|spec-lang-stdlib-json|spec-lang-stdlib-md|ci-gate-summary|ci-cleanroom|perf-smoke|docs-generate|docs-generate-check|docs-build|docs-build-check|docs-lint|docs-graph|conformance-parity|test-core|test-full|runner-certify)
    exec_rust_subcommand "${subcommand}" "$@"
    ;;
esac

case "${subcommand}" in
  governance)
    run_rust_subcommand_with_retry "${subcommand}" "${SPEC_RUNNER_RETRY_GOVERNANCE_ATTEMPTS:-2}" "$@"
    ;;
  governance-heavy)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_GOVERNANCE_HEAVY_SECONDS \
      180 \
      governance-heavy \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  normalize-check)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_NORMALIZE_SECONDS \
      120 \
      normalize-check \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  normalize-fix)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_NORMALIZE_SECONDS \
      120 \
      normalize-fix \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  objective-scorecard-json)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  objective-scorecard-md)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  spec-lang-stdlib-json)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  spec-lang-stdlib-md)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  ci-gate-summary)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  ci-cleanroom)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  perf-smoke)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  docs-generate)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_DOCS_SECONDS \
      180 \
      docs-generate \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  docs-generate-check)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_DOCS_SECONDS \
      180 \
      docs-generate-check \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  docs-build)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_DOCS_SECONDS \
      180 \
      docs-build \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  docs-build-check)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_DOCS_SECONDS \
      180 \
      docs-build-check \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  docs-lint)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  docs-graph)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  conformance-parity)
    run_with_timeout_env \
      SPEC_RUNNER_TIMEOUT_CONFORMANCE_PARITY_SECONDS \
      240 \
      conformance-parity \
      run_rust_subcommand "${subcommand}" "$@"
    ;;
  test-core)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  test-full)
    if [[ -x "${RUST_CLI_BIN}" ]]; then
      exec "${RUST_CLI_BIN}" "${subcommand}" "$@"
    fi
    exec cargo run --quiet --manifest-path "${RUST_CLI_MANIFEST}" -- "${subcommand}" "$@"
    ;;
  *)
    echo "ERROR: unsupported runner adapter subcommand: ${subcommand}" >&2
    exit 2
    ;;
esac
