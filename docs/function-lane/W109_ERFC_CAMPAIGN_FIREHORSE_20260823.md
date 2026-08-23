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
association race peaked at 6127/15556, pins still 2–4 ULP. The leftover
is *where* Excel stores in a double-precision F-body around unsplit
`excel_exp(−z²)`. Firehorse can legally run `excel_exp` against the AMD
Excel banks. A 16-bit Horner-stage store mask × arithmetic × `z²` ×
ratio × small-cut is a finite, named region of that tree — large enough
for days, chunked so a 3-hour look-in still yields a map.

## CPU / time

- 12 of 16 vCPUs (75%, under the ~80% cap), `RAYON_NUM_THREADS=12`
- default `--max-hours 96` (4 days); stop early with a `STOP` file
- restart = same command; completed chunks are skipped

## Tree regions (explored in order)

| ID | Region | What is enumerated | Completeness |
|---|---|---|---|
| R0 | named public F | libm, production, NSWC, Cody-unsplit, SLATEC Cheb, MATH77 IEEE | exhaustive, small |
| R1 | NSWC P/Q Horner-stage store mask (16 bits) × `{X87Continuous, Native53}` × `{zz native, x87-DR}` × `{uv continuous, store, u*(1/v)}` × small-cut `{0.5, 1.0}` | 16-bit hypercube of “spill after Horner stage i” on the TR 92/425 P and Q (or AA/BB) polynomials | exhaustive for those axes |
| R2 | Cody C/D Horner-stage store mask (16 bits) × best two arith from R1 × zz native × uv continuous × small-cut 0.5 | same idea on SPECFUN CALERF C/D | exhaustive on that slice |
| R3 | implied-F scoreboard | `F = Q / excel_exp(−z²)` vs named F at the five pins and a mid-band sample | diagnostic, not a search |

If wall-clock remains after R2, R1 continues into extra small-cut `{0.46875, 0.75}` and `X87Pc53` as **R1b** (same 16-bit cube, extra axes). STATUS.md always says which cube and which mask prefix have been finished.

## Checkpoints

Directory: `smart-fuzzer/work/w109/erfc-campaign/`

| File | Role |
|---|---|
| `STATUS.md` | human snapshot, rewritten after every chunk (~256 configs) |
| `status.json` | same numbers, machine-readable |
| `checkpoint.json` | completed chunk ids; resume source |
| `leaders.jsonl` | every config that beats the running best exact or hits a pin |
| `campaign.log` | stdout/stderr from the tmux session |

`STOP` file in that directory: finish the current chunk, write checkpoint, exit.

## Monitor from this PC

```text
scp dna-firehorse:/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-campaign/STATUS.md .
ssh dna-firehorse -- tmux ls
ssh dna-firehorse -- tmux capture-pane -t oxfunc-erfc-campaign -p | tail
```

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
