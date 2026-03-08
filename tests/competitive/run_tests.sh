#!/bin/bash
# Competitive Analysis: Fetchium vs Tavily vs Serper vs Exa vs Firecrawl
set -euo pipefail

FETCHIUM_KEY="fetchium_95255db17cadef51a2ff90dc19362042565de7ded675f6af07719edfff41132e"
FETCHIUM_BASE="http://127.0.0.1:3050"
TAVILY_KEY="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"
SERPER_KEY="e88feb18b71987dde4301947f050604b22bb9363"
EXA_KEY="eb4d14ea-602c-4a03-bf41-1c496f8bfb00"
FIRECRAWL_KEY="fc-c3ed9e586f2e438f96653e98d3ddcac4"

OUT_DIR="/home/echo/projects/Fetchium/tests/competitive/results"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

declare -a QUERIES=(
  "Latest advances in quantum computing 2026"
  "How to implement a B-tree in Rust"
  "Impact of AI on climate change research"
  "Best practices for Kubernetes security"
  "History of the Byzantine Empire"
)
QUERY_LABELS=("quantum" "btree" "ai_climate" "k8s_security" "byzantine")

echo "=== COMPETITIVE ANALYSIS: $(date) ==="
echo ""

for i in "${!QUERIES[@]}"; do
  Q="${QUERIES[$i]}"
  L="${QUERY_LABELS[$i]}"
  echo "--- Query $((i+1)): $Q ---"

  # FETCHIUM SEARCH
  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "$FETCHIUM_BASE/v1/search" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $FETCHIUM_KEY" \
    -d "{\"query\":\"$Q\",\"max_sources\":5,\"tier\":\"summary\",\"token_budget\":2000}" \
    -o "$OUT_DIR/fetchium_search_${L}.json" -w '%{http_code}' 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Fetchium search: $((END-START))ms (HTTP $HTTP_CODE)"

  # FETCHIUM RESEARCH
  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "$FETCHIUM_BASE/v1/research" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $FETCHIUM_KEY" \
    -d "{\"query\":\"$Q\",\"max_sources\":5,\"token_budget\":2000,\"depth\":\"standard\"}" \
    -o "$OUT_DIR/fetchium_research_${L}.json" -w '%{http_code}' --max-time 60 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Fetchium research: $((END-START))ms (HTTP $HTTP_CODE)"

  # TAVILY
  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "https://api.tavily.com/search" \
    -H "Content-Type: application/json" \
    -d "{\"api_key\":\"$TAVILY_KEY\",\"query\":\"$Q\",\"max_results\":5,\"search_depth\":\"advanced\",\"include_answer\":true,\"include_raw_content\":false}" \
    -o "$OUT_DIR/tavily_${L}.json" -w '%{http_code}' 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Tavily: $((END-START))ms (HTTP $HTTP_CODE)"

  # SERPER
  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "https://google.serper.dev/search" \
    -H "Content-Type: application/json" \
    -H "X-API-KEY: $SERPER_KEY" \
    -d "{\"q\":\"$Q\",\"num\":5}" \
    -o "$OUT_DIR/serper_${L}.json" -w '%{http_code}' 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Serper: $((END-START))ms (HTTP $HTTP_CODE)"

  # EXA
  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "https://api.exa.ai/search" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $EXA_KEY" \
    -d "{\"query\":\"$Q\",\"numResults\":5,\"useAutoprompt\":true,\"type\":\"auto\",\"contents\":{\"text\":{\"maxCharacters\":500}}}" \
    -o "$OUT_DIR/exa_${L}.json" -w '%{http_code}' 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Exa: $((END-START))ms (HTTP $HTTP_CODE)"

  # FIRECRAWL
  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "https://api.firecrawl.dev/v1/search" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $FIRECRAWL_KEY" \
    -d "{\"query\":\"$Q\",\"limit\":5}" \
    -o "$OUT_DIR/firecrawl_${L}.json" -w '%{http_code}' 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Firecrawl: $((END-START))ms (HTTP $HTTP_CODE)"

  echo ""
  sleep 1
done

echo "=== SCRAPE/FETCH TEST ==="
TEST_URLS=("https://www.rust-lang.org/" "https://en.wikipedia.org/wiki/Quantum_computing")
URL_LABELS=("rust_lang" "wiki_quantum")

for i in "${!TEST_URLS[@]}"; do
  URL="${TEST_URLS[$i]}"
  UL="${URL_LABELS[$i]}"
  echo "--- Scrape: $URL ---"

  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "$FETCHIUM_BASE/v1/fetch" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $FETCHIUM_KEY" \
    -d "{\"url\":\"$URL\",\"format\":\"markdown\",\"token_budget\":1000}" \
    -o "$OUT_DIR/fetchium_fetch_${UL}.json" -w '%{http_code}' --max-time 30 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Fetchium fetch: $((END-START))ms (HTTP $HTTP_CODE)"

  START=$(date +%s%3N)
  HTTP_CODE=$(curl -sX POST "https://api.firecrawl.dev/v1/scrape" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $FIRECRAWL_KEY" \
    -d "{\"url\":\"$URL\",\"formats\":[\"markdown\"]}" \
    -o "$OUT_DIR/firecrawl_scrape_${UL}.json" -w '%{http_code}' 2>/dev/null)
  END=$(date +%s%3N)
  echo "  Firecrawl scrape: $((END-START))ms (HTTP $HTTP_CODE)"
  echo ""
  sleep 1
done

echo "=== FETCHIUM CLI SEARCH TEST ==="
export PATH="$HOME/.cargo/bin:$PATH"
for i in "${!QUERIES[@]}"; do
  Q="${QUERIES[$i]}"
  L="${QUERY_LABELS[$i]}"
  echo "--- CLI: $Q ---"
  START=$(date +%s%3N)
  fetchium search "$Q" --max-sources 5 --no-ai > "$OUT_DIR/fetchium_cli_${L}.txt" 2>&1 || true
  END=$(date +%s%3N)
  echo "  CLI search (no-ai): $((END-START))ms"
done

echo ""
echo "=== ALL TESTS COMPLETE ==="
