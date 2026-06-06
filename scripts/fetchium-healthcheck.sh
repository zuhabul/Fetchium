#!/usr/bin/env bash
set -euo pipefail

HEALTH_URL="${FETCHIUM_HEALTH_URL:-http://127.0.0.1:3050/v1/health}"
SEARCH_URL="${FETCHIUM_SEARCH_URL:-http://127.0.0.1:3050/v1/search}"
HEALTH_TIMEOUT_SECS="${FETCHIUM_HEALTH_TIMEOUT_SECS:-5}"
SEARCH_TIMEOUT_SECS="${FETCHIUM_SEARCH_TIMEOUT_SECS:-20}"
HEALTH_CONFIRM_TIMEOUT_SECS="${FETCHIUM_HEALTHCHECK_HEALTH_CONFIRM_TIMEOUT_SECS:-20}"
SEARCH_CONFIRM_TIMEOUT_SECS="${FETCHIUM_HEALTHCHECK_CONFIRM_TIMEOUT_SECS:-30}"
SERVICE_NAME="${FETCHIUM_SYSTEMD_SERVICE:-fetchium-api}"
SEARCH_QUERY="${FETCHIUM_HEALTHCHECK_SEARCH_QUERY:-polyandry}"
FALLBACK_ENV_FILE="${FETCHIUM_HEALTHCHECK_ENV_FILE:-/home/echo/projects/ogroshor/packages/api-gateway/.env}"
STATE_FILE="${FETCHIUM_HEALTHCHECK_STATE_FILE:-/home/echo/.fetchium/healthcheck-search-failures}"
LEGACY_FAILURE_THRESHOLD="${FETCHIUM_HEALTHCHECK_FAILURE_THRESHOLD:-3}"
HEALTH_FAILURE_THRESHOLD="${FETCHIUM_HEALTHCHECK_HEALTH_FAILURE_THRESHOLD:-${LEGACY_FAILURE_THRESHOLD}}"
SEARCH_FAILURE_THRESHOLD="${FETCHIUM_HEALTHCHECK_SEARCH_FAILURE_THRESHOLD:-${LEGACY_FAILURE_THRESHOLD}}"

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
  check_health_with_timeout "${HEALTH_TIMEOUT_SECS}"
}

check_health_with_timeout() {
  local timeout_secs="$1"
  curl -fsS --max-time "${timeout_secs}" "${HEALTH_URL}" >/dev/null
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
  local key="$1"
  local value
  if [[ -f "${STATE_FILE}" ]]; then
    value="$(grep -E "^${key}=" "${STATE_FILE}" 2>/dev/null | tail -n 1 | cut -d= -f2 || true)"
    if [[ "${value}" =~ ^[0-9]+$ ]]; then
      echo "${value}"
      return
    fi
  fi
  echo 0
}

write_failures() {
  local health_failures="$1"
  local search_failures="$2"
  mkdir -p "$(dirname "${STATE_FILE}")"
  cat > "${STATE_FILE}" <<EOF
health_failures=${health_failures}
search_failures=${search_failures}
EOF
}

reset_failures() {
  rm -f "${STATE_FILE}"
}

health_failures="$(read_failures health_failures)"
search_failures="$(read_failures search_failures)"

if check_health; then
  health_failures=0

  if check_search; then
    reset_failures
    exit 0
  fi

  search_failures="$((search_failures + 1))"
  write_failures "${health_failures}" "${search_failures}"
  echo "fetchium-healthcheck: search probe failed while /v1/health is ok (consecutive=${search_failures}/${SEARCH_FAILURE_THRESHOLD})" >&2
  if [[ "${search_failures}" -lt "${SEARCH_FAILURE_THRESHOLD}" ]]; then
    exit 0
  fi

  # Before restarting a healthy process, require one slower confirmation probe.
  # This avoids watchdog churn during temporary mixed-load latency spikes.
  if check_search_with_timeout "${SEARCH_CONFIRM_TIMEOUT_SECS}"; then
    health_failures=0
    search_failures=0
    write_failures "${health_failures}" "${search_failures}"
    exit 0
  fi

  echo "fetchium-healthcheck: search confirmation failed; restarting ${SERVICE_NAME}" >&2
else
  health_failures="$((health_failures + 1))"
  write_failures "${health_failures}" "${search_failures}"
  echo "fetchium-healthcheck: /v1/health probe failed (consecutive=${health_failures}/${HEALTH_FAILURE_THRESHOLD})" >&2
  if [[ "${health_failures}" -lt "${HEALTH_FAILURE_THRESHOLD}" ]]; then
    exit 0
  fi

  # Before restarting the service on health failures, require one slower
  # confirmation probe. This avoids restarts when the process is briefly
  # overloaded but still alive.
  if check_health_with_timeout "${HEALTH_CONFIRM_TIMEOUT_SECS}"; then
    health_failures=0
    search_failures=0
    write_failures "${health_failures}" "${search_failures}"
    reset_failures
    exit 0
  fi

  echo "fetchium-healthcheck: health confirmation failed; restarting ${SERVICE_NAME}" >&2
fi

restart_service
reset_failures
