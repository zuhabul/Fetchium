#!/usr/bin/env bash
set -euo pipefail

# Compare Supadata YouTube APIs with Fetchium YouTube functionality.
#
# Required:
#   SUPADATA_API_KEY=sd_... scripts/supadata_comparison.sh
#
# Optional:
#   VIDEO_IDS="dQw4w9WgXcQ,3fumBcKC6RE" SUPADATA_API_KEY=... scripts/supadata_comparison.sh
#   SUPA_CHANNEL_ID="UCuAXFkgsw1L7xaCfnd5JJOw" SUPA_PLAYLIST_ID="PL..." SUPADATA_API_KEY=... scripts/supadata_comparison.sh

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT_DIR/target/debug/fetchium"

if [[ -z "${SUPADATA_API_KEY:-}" ]]; then
  echo "SUPADATA_API_KEY is required" >&2
  exit 1
fi

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
must_have curl

VIDEO_IDS="${VIDEO_IDS:-dQw4w9WgXcQ,3fumBcKC6RE}"
IFS=',' read -r -a VIDEO_ID_ARR <<<"$VIDEO_IDS"

echo "== Transcript Comparison =="
printf "video_id\tsupadata_ms\tsupadata_lang\tsupadata_segments\tsupadata_text_chars\tfetchium_ms\tfetchium_lang\tfetchium_segments\tfetchium_text_chars\tfetchium_quality\n"

for vid in "${VIDEO_ID_ARR[@]}"; do
  vid="$(echo "$vid" | xargs)"
  url="https://www.youtube.com/watch?v=${vid}"

  t0="$(date +%s%3N)"
  sres="$(curl -sS -m 20 "https://api.supadata.ai/v1/youtube/transcript?url=${url}" -H "x-api-key: ${SUPADATA_API_KEY}" || true)"
  t1="$(date +%s%3N)"
  supa_ms="$((t1 - t0))"
  supa_lang="$(echo "$sres" | jq -r '.lang // "na"' 2>/dev/null || echo "na")"
  supa_segments="$(echo "$sres" | jq -r '(.content // []) | length' 2>/dev/null || echo "0")"
  supa_text_chars="$(echo "$sres" | jq -r '((.content // []) | map(.text // "") | join(" ") | length)' 2>/dev/null || echo "0")"

  t0="$(date +%s%3N)"
  fres="$("$BIN" youtube transcript "$url" -f json 2>/tmp/fetchium_supadata_cmp.err || true)"
  t1="$(date +%s%3N)"
  fet_ms="$((t1 - t0))"
  fet_lang="$(echo "$fres" | jq -r '.language // "na"' 2>/dev/null || echo "na")"
  fet_segments="$(echo "$fres" | jq -r '(.entries // []) | length' 2>/dev/null || echo "0")"
  fet_text_chars="$(echo "$fres" | jq -r '(.full_text // "" | length)' 2>/dev/null || echo "0")"
  fet_quality="$(echo "$fres" | jq -r '.quality_score // 0' 2>/dev/null || echo "0")"

  printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
    "$vid" "$supa_ms" "$supa_lang" "$supa_segments" "$supa_text_chars" \
    "$fet_ms" "$fet_lang" "$fet_segments" "$fet_text_chars" "$fet_quality"
done

echo
echo "== Video Metadata Comparison =="
printf "video_id\tsupadata_ms\tsupadata_title_len\tfetchium_ms\tfetchium_title_len\n"
for vid in "${VIDEO_ID_ARR[@]}"; do
  vid="$(echo "$vid" | xargs)"
  t0="$(date +%s%3N)"
  sres="$(curl -sS -m 20 "https://api.supadata.ai/v1/youtube/video?id=${vid}" -H "x-api-key: ${SUPADATA_API_KEY}" || true)"
  t1="$(date +%s%3N)"
  supa_ms="$((t1 - t0))"
  supa_tlen="$(echo "$sres" | jq -r '(.title // "" | length)' 2>/dev/null || echo "0")"

  t0="$(date +%s%3N)"
  fres="$("$BIN" youtube video "$vid" -f json 2>/tmp/fetchium_supadata_cmp.err || true)"
  t1="$(date +%s%3N)"
  fet_ms="$((t1 - t0))"
  fet_tlen="$(echo "$fres" | jq -r '(.title // "" | length)' 2>/dev/null || echo "0")"

  printf "%s\t%s\t%s\t%s\t%s\n" "$vid" "$supa_ms" "$supa_tlen" "$fet_ms" "$fet_tlen"
done

echo
echo "== Channel/Playlist Parity Check =="
if command -v yt-dlp >/dev/null 2>&1; then
  echo "yt-dlp: installed"
  if [[ -n "${SUPA_CHANNEL_ID:-}" ]]; then
    s_count="$(curl -sS -m 20 "https://api.supadata.ai/v1/youtube/channel/videos?id=${SUPA_CHANNEL_ID}" -H "x-api-key: ${SUPADATA_API_KEY}" | jq -r '(.videoIds // []) | length')"
    f_count="$("$BIN" youtube channel "${SUPA_CHANNEL_ID}" --videos -n 50 -f json | jq -r '(.videos // []) | length')"
    echo "channel_videos_count supadata=${s_count} fetchium=${f_count}"
  else
    echo "channel_videos_count skipped (set SUPA_CHANNEL_ID)"
  fi
  if [[ -n "${SUPA_PLAYLIST_ID:-}" ]]; then
    s_count="$(curl -sS -m 20 "https://api.supadata.ai/v1/youtube/playlist/videos?id=${SUPA_PLAYLIST_ID}" -H "x-api-key: ${SUPADATA_API_KEY}" | jq -r '(.videoIds // []) | length')"
    f_count="$("$BIN" youtube playlist "${SUPA_PLAYLIST_ID}" -n 100 -f json | jq -r '(.video_ids // []) | length')"
    echo "playlist_videos_count supadata=${s_count} fetchium=${f_count}"
  else
    echo "playlist_videos_count skipped (set SUPA_PLAYLIST_ID)"
  fi
else
  echo "yt-dlp: not installed (channel/playlist parity requires yt-dlp)"
fi
