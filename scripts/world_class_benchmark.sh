#!/usr/bin/env bash
set -euo pipefail

# World-class benchmark harness for Fetchium.
#
# Runs:
# 1) Standalone search quality/speed
# 2) URL/text summarize latency + output quality proxy
# 3) YouTube summarize latency + output quality proxy
# 4) Optional direct premium API comparison (if keys are exported)
#
# Usage:
#   scripts/world_class_benchmark.sh
#   QUERY="rust async runtime benchmark 2025" scripts/world_class_benchmark.sh
#   QUERY="..." YT_URL="https://youtu.be/dQw4w9WgXcQ" scripts/world_class_benchmark.sh

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT_DIR/target/debug/fetchium"

QUERY="${QUERY:-best rust async runtime for high-throughput web api in 2025 benchmark}"
URL_INPUT="${URL_INPUT:-https://blog.rust-lang.org/}"
YT_URL="${YT_URL:-https://www.youtube.com/watch?v=dQw4w9WgXcQ}"
MAX_RESULTS="${MAX_RESULTS:-10}"

TMP_HOME="${TMP_HOME:-/tmp/fetchium_worldclass_home}"
mkdir -p "$TMP_HOME/.fetchium"
cat >"$TMP_HOME/.fetchium/config.toml" <<'EOF'
[ai]
ollama_host = "http://localhost"
default_model = ""
max_tokens = 4096

[ai.providers]
fallback_chain = []
EOF

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build -p fetchium-cli >/dev/null)
fi

must_have() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing dependency: $1" >&2
    exit 1
  }
}

must_have jq
must_have awk

run_fetchium_search() {
  echo "== Fetchium Standalone Search =="
  HOME="$TMP_HOME" \
    TAVILY_API_KEY="" SERPER_API_KEY="" EXA_API_KEY="" FIRECRAWL_API_KEY="" \
    GEMINI_API_KEY="" GEMINI_API_KEYS="" OPENAI_API_KEY="" ANTHROPIC_API_KEY="" OPENROUTER_API_KEY="" \
    "$BIN" --no-cache -f json search "$QUERY" -n "$MAX_RESULTS" >/tmp/fetchium_wc_search.json

  jq -r '[.meta.duration_ms, (.items|length), ([.items[].url|split("/")[2]]|unique|length)] | @tsv' \
    /tmp/fetchium_wc_search.json \
    | awk '{print "duration_ms=" $1 " count=" $2 " unique_domains=" $3}'
}

run_summarize_url() {
  echo "== Fetchium URL Summarize =="
  HOME="$TMP_HOME" "$BIN" summarize "$URL_INPUT" >/tmp/fetchium_wc_summary_url.txt 2>/tmp/fetchium_wc_summary_url.err || true
  local secs
  secs="$(awk '/Completed in/{print $(NF-0)}' /tmp/fetchium_wc_summary_url.err | tail -1)"
  local chars
  chars="$(wc -c </tmp/fetchium_wc_summary_url.txt | tr -d ' ')"
  echo "duration_s=${secs:-n/a} output_chars=$chars"
}

run_summarize_youtube() {
  echo "== Fetchium YouTube Summarize =="
  HOME="$TMP_HOME" "$BIN" summarize "$YT_URL" >/tmp/fetchium_wc_summary_yt.txt 2>/tmp/fetchium_wc_summary_yt.err || true
  local secs
  secs="$(awk '/Completed in/{print $(NF-0)}' /tmp/fetchium_wc_summary_yt.err | tail -1)"
  local chars
  chars="$(wc -c </tmp/fetchium_wc_summary_yt.txt | tr -d ' ')"
  echo "duration_s=${secs:-n/a} output_chars=$chars"
}

run_premium_compare_optional() {
  echo "== Premium Direct Compare (Optional) =="
  if [[ -z "${TAVILY_API_KEY:-}" && -z "${SERPER_API_KEY:-}" && -z "${EXA_API_KEY:-}" && -z "${FIRECRAWL_API_KEY:-}" ]]; then
    echo "skipped (no premium env keys exported)"
    return
  fi

  if [[ -n "${SERPER_API_KEY:-}" ]]; then
    local t0 t1 res
    t0="$(date +%s%3N)"
    res="$(curl -sS -m 20 https://google.serper.dev/search \
      -H "X-API-KEY: $SERPER_API_KEY" \
      -H 'Content-Type: application/json' \
      -d "{\"q\":\"$QUERY\",\"num\":10}")" || true
    t1="$(date +%s%3N)"
    if echo "$res" | jq -e '.organic|type=="array"' >/dev/null 2>&1; then
      echo "serper duration_ms=$((t1-t0)) count=$(echo "$res" | jq '.organic|length')"
    fi
  fi

  if [[ -n "${EXA_API_KEY:-}" ]]; then
    local t0 t1 res
    t0="$(date +%s%3N)"
    res="$(curl -sS -m 20 https://api.exa.ai/search \
      -H "x-api-key: $EXA_API_KEY" \
      -H 'Content-Type: application/json' \
      -d "{\"query\":\"$QUERY\",\"type\":\"auto\",\"numResults\":10}")" || true
    t1="$(date +%s%3N)"
    if echo "$res" | jq -e '.results|type=="array"' >/dev/null 2>&1; then
      echo "exa duration_ms=$((t1-t0)) count=$(echo "$res" | jq '.results|length')"
    fi
  fi

  if [[ -n "${TAVILY_API_KEY:-}" ]]; then
    local t0 t1 res
    t0="$(date +%s%3N)"
    res="$(curl -sS -m 20 https://api.tavily.com/search \
      -H 'Content-Type: application/json' \
      -d "{\"api_key\":\"$TAVILY_API_KEY\",\"query\":\"$QUERY\",\"search_depth\":\"advanced\",\"max_results\":10}")" || true
    t1="$(date +%s%3N)"
    if echo "$res" | jq -e '.results|type=="array"' >/dev/null 2>&1; then
      echo "tavily duration_ms=$((t1-t0)) count=$(echo "$res" | jq '.results|length')"
    fi
  fi

  if [[ -n "${FIRECRAWL_API_KEY:-}" ]]; then
    local t0 t1 res
    t0="$(date +%s%3N)"
    res="$(curl -sS -m 20 https://api.firecrawl.dev/v1/search \
      -H "Authorization: Bearer $FIRECRAWL_API_KEY" \
      -H 'Content-Type: application/json' \
      -d "{\"query\":\"$QUERY\",\"limit\":5}")" || true
    t1="$(date +%s%3N)"
    if echo "$res" | jq -e '.data|type=="array"' >/dev/null 2>&1; then
      echo "firecrawl duration_ms=$((t1-t0)) count=$(echo "$res" | jq '.data|length')"
    fi
  fi
}

echo "Query: $QUERY"
echo "URL:   $URL_INPUT"
echo "YT:    $YT_URL"
echo

run_fetchium_search
run_summarize_url
run_summarize_youtube
run_premium_compare_optional

