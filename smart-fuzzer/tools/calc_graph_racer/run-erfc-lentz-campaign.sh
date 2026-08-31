#!/usr/bin/env bash
# Firehorse complementary Lentz-F campaign. 4/16 vCPUs, 96h default, resume-safe.
set -euo pipefail
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi
RACER="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$RACER/../../.." && pwd)"
OUT="$ROOT/smart-fuzzer/work/w109/erfc-lentz-campaign"
DIR="$ROOT/smart-fuzzer/work/w109/G3-01-dist"
mkdir -p "$OUT"
export RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-4}"
cd "$RACER"
ONLY_ARGS=()
if [[ -n "${ONLY:-}" ]]; then
  ONLY_ARGS+=(--only "$ONLY")
fi
exec cargo run --release --offline --bin campaign_erfc_lentz -- \
  --dir "$DIR" \
  --out "$OUT" \
  --threads "$RAYON_NUM_THREADS" \
  --max-hours "${MAX_HOURS:-96}" \
  "${ONLY_ARGS[@]}" \
  >>"$OUT/campaign.log" 2>&1
