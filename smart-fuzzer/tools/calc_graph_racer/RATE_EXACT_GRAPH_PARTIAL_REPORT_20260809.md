# W109 G6-05 RATE exact-graph discovery — scope-partial handoff

Date: 2026-08-09
Reference host: Excel 16.0 build 20228, 64-bit, workbook compatibility 2
Capture plumbing: `cell_value2_bulk`, `NoCache`, runner `w109-bulk-batch-v2`

## Status

- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- production changes: none
- shared doctrine/catalog/state/bead changes: none
- heldout answers: not captured or inspected

This lane produced a reproducible discovery corpus and bounded exact-graph
eliminations, but no exact coherent RATE survivor.  It therefore makes no
function-wide implementation or closure claim.

## Discovery design and capture integrity

The RATE batch contains 256 deterministic cancellation-tuned rows.  Each row
sets the requested future value so that all 13,824 pre-frozen balance/FD/update
graphs have a finite first residual below `1e-7`.  The discovery and heldout
inputs came from disjoint deterministic streams.  The generator is immutable:
a rerun verified every artifact byte-for-byte and refuses a differing overwrite.

Only the discovery batch was captured:

- function/count: `RATE`, 256/256
- unique nonempty IDs: 256
- ordered ID plus exact argument-bit mismatches: 0
- result kinds: 256 numeric, 0 error, 0 other
- Excel process count: pre `0`, bounded teardown post `0`
- provenance: Excel `16.0` build `20228`, `64-bit`, CV `2`,
  `cell_value2_bulk`, `no_cache`, cache hits/misses `0/0`

The answer-blind FV companion reuses each frozen RATE row at its exact guess and
at every distinct candidate `x+h`.  All four frozen h/store combinations collapse
to one `x+h` bit pattern on all 256 rows, giving exactly 512 FV calls.  Its capture
also passed exact ordered ID/argument alignment:

- function/count: `FV`, 512/512
- unique nonempty IDs: 512
- ordered ID plus exact argument-bit mismatches: 0
- result kinds: 512 numeric, 0 error, 0 other
- Excel process count: pre `0`, bounded teardown post `0`
- provenance: Excel `16.0` build `20228`, `64-bit`, CV `2`,
  `cell_value2_bulk`, `no_cache`, cache hits/misses `0/0`

## Exact score summary

| Discovery race | Rows | Candidate graphs | Exact survivors | Best exact | ≤1 ULP | ≤4 ULP | ≤16 ULP |
|---|---:|---:|---:|---:|---:|---:|---:|
| Frozen balance + FD + update v2 | 256 | 13,824 | 0 | 2 | 2 | 3 | 5 |
| Public-FV objective substitution | 256 | 1,536 | 0 | 2 | 2 | 4 | 7 |
| Public FV spill isolation | 512 | 30,720 | 0 | 502 | 502 | 502 | 502 |
| Raw-power inline FV helper + outer spill | 256 | 7,864,320 | 0 | 2 | 2 | 4 | 8 |

The 512-row public-FV race extends historical `fit_fv_stores.rs` across four
factor forms, fifteen coherent annuity associations, and nine spill sites.  The
leader is the worksheet integer/fractional POWER kernel with
`pmt*(type_factor*q)` and material intermediate stores.  It is exact on 502/512.
The ten misses are cancellation-sensitive rows (mostly `rate = 2^-18`, plus two
ordinary-rate `nper=3` rows and one very-small-rate row) where all association
families share the same material miss.  Thus even the worksheet FV graph gains a
new, explicitly bounded cancellation/store or small-rate sub-lane on this corpus.

Keeping the final FV sum in PC64 through `calcFv(...) - requested_fv`, spilling
it at a helper-call boundary, and all enumerated objective/difference/division/
reciprocal/update spill combinations do not lift RATE beyond 2/256 exact.

## Schedule classification

The public FV residual `FV(guess, ...) - requested_fv` is below `1e-7` on all
256 rows (maximum absolute residual `9.685754776000977e-8`).  The leading 95
arithmetic graph pairs were then iterated under eleven publication/stop rules:

| Schedule | Exact | ≤1 | ≤4 | ≤16 | No result |
|---|---:|---:|---:|---:|---:|
| First step | 2 | 2 | 4 | 8 | 0 |
| Second step | 2 | 5 | 5 | 9 | 0 |
| Stop on pre-step `abs(f)`, publish current | 0 | 0 | 0 | 0 | 0 |
| Stop on pre-step `abs(f)`, publish next | 2 | 2 | 4 | 8 | 0 |
| Minimum two steps, residual stop, publish next | 2 | 5 | 5 | 9 | 3 |
| Stop on `abs(delta)`, publish current | 0 | 0 | 0 | 0 | 0 |
| Stop on `abs(delta)`, publish next | 2 | 2 | 4 | 8 | 0 |
| Stop on next residual, publish next | 2 | 2 | 4 | 8 | 3 |
| Iterate until stable / last | 3 | 5 | 5 | 9 | 0 |
| Fixed 100 steps | 3 | 5 | 5 | 9 | 0 |
| Root-adjacent minimum residual | 9 | 11 | 20 | 26 | 0 |

Current-build evidence rules out a stop rule that publishes the unstepped current
guess.  It does not yet separate pre-step residual-stop-next from
delta-stop-next because the leading arithmetic graphs produce the same first
publication on this constructed bank.  First-step publication has the lowest
aggregate ULP error among the iterative schedules, so this discovery does not
defensibly refute the inherited apply-one-step premise; it does show that the
premise cannot be promoted as exact until the helper arithmetic is identified.

## Frozen artifacts and SHA-256

### RATE discovery and sealed heldout inputs

- `candidate-manifest-rate-one-step-v2.json`
  `C5D67C25162F01A0E3AB9A95D282C34345359C9C76A4D79553AF17F4C7F6F1EA`
- `meta-rate-one-step-discovery-v2.json`
  `83EF51D6F13AE373B9BC52CD1E1C29EE3842A173B5C0F1B938DAAB7570F794F7`
- `batch-rate-one-step-discovery-v2.json`
  `09E7226D5D2DDA3E5AADF6D16DA3C5E87931F3686D4F3FF1AC96F86D0D314B00`
- `answers-rate-one-step-discovery-v2.json`
  `DBE9C5C3F8FFC6126536EDF492BCE032F1518252BBE121508389FF5C07C8DD21`
- `meta-rate-one-step-heldout-v2.json` (sealed input metadata only)
  `63BD955CA8C2810D690FBA3A1F76625A47AE3D98F1492D540BBC3FF069CAD25C`
- `batch-rate-one-step-heldout-v2.json` (sealed input; no answers)
  `7F33EB64FF8092A8889A6B298FAD3BA07EC53C89F1ADC6C1C1038F1E36C870E3`

### FV companion

- `candidate-manifest-rate-fv-companion-v1.json`
  `6C5D90B381131D98202B9999135B921A46B9A25FE4C1C2C3CF6E5D1F4020FAF6`
- `meta-rate-fv-companion-discovery-v1.json`
  `426DB23C39116812F60B755ECF39352ED4650C251831A75F9FF355F441AE1318`
- `batch-rate-fv-companion-discovery-v1.json`
  `2C36574F0559BE15AE422839441003DA9467CE351819E8233ABE6F409DEC5753`
- `answers-rate-fv-companion-discovery-v1.json`
  `9B8EFDFAC149CDF937C60DAD288C23D2F46A40FE61F1EBD120FD44BF4443C4AB`

### Reports

- `report-rate-one-step-discovery-v2.json`
  `E6973039187E0B4A73216E329713835C8860E9811BDEAD996FBAF2D4A513BA65`
- `report-rate-one-step-discovery-v2-classification.json`
  `2EDD51C74665EBBD9C6BBE3C914CEEEFDB25546D1A032B782ED496EEFD3E701F`
- `report-rate-fv-companion-discovery-v1.json`
  `3339453448F90CF30D03766F53536189F795CEBF7CDF1CEBA8D17640D46FF514`
- `report-rate-fv-inline-discovery-v1.json`
  `8EB9DF4E569EE7FF177EB0E5D1EA512BEA9E2840EA803131E7170F8D6A5122FC`

All JSON artifacts above live under `smart-fuzzer/work/w109/G6-rate/`.

## Durable tools and SHA-256

- `src/bin/generate_rate_one_step_discriminator.rs`
  `BEE2B8F4EC2D8C6EC297222A810AD478F871014CF81EAF1EA93E1FAF75861BDD`
- `src/bin/rate_research/one_step.rs`
  `3253CA13F6D12ED540A28666B11A766269149321DFA10371D1E5EE32F8008EC4`
- `src/bin/score_rate_one_step_discovery.rs`
  `6723DD8E4F38F33AD1AEBB310BEC505763E2611C0E46E2F2FC342F9C8A35007A`
- `src/bin/generate_rate_fv_companion.rs`
  `F9D2EF0B509304DDCD2CF1A6C36D9E5D9DCDB79E259ADC54EC8AC650003CC0FF`
- `src/bin/score_rate_fv_companion.rs`
  `AEC594768FCE09D6987CBF4CE0352BC1143F71C6CEE0B2D43E9D1D0B875EE381`
- `src/bin/race_rate_fv_inline.rs`
  `371F26C8597A131C20FAEA285B55061590C5C81D438A92323CB6F53725153065`

Tool paths are relative to `smart-fuzzer/tools/calc_graph_racer/`.

## Offline replay

From `smart-fuzzer/tools/calc_graph_racer/`:

```powershell
rustfmt --edition 2024 --check src/bin/generate_rate_one_step_discriminator.rs src/bin/rate_research/one_step.rs src/bin/score_rate_one_step_discovery.rs src/bin/generate_rate_fv_companion.rs src/bin/score_rate_fv_companion.rs src/bin/race_rate_fv_inline.rs
cargo check --quiet --bin generate_rate_one_step_discriminator --bin score_rate_one_step_discovery --bin generate_rate_fv_companion --bin score_rate_fv_companion --bin race_rate_fv_inline
cargo run --release --quiet --bin generate_rate_one_step_discriminator
cargo run --release --quiet --bin score_rate_one_step_discovery
cargo run --release --quiet --bin generate_rate_fv_companion
cargo run --release --quiet --bin score_rate_fv_companion
cargo run --release --quiet --bin race_rate_fv_inline
```

The two scorers require the discovery answer files listed above.  No replay
command opens or scores the sealed heldout inputs.

## Open lanes

1. Identify the ten-row cancellation/small-rate branch or factor staging in the
   worksheet FV helper; the previous 149-row FV claim does not cover this bank.
2. Identify RATE's internal `calcFv`/balance arithmetic beyond the exhaustive
   factor/association/spill grammar here, with emphasis on a dedicated small-rate
   path or a factor-minus-one primitive rather than more local store toggles.
3. Re-run schedule classification only after an exact internal objective graph
   exists; residual-stop-next and delta-stop-next remain observationally tied.
4. Freeze one coherent discovery survivor before requesting any heldout capture.
5. Keep the existing 256-row heldout sealed until item 4 is satisfied.
6. Characterize the wider RATE function surface: multi-iteration basins, cap,
   omitted/default guess, coercion, domain/errors, cross-version axes, and
   production integration.
