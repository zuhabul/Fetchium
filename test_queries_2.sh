#!/bin/bash
set -o allexport; source .env.benchmark; set +o allexport

QUERIES=(
  "rust vs go performance benchmark 2025"
  "best local llm for coding on macbook pro m3 max"
)

for q in "${QUERIES[@]}"; do
  echo "Testing query: $q"
  cargo run -p fetchium-cli -- --config test_config.toml --no-cache -f json search "$q" -n 5 > "result_${q// /_}.json" 2> "log_${q// /_}.txt"
  jq -r '.items[] | "  [\(.score)] \(.title) - \(.url)"' "result_${q// /_}.json" || echo "Failed to parse JSON for $q"
  echo "Duration: $(jq -r '.meta.duration_ms' "result_${q// /_}.json" || echo 'N/A') ms"
  echo "---"
done
