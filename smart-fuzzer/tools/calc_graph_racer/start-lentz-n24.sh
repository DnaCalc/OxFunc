#!/usr/bin/env bash
# Pack leftover firehorse with Lentz n24 C-spill after n21 exit-regions.
set -euo pipefail
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi
ROOT=/data/projects/DnaCalc/OxFunc
RACER=$ROOT/smart-fuzzer/tools/calc_graph_racer
OUT=$ROOT/smart-fuzzer/work/w109/erfc-lentz-campaign
DIR=$ROOT/smart-fuzzer/work/w109/G3-01-dist
BIN=/tmp/cargo-target-lentz/release/campaign_erfc_lentz
ONLY=lentz/gaut/n24,lentz/as714/n24

cd "$ROOT"
git pull --ff-only origin main
export CARGO_TARGET_DIR=/tmp/cargo-target-lentz
cd "$RACER"
cargo build --release --offline --bin campaign_erfc_lentz
test -x "$BIN"
rm -f "$OUT/STOP"
tmux kill-session -t oxfunc-erfc-lentz 2>/dev/null || true
tmux new-session -d -s oxfunc-erfc-lentz bash -lc "exec '$BIN' --dir '$DIR' --out '$OUT' --threads 12 --max-hours 120 --only '$ONLY' >>'$OUT/campaign.log' 2>&1"
sleep 6
pgrep -af /campaign_erfc_lentz || { echo "FAILED spawn"; exit 1; }
python3 -c "import json; d=json.load(open('$OUT/status.json')); print(d.get('region'), d.get('chunk'), d.get('best_tail_exact'))"
echo STARTED only=$ONLY
