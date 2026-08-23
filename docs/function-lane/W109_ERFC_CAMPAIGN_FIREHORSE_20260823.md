# ERFC-body firehorse campaign

Date: 2026-08-23
Host: `dna-firehorse` (16-vCPU AMD EPYC KVM, x87 last-bit matched this
Windows AMD box 3157/3157)
Lane: W109 ERFC.PRECISE body — store-site × arithmetic × staging tree

## Status axes

- `execution_state`: `in_progress` (long run)
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`: ERFC body; this campaign does not land kernels

Heldouts are not named and are not copied. Discovery banks only.

## Why this campaign

Named public F graphs (NSWC, Cody, SLATEC Chebyshev) and a 5-axis
association race peaked at 6127/15556 (mid 3332/7741), pins still 2–4 ULP.
The leftover is *where* Excel stores in a double-precision F-body around
unsplit `excel_exp(−z²)`. Firehorse can legally run `excel_exp` against
the AMD Excel banks.

A local smoke showed ~900 configs/s on 8 threads, so a 16-bit cube
finishes in minutes and would idle the machine. The long run is a
**26-bit** P/Q/R + t-formation store mask on x87-continuous Horner
(~67 million configs per axis, about half a day to a day per axis at
12 threads). Native and every-op/all-stage spill ignore extra mask
bits, so they are scored once (R1base). Ratio includes `u*(1/v)`.
NSWC mid_cut 1.5 (the assoc-race winner) is first in the queue.

## CPU / time

- 12 of 16 vCPUs (75%, under the ~80% cap), `RAYON_NUM_THREADS=12`
- default `--max-hours 96` (4 days); stop early with a `STOP` file
- restart = same command; completed chunks are skipped
- if every named cube finishes early, the process **exits** rather than
  re-rolling the already-enumerated 16-bit space

## Tree regions (explored in order)

| ID | Region | What is enumerated | Completeness |
|---|---|---|---|
| R0 | named F + implied-F pins | libm, production, NSWC DERFC0 native, Cody-unsplit, assoc-race bar cfg, F = Q/exp(−z²) vs NSWC/Cody at five pins | exhaustive, minutes |
| R0c | NSWC P/Q/R ±1 ULP | one-coefficient next_up/next_down on the published decimals (Cody last-bit already noise) | exhaustive, minutes |
| R1base | mask-insensitive arith | Native / EveryOp53 / Pc53 / all-stage-spill × zz × uv × mid_cut {4, 1.5} | exhaustive, minutes |
| R1m | NSWC 26-bit P/Q/R + t-formation store mask × zz {x87-DR first, native} × uv {store first, continuous, recip}, mid_cut=1.5 | PQR below 1.5, AA/BB above. First axis is the assoc-race bar. ~67e6 configs/axis | first, then the other five R1m axes |
| R1 | same 26-bit mask, mid_cut=4 | PQR on the whole mid band `[0.5,4)` | after R1m, if wall-clock remains |
| R4 | NSWC AA/BB 19-bit (AA 0–7, BB 8–18) × zz × uv, mid_cut=1.5 | PQR continuous below 1.5 | after R1 |
| R2 | Cody C/D 16-bit × zz × uv | SPECFUN CALERF, X87Cont | after R4 |
| R1p | one Pc53 26-bit PQR+t cube | zz-DR + store uv + mid_cut 1.5, if wall-clock remains | last |

`REGION_MAP.md` always says which cube and which mask prefix have been
finished. Scoring is the 7,741-row mid band `[0.5, 4)` plus pin hits.
All-band R0 is recorded separately. The assoc-race mid bar (3332/7741)
is printed on every STATUS snapshot.

## Checkpoints

Directory: `smart-fuzzer/work/w109/erfc-campaign/`

| File | Role |
|---|---|
| `STATUS.md` | human snapshot, rewritten after every chunk (~4096 configs) |
| `R0.md` `R0c.md` `R1base.md` | phase reports that STATUS extra would otherwise overwrite |
| `REGION_MAP.md` | named tree regions and per-axis next-mask |
| `status.json` | same numbers, machine-readable |
| `checkpoint.json` | completed chunk ids; resume source |
| `leaders.jsonl` | every config that beats running best exact/mid/pins |
| `MONITOR.md` | scp/ssh recipes (copied into the out dir) |
| `campaign.log` | stdout/stderr from the tmux session |
| `HIT_ALL_MID` | written only if some config is exact on all mid rows |
| `HIT_PIN` | written if a pin z goes exact |

`STOP` file in that directory: finish the current chunk, write checkpoint, exit.

## Monitor from this PC

```text
scp dna-firehorse:/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-campaign/STATUS.md .
scp dna-firehorse:/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-campaign/REGION_MAP.md .
ssh dna-firehorse -- tmux ls
ssh dna-firehorse -- tmux capture-pane -t oxfunc-erfc-campaign -p | tail
```

A few-hour look-in is useful: R0/R0c/R1base finish in minutes and then
STATUS shows the first R1 mask prefixes. A multi-day look-in is the
full named cubes plus leaders.jsonl.

## Session

tmux session `oxfunc-erfc-campaign` (detached). Reattach:
`ssh -t dna-firehorse tmux attach -t oxfunc-erfc-campaign`
Detach: `Ctrl-b d`. Survives SSH drop. Reboot would kill it.

## Binary

`smart-fuzzer/tools/calc_graph_racer` bin `campaign_erfc_body`

```text
cargo run --release --bin campaign_erfc_body -- \
  --dir ../../work/w109/G3-01-dist \
  --out ../../work/w109/erfc-campaign \
  --threads 12 --max-hours 96
```
