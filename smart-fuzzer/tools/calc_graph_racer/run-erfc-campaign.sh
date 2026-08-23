#!/usr/bin/env bash
# Firehorse ERFC-body campaign. 12/16 vCPUs, 96h default, resume-safe.
set -euo pipefail
RACER="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$RACER/../../.." && pwd)"
OUT="$ROOT/smart-fuzzer/work/w109/erfc-campaign"
DIR="$ROOT/smart-fuzzer/work/w109/G3-01-dist"
mkdir -p "$OUT"
export RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-12}"
cd "$RACER"
if [[ ! -x ./target/release/campaign_erfc_body ]]; then
  cargo build --release --bin campaign_erfc_body
fi
exec ./target/release/campaign_erfc_body \
  --dir "$DIR" \
  --out "$OUT" \
  --threads "$RAYON_NUM_THREADS" \
  --max-hours "${MAX_HOURS:-96}" \
  >>"$OUT/campaign.log" 2>&1
