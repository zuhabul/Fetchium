#!/bin/bash
set -o allexport; source .env.benchmark; set +o allexport

QUERIES=(
  "how to implement oauth2 in rust using axum"
  "what are the symptoms of long covid in adults 2025"
  "latest developments in solid state batteries"
  "frieren beyond journey's end ending explained"
)

for q in "${QUERIES[@]}"; do
  echo "Testing query: $q"
  cargo run -p fetchium-cli -- --config test_config.toml --no-cache -f json search "$q" -n 5 > "result_${q// /_}.json" 2> "log_${q// /_}.txt"
  jq -r '.items[] | "  [\(.score)] \(.title) - \(.url)"' "result_${q// /_}.json" || echo "Failed to parse JSON for $q"
  echo "Duration: $(jq -r '.meta.duration_ms' "result_${q// /_}.json" || echo 'N/A') ms"
  echo "---"
done
