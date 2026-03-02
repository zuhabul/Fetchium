#!/usr/bin/env bash
set -euo pipefail

# YouTube-focused regression suite:
# - Metadata completeness rate
# - Transcript success rate
# - Median latency (metadata/transcript/watch)
# - Ranking consistency across runs
#
# Usage:
#   scripts/youtube_regression_suite.sh
#   BIN=./target/debug/fetchium scripts/youtube_regression_suite.sh
#   FIXTURES=tests/fixtures/youtube_golden_videos.txt scripts/youtube_regression_suite.sh

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${BIN:-$ROOT_DIR/target/debug/fetchium}"
FIXTURES="${FIXTURES:-$ROOT_DIR/tests/fixtures/youtube_golden_videos.txt}"
QUERY="${QUERY:-rust async tutorial}"
TOP_K="${TOP_K:-5}"
TIMEOUT_SECS="${TIMEOUT_SECS:-45}"

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build -p fetchium-cli >/dev/null)
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi

if [[ ! -f "$FIXTURES" ]]; then
  echo "Fixture file not found: $FIXTURES" >&2
  exit 1
fi

mapfile -t urls < <(grep -v '^\s*#' "$FIXTURES" | sed '/^\s*$/d')
if [[ "${#urls[@]}" -eq 0 ]]; then
  echo "No fixture URLs found in $FIXTURES" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

median_from_file() {
  local file="$1"
  local n
  n="$(wc -l < "$file" | tr -d ' ')"
  if [[ "$n" -eq 0 ]]; then
    echo "0"
    return
  fi
  sort -n "$file" > "${file}.sorted"
  local mid=$((n / 2))
  if (( n % 2 == 1 )); then
    sed -n "$((mid + 1))p" "${file}.sorted"
  else
    local a b
    a="$(sed -n "${mid}p" "${file}.sorted")"
    b="$(sed -n "$((mid + 1))p" "${file}.sorted")"
    echo $(((a + b) / 2))
  fi
}

meta_completeness_total=0
meta_field_total=0
transcript_success=0

meta_lat_file="$tmp_dir/meta_latency_ms.txt"
tr_lat_file="$tmp_dir/transcript_latency_ms.txt"
watch_lat_file="$tmp_dir/watch_latency_ms.txt"

echo "Running YouTube regression suite on ${#urls[@]} golden fixtures..."

for i in "${!urls[@]}"; do
  idx=$((i + 1))
  url="${urls[$i]}"

  meta_json="$tmp_dir/meta_${idx}.json"
  t0="$(date +%s%3N)"
  if timeout "${TIMEOUT_SECS}"s "$BIN" youtube video "$url" -f json >"$meta_json" 2>"$tmp_dir/meta_${idx}.err"; then
    t1="$(date +%s%3N)"
    echo "$((t1 - t0))" >> "$meta_lat_file"

    complete_fields="$(jq -r '
      [
        (.video_id // "") != "",
        (.title // "") != "",
        (.duration_secs // 0) > 0,
        (.view_count // 0) > 0,
        (.published // "") != "",
        (.channel.name // "") != "",
        ((.thumbnail_url // "") | tostring) != ""
      ] | map(select(. == true)) | length
    ' "$meta_json")"
    total_fields=7
    meta_completeness_total=$((meta_completeness_total + complete_fields))
    meta_field_total=$((meta_field_total + total_fields))
  fi

  transcript_json="$tmp_dir/transcript_${idx}.json"
  t0="$(date +%s%3N)"
  if timeout "${TIMEOUT_SECS}"s "$BIN" youtube transcript "$url" -f json >"$transcript_json" 2>"$tmp_dir/tr_${idx}.err"; then
    t1="$(date +%s%3N)"
    echo "$((t1 - t0))" >> "$tr_lat_file"
    ok="$(jq -r '((.word_count // 0) > 0 and (.quality_score // 0) > 0.25)' "$transcript_json")"
    if [[ "$ok" == "true" ]]; then
      transcript_success=$((transcript_success + 1))
    fi
  fi

  t0="$(date +%s%3N)"
  timeout "${TIMEOUT_SECS}"s "$BIN" youtube watch "$url" --highlights 3 >/dev/null \
    2>"$tmp_dir/watch_${idx}.err" || true
  t1="$(date +%s%3N)"
  echo "$((t1 - t0))" >> "$watch_lat_file"
done

search1="$tmp_dir/search1.json"
search2="$tmp_dir/search2.json"
timeout "${TIMEOUT_SECS}"s "$BIN" youtube search "$QUERY" -n "$TOP_K" -f json >"$search1" 2>"$tmp_dir/s1.err" || true
timeout "${TIMEOUT_SECS}"s "$BIN" youtube search "$QUERY" -n "$TOP_K" -f json >"$search2" 2>"$tmp_dir/s2.err" || true

ids1="$tmp_dir/ids1.txt"
ids2="$tmp_dir/ids2.txt"
jq -r ".rankings[0:${TOP_K}][]?.video_id" "$search1" | sort -u > "$ids1" || true
jq -r ".rankings[0:${TOP_K}][]?.video_id" "$search2" | sort -u > "$ids2" || true

intersection="$(comm -12 "$ids1" "$ids2" | wc -l | tr -d ' ')"
union_count="$(cat "$ids1" "$ids2" | sed '/^\s*$/d' | sort -u | wc -l | tr -d ' ')"
if [[ "$union_count" -gt 0 ]]; then
  ranking_consistency_pct=$((intersection * 100 / union_count))
else
  ranking_consistency_pct=0
fi

if [[ "$meta_field_total" -gt 0 ]]; then
  metadata_completeness_pct=$((meta_completeness_total * 100 / meta_field_total))
else
  metadata_completeness_pct=0
fi

fixture_count="${#urls[@]}"
transcript_success_pct=$((transcript_success * 100 / fixture_count))

meta_median_ms="$(median_from_file "$meta_lat_file")"
tr_median_ms="$(median_from_file "$tr_lat_file")"
watch_median_ms="$(median_from_file "$watch_lat_file")"

echo
echo "## YouTube Regression Report"
echo
echo "| Metric | Value |"
echo "|---|---:|"
echo "| Golden fixtures | ${fixture_count} |"
echo "| Metadata completeness | ${metadata_completeness_pct}% |"
echo "| Transcript success | ${transcript_success_pct}% |"
echo "| Median metadata latency | ${meta_median_ms} ms |"
echo "| Median transcript latency | ${tr_median_ms} ms |"
echo "| Median watch latency | ${watch_median_ms} ms |"
echo "| Ranking consistency (top-${TOP_K} Jaccard) | ${ranking_consistency_pct}% |"
echo
echo "Query used for consistency: ${QUERY}"
