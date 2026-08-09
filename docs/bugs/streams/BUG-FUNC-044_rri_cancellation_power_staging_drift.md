# BUG-FUNC-044: RRI cancellation and power-staging drift

## Summary
- **Bug id**: `BUG-FUNC-044`
- **Opened**: 2026-08-09
- **Status**: `closed_signed_off`
- **Owner workset**: `W109`

## Source Refs
- **Reported against ref**: `e73115a`
- **Reproduced on ref**: `e73115a` against the 2026-07-24 live-Excel grid
- **Introduced in ref**: unknown
- **Fixed in ref**: `876635e`
- **Ref notes**: The oracle answers were captured through the Value2 cell-reference batch runner and replayed against the landed production kernel with `race_effect_rri_check`. The repair, exact pins, contract/formal alignment, and evidence record landed together in `876635e`.

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
3. Excel publishes `nper=64` as `0x3e4fffff00000000`; at the reported ref, the pre-repair production result is `+134,217,728` output ULP high. The full banked grid is `151/154` exact and only those three cancellation-ladder rows miss.

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

## Fix And Landing
1. The identified production graph, ordered guards, DAZ boundaries, identity
   route, and deterministic pins landed in `876635e`.
2. The bug register, open discrepancy catalog, calculation map, ruled-out
   ledger, W109 report/workset surfaces, worklist, and bead state were
   reconciled after landing. Catalog row `G6-13` is retired.
3. Cross-repo impact was assessed. No FEC/F3E boundary, evaluator-facing
   clause, or downstream ownership changed, so no handoff is required.

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
- [x] fix landed or non-OxFunc ownership recorded (`876635e`)
- [x] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] cross-repo impact assessed; handoff not required because no FEC/F3E or evaluator-facing clause changed
- [x] linked reports updated

## Closure Verification (2026-08-09)

Status axes for the declared `BUG-FUNC-044` discrepancy slice:

- `execution_state: complete`
- `scope_completeness: scope_complete`
- `target_completeness: target_complete`
- `integration_completeness: integrated`
- `open_lanes: []` within this bug slice. Wider W109 financial/campaign rows and
  orthogonal alternate-version, alternate-channel, and locale validation are
  outside this bug closure; no function-phase, family, or global claim is made.

Pre-Closure Verification Checklist (`OPERATIONS.md` Section 12):

1. contract rows complete/promoted for the slice: yes; the exact RRI guards,
   DAZ boundaries, identity route, and raw power graph are aligned at the
   required `provisional_w109_aligned` state. The wider family contract remains
   provisional and is not claimed here.
2. Lean/formal alignment satisfied: yes; the ordered route model and guard
   priority theorems landed and the `492`-job Lean build passed.
3. Rust implementation and required tests pass: yes; the full core suite passed
   with `1518` library tests, `0` failures, and `4` ignored tests, and all shown
   integration/doc-test targets passed.
4. deterministic replay artifact exists: yes; the `5536/5536` combined replay
   covers positive, edge-domain, wrapper, and exact-period corpora.
5. evidence links complete and reproducible: yes; hashes and the deterministic
   edge-artifact generator are recorded above and in the linked W109 report.
6. both version axes explicit: yes; Excel 16.0 build 20228 x64 and workbook
   Compatibility Version 2 are the declared reference profile.
7. public-doc/empirical divergence handled in favor of Excel: yes; the landed
   graph follows the reproducible black-box observations.
8. XLL seam limitation documented where material: yes; it is not material to
   this direct worksheet-oracle/core-kernel slice.
9. cross-repo impact assessed: yes; no FEC/F3E boundary or evaluator-facing
   change was made, so no handoff is required.
10. no known semantic gap remains in the declared discrepancy slice: yes.
11. completion-language audit passed: yes; closure is limited to this bug and
    does not claim wider RRI function-phase or campaign closure.
12. in-progress worklist updated: yes; it records the landed repair while the
    wider W109 lane remains partial.
13. execution-state surface updated: yes; bead `oxf-jwh5.2` is closed with the
    landed ref and validation evidence.

Completion Claim Self-Audit (`OPERATIONS.md` Section 14):

1. scope re-read: pass; only the identified RRI cancellation, guard, and
   power-staging bug is claimed.
2. gate criteria re-read: pass; implementation, exact replay, full tests,
   formal alignment, evidence, and landed ref are present.
3. silent scope reduction: pass; no declared guard, identity, or numeric lane
   was dropped.
4. looks-done-but-is-not patterns: pass; no stub, compile-only path,
   unsupported proof claim, or unacknowledged handoff supports closure.
5. result: pass for the declared `BUG-FUNC-044` slice.
