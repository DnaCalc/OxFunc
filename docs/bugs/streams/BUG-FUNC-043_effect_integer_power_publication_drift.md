# BUG-FUNC-043: EFFECT integer-power publication drift

## Summary
- **Bug id**: `BUG-FUNC-043`
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
- **Root cause summary**: EFFECT uses x87-double-rounded LSB-first binary
  exponentiation only for truncated counts below `u32::MAX`. At exactly
  `u32::MAX` it dispatches to the raw stored-base x87 LN/product/EXP chain.
  Its base divide/add and final subtraction are double-rounded in both routes.
  Reusing POWER's ordinary-f64 helper or extending the spill loop to a wider
  counter loses this call-site-specific graph and boundary.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: unknown
- **Spec vague or missing?**: yes
- **Code once correct and later regressed?**: no
- **Likely introduced in ref**: unknown
- **Explanation**: Earlier typical-value witnesses did not exercise integer-power order divergence followed by catastrophic subtraction of one.

## Reproduction
1. From `smart-fuzzer/tools/calc_graph_racer`, run `cargo run --release --bin race_effect_rri_check`.
2. Live Excel publishes `EFFECT(0.03125,8)` as `0x3fa038708c703800`.
3. At the reported ref, the pre-repair production path is 32 output ULP high on that row. Across the banked grid it is `305/315` exact; nine `nominal/npery = 2^-8` rows miss by `+32..+76` ULP and one tiny-rate row misses by `+65,536` ULP.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md#G6--financial-exactness-computation-and-solver`
- **Spec state at intake**: vague
- **Notes**: The mathematical identity is recorded, but the exact binary64 operation graph is not.

## Investigation Log
1. 2026-07-24: Fresh live-Excel observability grid localized the gross route to integer binary exponentiation followed by plain subtraction.
2. 2026-08-09: Current-production replay reconfirmed all ten mismatches and registered catalog row `G6-12`.
3. 2026-08-09: Candidate racing identified the x87 spill graph at `315/315`.
   A fresh oracle-blind 870-row battery rejected the plain integer path
   (`770/870`) while the identified graph scored `870/870`.
4. 2026-08-09: Four targeted base-add probes separated ordinary binary64 from
   x87 double rounding `4/4`. The production repair and exact regression pins
   replay the characterized small-period artifacts with zero residual.
5. 2026-08-09: A fresh provenance-bearing 160-row extreme-domain battery
   located a post-truncation dispatch at `u32::MAX`: 4,294,967,294 stays on the
   spill loop, while 4,294,967,295 and every larger tested finite count use
   `excel_pow_chain`. The hybrid scores `160/160`; the former u64-loop
   representative scores `144/160`.

## Similar-Risk Scan
### Adjacent families to check
1. `NOMINAL`
2. Integer-exponent discount factors used by bond and annuity functions

### Check method
1. Race binary-exponentiation traversal orders, operand-store placements, x87 double-rounded operations, and final subtraction staging over the full cached grid.
2. Validate the selected graph on a fresh held-out adjacent-double grid before promotion.

### Results
1. NOMINAL does not reuse EFFECT's integer body; it has its own two-route
   fractional-power graph under `BUG-FUNC-045`.
2. RRI likewise uses a raw reciprocal-first LN/EXP chain under `BUG-FUNC-044`.
3. No shared POWER-wrapper change is appropriate; each financial surface now
   owns its empirically identified call-site graph.
4. Extreme positive counts through `f64::MAX` were included; numeric zero and
   `#NUM!` overflow publication matched all 160 live typed outcomes.

### Follow-on Openings
1. `BUG-FUNC-044` tracks the adjacent RRI cancellation discrepancy separately.

## Fix And Landing
1. The identified production graph and deterministic pins landed in `876635e`.
2. The bug register, open discrepancy catalog, calculation map, ruled-out
   ledger, W109 report/workset surfaces, worklist, and bead state were
   reconciled after landing. Catalog row `G6-12` is retired.
3. Cross-repo impact was assessed. No FEC/F3E boundary, evaluator-facing
   clause, or downstream ownership changed, so no handoff is required.

## Validation
1. Banked grid: production repair `315/315`; former plain/POWER route `305/315`.
2. Fresh provenance-bearing build-20228/CV2 `-NoCache` held-out:
   `870/870`; SHA256
   `64B42A8B394612FA90CB9C1711D4897970661D8241C1F251D30DF2E13EC7C732`.
3. Targeted wrapper staging: `4/4` exact.
4. Fresh extreme-domain/dispatch battery: `160/160` exact and by kind; SHA256
   `EB7CBA416C3A4C7145A1661ADCEBC6D3A3FC7645750F2CAF0E7E8A592D8430C0`.
5. Deterministic unit tests:
   `effect_uses_x87_spill_binexp_on_banked_and_blind_discriminators` passes.
   `effect_switches_to_raw_pow_chain_at_u32_max_truncated_periods` passes.
6. Post-repair `cargo test --manifest-path crates/oxfunc_core/Cargo.toml`:
   passed;
   library `1518 passed, 0 failed, 4 ignored`, with all integration and doc-test
   executables passing.

## Linked Reports
1. `docs/function-lane/W109_EFFECT_RRI_NOMINAL_IDENTIFICATION_20260809.md`

## Evidence
1. `smart-fuzzer/work/w109/G6-solvers/batch-effect-grid.json`
2. `smart-fuzzer/work/w109/G6-solvers/answers-effect-grid.json`
3. `smart-fuzzer/tools/calc_graph_racer/src/bin/race_effect_rri_check.rs`
4. `smart-fuzzer/work/w109/G6-solvers/batch-effect-heldout-20260809.json`
5. `smart-fuzzer/work/w109/G6-solvers/answers-effect-heldout-20260809.json`
6. `smart-fuzzer/work/w109/G6-solvers/batch-effect-huge-domain-scratch.json`
7. `smart-fuzzer/work/w109/G6-solvers/answers-effect-huge-domain-scratch.json`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded (`876635e`)
- [x] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] cross-repo impact assessed; handoff not required because no FEC/F3E or evaluator-facing clause changed
- [x] linked reports updated

## Closure Verification (2026-08-09)

Status axes for the declared `BUG-FUNC-043` discrepancy slice:

- `execution_state: complete`
- `scope_completeness: scope_complete`
- `target_completeness: target_complete`
- `integration_completeness: integrated`
- `open_lanes: []` within this bug slice. Wider W109 financial/campaign rows and
  orthogonal alternate-version, alternate-channel, and locale validation are
  outside this bug closure; no function-phase, family, or global claim is made.

Pre-Closure Verification Checklist (`OPERATIONS.md` Section 12):

1. contract rows complete/promoted for the slice: yes; the exact EFFECT route
   and dispatch are aligned at the required `provisional_w109_aligned` state.
   The wider family contract remains provisional and is not claimed here.
2. Lean/formal alignment satisfied: yes; the route classifier landed and the
   `492`-job Lean build passed.
3. Rust implementation and required tests pass: yes; the full core suite passed
   with `1518` library tests, `0` failures, and `4` ignored tests, and all shown
   integration/doc-test targets passed.
4. deterministic replay artifact exists: yes; the `1349/1349` combined replay
   includes banked, held-out, wrapper, and extreme-domain/dispatch corpora.
5. evidence links complete and reproducible: yes; hashes and generator/racer
   paths are recorded above and in the linked W109 report.
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
    does not claim wider EFFECT function-phase or campaign closure.
12. in-progress worklist updated: yes; it records the landed repair while the
    wider W109 lane remains partial.
13. execution-state surface updated: yes; bead `oxf-jwh5.1` is closed with the
    landed ref and validation evidence.

Completion Claim Self-Audit (`OPERATIONS.md` Section 14):

1. scope re-read: pass; only the identified EFFECT publication/dispatch bug is
   claimed.
2. gate criteria re-read: pass; implementation, exact replay, full tests,
   formal alignment, evidence, and landed ref are present.
3. silent scope reduction: pass; no declared input or dispatch boundary was
   dropped.
4. looks-done-but-is-not patterns: pass; no stub, compile-only path,
   unsupported proof claim, or unacknowledged handoff supports closure.
5. result: pass for the declared `BUG-FUNC-043` slice.
