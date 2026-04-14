#!/usr/bin/env bash
set -euo pipefail

HEALTH_URL="${FETCHIUM_HEALTH_URL:-http://127.0.0.1:3050/v1/health}"
SEARCH_URL="${FETCHIUM_SEARCH_URL:-http://127.0.0.1:3050/v1/search}"
HEALTH_TIMEOUT_SECS="${FETCHIUM_HEALTH_TIMEOUT_SECS:-5}"
SEARCH_TIMEOUT_SECS="${FETCHIUM_SEARCH_TIMEOUT_SECS:-20}"
SEARCH_CONFIRM_TIMEOUT_SECS="${FETCHIUM_HEALTHCHECK_CONFIRM_TIMEOUT_SECS:-30}"
SERVICE_NAME="${FETCHIUM_SYSTEMD_SERVICE:-fetchium-api}"
SEARCH_QUERY="${FETCHIUM_HEALTHCHECK_SEARCH_QUERY:-polyandry}"
FALLBACK_ENV_FILE="${FETCHIUM_HEALTHCHECK_ENV_FILE:-/home/echo/projects/ogroshor/packages/api-gateway/.env}"
STATE_FILE="${FETCHIUM_HEALTHCHECK_STATE_FILE:-/home/echo/.fetchium/healthcheck-search-failures}"
FAILURE_THRESHOLD="${FETCHIUM_HEALTHCHECK_FAILURE_THRESHOLD:-3}"

load_api_key() {
  if [[ -n "${FETCHIUM_API_KEY:-}" ]]; then
    return 0
  fi

  if [[ -f "${FALLBACK_ENV_FILE}" ]]; then
    # shellcheck disable=SC1090
    set -a
    source "${FALLBACK_ENV_FILE}"
    set +a
    export FETCHIUM_API_KEY="${FETCHIUM_API_KEY:-${HSX_API_KEY:-}}"
  fi
}

check_health() {
  curl -fsS --max-time "${HEALTH_TIMEOUT_SECS}" "${HEALTH_URL}" >/dev/null
}

check_search_with_timeout() {
  local timeout_secs="$1"
  load_api_key
  if [[ -z "${FETCHIUM_API_KEY:-}" ]]; then
    return 0
  fi

  local escaped_query
  escaped_query="${SEARCH_QUERY//\"/\\\"}"
  curl -fsS --max-time "${timeout_secs}" \
    -H "Authorization: Bearer ${FETCHIUM_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "{\"query\":\"${escaped_query}\",\"max_sources\":1}" \
    "${SEARCH_URL}" >/dev/null
}

check_search() {
  check_search_with_timeout "${SEARCH_TIMEOUT_SECS}"
}

restart_service() {
  if timeout "${FETCHIUM_RESTART_TIMEOUT_SECS:-20}" systemctl restart "${SERVICE_NAME}"; then
    return 0
  fi

  systemctl kill -s SIGKILL "${SERVICE_NAME}" || true
  sleep 1
  systemctl reset-failed "${SERVICE_NAME}" || true
  systemctl start "${SERVICE_NAME}"
}

read_failures() {
  if [[ -f "${STATE_FILE}" ]]; then
    cat "${STATE_FILE}" 2>/dev/null || echo 0
    return
  fi
  echo 0
}

write_failures() {
  mkdir -p "$(dirname "${STATE_FILE}")"
  printf '%s\n' "$1" > "${STATE_FILE}"
}

reset_failures() {
  rm -f "${STATE_FILE}"
}

if check_health && check_search; then
  reset_failures
  exit 0
fi

if check_health; then
  failures="$(read_failures)"
  failures="$((failures + 1))"
  write_failures "${failures}"
  echo "fetchium-healthcheck: search probe failed while /v1/health is ok (consecutive=${failures}/${FAILURE_THRESHOLD})" >&2
  if [[ "${failures}" -lt "${FAILURE_THRESHOLD}" ]]; then
    exit 0
  fi

  # Before restarting a healthy process, require one slower confirmation probe.
  # This avoids watchdog churn during temporary mixed-load latency spikes.
  if check_search_with_timeout "${SEARCH_CONFIRM_TIMEOUT_SECS}"; then
    reset_failures
    exit 0
  fi
fi

restart_service
reset_failures
