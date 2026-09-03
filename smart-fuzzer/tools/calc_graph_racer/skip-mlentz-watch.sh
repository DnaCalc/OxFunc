#!/usr/bin/env bash
# After lentz/as714/n12b2 finishes, skip the duplicate 2^24 mlentz/as714/n12b2
# and resume the leftover n16/n21 cubes on 12 threads. Idempotent.
set -euo pipefail
OUT=/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-lentz-campaign
RACER=/data/projects/DnaCalc/OxFunc/smart-fuzzer/tools/calc_graph_racer
ROOT=/data/projects/DnaCalc/OxFunc
DIR="$ROOT/smart-fuzzer/work/w109/G3-01-dist"
BIN=/tmp/cargo-target-lentz/release/campaign_erfc_lentz
LOG="$OUT/skip-mlentz-watch.log"
ONLY="lentz/gaut/n16,mlentz/gaut/n16,lentz/as714/n16,mlentz/as714/n16,lentz/gaut/n21,lentz/as714/n21"
mkdir -p "$OUT"

log() { echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $*" | tee -a "$LOG"; }

region_of() {
  python3 - <<'PY'
import json
p = "/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-lentz-campaign/status.json"
try:
    d = json.load(open(p))
    print(d.get("region", ""))
except Exception:
    print("")
PY
}

alive() { pgrep -f '/campaign_erfc_lentz ' >/dev/null 2>&1; }

leftover_region() {
  case "$1" in
    lentz/gaut/n16|mlentz/gaut/n16|lentz/as714/n16|mlentz/as714/n16|lentz/gaut/n21|lentz/as714/n21)
      return 0 ;;
    *) return 1 ;;
  esac
}

log "watch start pid=$$"

while true; do
  r=$(region_of)
  running=0
  if alive; then running=1; fi
  log "poll region=$r running=$running"
  if leftover_region "$r"; then
    log "already on leftover cube $r — no skip"
    echo DONE
    exit 0
  fi
  if [[ "$r" == "mlentz/as714/n12b2" ]]; then
    log "mlentz/as714/n12b2 started — STOP and skip"
    break
  fi
  if [[ "$r" == "exit-regions" || "$r" == "exit-timeout" || "$r" == "exit-stop" ]]; then
    log "campaign exited ($r) — start leftover cubes"
    break
  fi
  if [[ "$running" -eq 0 ]]; then
    log "process gone, region=$r — start leftover cubes"
    break
  fi
  sleep 20
done

touch "$OUT/STOP"
log "STOP touched"
for _ in $(seq 1 180); do
  if ! alive; then
    log "process exited after STOP"
    break
  fi
  sleep 2
done
if alive; then
  log "FAILED process still alive after STOP wait"
  echo FAILED
  exit 1
fi
rm -f "$OUT/STOP"

if [[ ! -x "$BIN" ]]; then
  log "FAILED missing binary $BIN"
  echo FAILED
  exit 1
fi

tmux kill-session -t oxfunc-erfc-lentz 2>/dev/null || true
sleep 1
# shellcheck disable=SC2086
tmux new-session -d -s oxfunc-erfc-lentz bash -lc "rm -f '$OUT/STOP'; exec '$BIN' --dir '$DIR' --out '$OUT' --threads 12 --max-hours 96 --only '$ONLY' >>'$OUT/campaign.log' 2>&1"
sleep 6
r2=$(region_of)
if alive; then
  log "SKIP_RESTART only=$ONLY region=$r2"
  echo DONE
  exit 0
fi
log "FAILED restart did not spawn campaign_erfc"
echo FAILED
exit 1
