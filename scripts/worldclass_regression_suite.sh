#!/usr/bin/env bash
set -euo pipefail

# Multi-query world-class regression suite for Fetchium.
# Produces compact metrics for standalone quality/speed and optional premium baselines.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT_DIR/target/debug/fetchium"
TMP_HOME="${TMP_HOME:-/tmp/fetchium_worldclass_suite_home}"
FETCH_TIMEOUT_SECS="${FETCH_TIMEOUT_SECS:-25}"

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

if ! command -v jq >/dev/null 2>&1; then
  echo "jq required" >&2
  exit 1
fi

queries=(
  # Developer / systems
  "best rust async runtime for high-throughput web api in 2025 benchmark"
  "how to implement oauth pkce in rust axum"
  "compare rust vs go microservices performance tradeoffs"
  # Current events / news
  "who won the latest super bowl and what was the score"
  "latest nvidia ai chip announcement 2026"
  # Finance / economics
  "latest fed interest rate decision summary and impact"
  "bitcoin etf inflow trend this month"
  # Health / medical info seeking
  "difference between flu and covid symptoms in adults"
  # Law / policy
  "new eu ai act requirements for startups"
  # Science / academic
  "recent breakthroughs in solid state battery research"
  # Consumer / product
  "best noise cancelling headphones for office calls 2026"
  # Local / practical
  "how to renew passport in california processing time"
  # Multilingual / cross-lingual
  "mejores frameworks web de rust en 2026"
  "quelles sont les dernieres avancees en IA generative"
  "最新の生成aiニュース 2026"
)

printf "query_idx\tstandalone_ms\tstandalone_count\tstandalone_domains\tserper_ms\tserper_count\texa_ms\texa_count\ttavily_ms\ttavily_count\n"

idx=0
for q in "${queries[@]}"; do
  idx=$((idx + 1))

  s_ms="timeout"; s_count="0"; s_domains="0"
  if timeout "${FETCH_TIMEOUT_SECS}"s \
    env HOME="$TMP_HOME" \
    TAVILY_API_KEY="" SERPER_API_KEY="" EXA_API_KEY="" FIRECRAWL_API_KEY="" \
    GEMINI_API_KEY="" GEMINI_API_KEYS="" OPENAI_API_KEY="" ANTHROPIC_API_KEY="" OPENROUTER_API_KEY="" \
    "$BIN" --no-cache -f json search "$q" -n 10 >/tmp/fetchium_suite_${idx}.json; then
    s_ms="$(jq -r '.meta.duration_ms' /tmp/fetchium_suite_${idx}.json)"
    s_count="$(jq -r '.items|length' /tmp/fetchium_suite_${idx}.json)"
    s_domains="$(jq -r '[.items[].url|split("/")[2]]|unique|length' /tmp/fetchium_suite_${idx}.json)"
  fi

  serper_ms="na"; serper_count="na"
  exa_ms="na"; exa_count="na"
  tavily_ms="na"; tavily_count="na"

  if [[ -n "${SERPER_API_KEY:-}" ]]; then
    t0="$(date +%s%3N)"
    r="$(curl -sS -m 20 https://google.serper.dev/search \
      -H "X-API-KEY: $SERPER_API_KEY" \
      -H 'Content-Type: application/json' \
      -d "{\"q\":\"$q\",\"num\":10}")" || true
    t1="$(date +%s%3N)"
    if echo "$r" | jq -e '.organic|type=="array"' >/dev/null 2>&1; then
      serper_ms="$((t1-t0))"
      serper_count="$(echo "$r" | jq -r '.organic|length')"
    fi
  fi

  if [[ -n "${EXA_API_KEY:-}" ]]; then
    t0="$(date +%s%3N)"
    r="$(curl -sS -m 20 https://api.exa.ai/search \
      -H "x-api-key: $EXA_API_KEY" \
      -H 'Content-Type: application/json' \
      -d "{\"query\":\"$q\",\"type\":\"auto\",\"numResults\":10}")" || true
    t1="$(date +%s%3N)"
    if echo "$r" | jq -e '.results|type=="array"' >/dev/null 2>&1; then
      exa_ms="$((t1-t0))"
      exa_count="$(echo "$r" | jq -r '.results|length')"
    fi
  fi

  if [[ -n "${TAVILY_API_KEY:-}" ]]; then
    t0="$(date +%s%3N)"
    r="$(curl -sS -m 20 https://api.tavily.com/search \
      -H 'Content-Type: application/json' \
      -d "{\"api_key\":\"$TAVILY_API_KEY\",\"query\":\"$q\",\"search_depth\":\"advanced\",\"max_results\":10}")" || true
    t1="$(date +%s%3N)"
    if echo "$r" | jq -e '.results|type=="array"' >/dev/null 2>&1; then
      tavily_ms="$((t1-t0))"
      tavily_count="$(echo "$r" | jq -r '.results|length')"
    fi
  fi

  printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
    "$idx" "$s_ms" "$s_count" "$s_domains" \
    "$serper_ms" "$serper_count" \
    "$exa_ms" "$exa_count" \
    "$tavily_ms" "$tavily_count"
done
