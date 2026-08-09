# BUG-FUNC-044: RRI cancellation and power-staging drift

## Summary
- **Bug id**: `BUG-FUNC-044`
- **Opened**: 2026-08-09
- **Status**: validating_local
- **Owner workset**: `W109`

## Source Refs
- **Reported against ref**: `e73115a`
- **Reproduced on ref**: `e73115a` against the 2026-07-24 live-Excel grid
- **Introduced in ref**: unknown
- **Fixed in ref**: working tree pending commit
- **Ref notes**: The oracle answers were captured through the Value2 cell-reference batch runner and replayed against the current production kernel with `race_effect_rri_check`.

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: wrong_composition
- **Root cause summary**: RRI is a legacy x87 spill body with DAZ-normalized
  value and quotient boundaries, period/equality/sign guards in a pinned order,
  an exact `periods==1` identity route, and otherwise a raw stored-LN,
  reciprocal-first double-rounded product, x87-EXP chain. It deliberately
  bypasses worksheet POWER's special dispatch. The quotient, reciprocal, and
  final subtraction are x87 double-rounded.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: unknown
- **Spec vague or missing?**: yes
- **Code once correct and later regressed?**: no
- **Likely introduced in ref**: unknown
- **Explanation**: The original conformance set used ordinary-scale outputs where several power implementations collapse to the same published bits; it omitted near-one power results whose final subtraction magnifies one power-bit into `2^27` result ULP.

## Reproduction
1. From `smart-fuzzer/tools/calc_graph_racer`, run `cargo run --release --bin race_effect_rri_check`.
2. Use `pv=1`, `fv=0x3ff0000100000000` (`1+2^-20`), and `nper` equal to `4`, `16`, or `64`.
3. Excel publishes `nper=64` as `0x3e4fffff00000000`; the current production result is `+134,217,728` output ULP high. The full banked grid is `151/154` exact and only those three cancellation-ladder rows miss.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md#G6--financial-exactness-computation-and-solver`
- **Spec state at intake**: vague
- **Notes**: The mathematical identity is recorded, but the exact fractional-power substrate and operation staging are not.

## Investigation Log
1. 2026-07-24: Fresh live-Excel observability grid established a fractional-power route followed by plain subtraction, not `expm1`.
2. 2026-08-09: Current-production replay reconfirmed the three deterministic mismatches and registered catalog row `G6-13`.
3. 2026-08-09: The 4,900-row oracle-blind held-out separated reciprocal-first
   multiplication from `ln(base)/n` (`4900/4900` versus `4803/4900`).
4. 2026-08-09: Live special-dispatch probes prove RRI bypasses POWER: for
   `RRI(2,1,2)` Excel/raw publishes `0x3fda827999fcef30`, while POWER's sqrt
   shortcut publishes `0x3fda827999fcef34`; fractional `n=.5`, `1/3`, and
   `.25` probes agree with the raw chain too.
5. 2026-08-09: Four full-mantissa-period probes identify the product node as
   `RN53(RN64((1/n)*LN(q)))` (`4/4` versus plain-product `0/4`), and six wrapper
   probes identify x87-double-rounded quotient, reciprocal, and final subtract.
6. 2026-08-09: Adversarial domain review exposed a guard gap outside those
   corpora: `RRI(1, f64::MAX, 0x0000000000000001)` publishes exact `-1` in
   Excel because the quotient rounds to `+0`; the unconditional raw-log repair
   returns `#NUM!`.
7. 2026-08-09: A 60-row edge matrix and fresh 35-row disagreement set identify
   the exact order: reject `periods < MIN_NORMAL`; DAZ `pv/fv`; equality to
   `+0` before sign guards; DAZ the x87 quotient; zero base to `-1`; reject
   nonfinite base/reciprocal/result. The composite scores `60/60 + 35/35` while
   former production scores `45/60 + 15/35`.
8. 2026-08-09: A clean six-row live discriminator proves `periods==1` is an
   exact quotient-identity branch (`6/6`); immediately adjacent period doubles
   use the raw chain. Former production scores `3/6`.
9. 2026-08-09: The complete working-tree repair replays all positive-domain
   and edge corpora at `5536/5536`.

## Similar-Risk Scan
### Adjacent families to check
1. `NOMINAL`
2. `PDURATION`
3. Other callers that use native `powf` instead of the identified Excel power substrate

### Check method
1. Race native `powf`, `power_kernel`, `excel_pow_chain`, and explicit exp/log compositions with product/division-store variants against the exact grid.
2. Use adjacent-double near-one ladders and fresh held-out exponents to separate candidates before promotion.

### Results
1. NOMINAL has a distinct direct-FYL2X branch for truncated periods `<=2` and
   is tracked under `BUG-FUNC-045`.
2. PDURATION remains a separate adjacent-family scan; no unverified shared
   wrapper change was made.

### Follow-on Openings
1. `BUG-FUNC-043` tracks the separate EFFECT integer-power discrepancy.

## Fix Plan
1. Run the post-repair full core suite without perturbing the `5536/5536`
   replay.
2. Commit and synchronize the catalog,
   calculation map, ruled-out ledger, and W109 report.

## Validation
1. Banked grid: production repair `154/154`; former native path `151/154`.
2. Fresh provenance-bearing build-20228/CV2 `-NoCache` held-out:
   `4900/4900`; SHA256
   `EDF5304E39855A04BCD4F75E6A6215EA34688F2CC6B0AF2B03AF1E03344E811D`.
3. Fresh boundary follow-up: `375/375`; SHA256
   `2E84DD72CE91BEA8E3D485E17C1995D69B93337FD551EC9E0C091FC735A829F0`.
4. Targeted wrapper staging: `6/6`; product-node and special-dispatch live
   probes all choose the production graph.
5. Edge-domain batteries: selected composite `60/60 + 35/35`; former
   production `45/60 + 15/35`.
6. Clean exact-period discriminator: selected composite `6/6`; former
   production `3/6`.
7. Deterministic unit tests
   `rri_uses_raw_x87_pow_chain_and_x87_spill_wrapper` and
   `rri_matches_excel_daz_guard_order_and_exact_period_identity` pass.
8. `race_effect_rri_check` reports repaired production `5536/5536` across all
   RRI corpora.
9. Post-edge-repair `cargo test --manifest-path crates/oxfunc_core/Cargo.toml`:
   library `1518 passed, 0 failed, 4 ignored`; every shown integration and
   doc-test target passed.
10. `lake build` in `formal/lean`: passed (`492` jobs), including the ordered
    RRI route model and guard-priority theorems.

## Linked Reports
1. `docs/function-lane/W109_EFFECT_RRI_NOMINAL_IDENTIFICATION_20260809.md`

## Evidence
1. `smart-fuzzer/work/w109/G6-solvers/batch-rri-grid.json`
2. `smart-fuzzer/work/w109/G6-solvers/answers-rri-grid.json`
3. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_effect_rri_check.rs`
4. `smart-fuzzer/work/w109/G6-solvers/batch-rri-heldout-20260809.json`
5. `smart-fuzzer/work/w109/G6-solvers/answers-rri-heldout-20260809.json`
6. `smart-fuzzer/work/w109/G6-solvers/batch-rri-followup-20260809.json`
7. `smart-fuzzer/work/w109/G6-solvers/answers-rri-followup-20260809.json`
8. The provenance-annotated 60-row, 35-row, and six-row edge outcomes embedded
   in `race_effect_rri_check.rs` and the W109 evidence ledger.
9. `smart-fuzzer/work/w109/G6-solvers/answers-rri-edge-stage1-20260809.json`
   (`40626DE8452BC87F8DC378CDF4CAD4C8CE03BB41EF8637EB2FB36E17C09AEB6F`)
10. `smart-fuzzer/work/w109/G6-solvers/answers-rri-edge-stage2-20260809.json`
    (`52E11144FA0BC8E0CAE88BB7ACE1F7084173D2836ACD82AC3FD07CA70C171F83`)
11. `smart-fuzzer/work/w109/G6-solvers/answers-rri-period-one-discriminator-20260809.json`
    (`7C751F7A0165377D9E8C23667C0FDA1BADC7DF574AD3687F717FC721069AF6EF`)

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [ ] handoff filed if required
- [x] linked reports updated
