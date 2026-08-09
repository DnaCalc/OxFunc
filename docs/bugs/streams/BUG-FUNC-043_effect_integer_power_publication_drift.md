# BUG-FUNC-043: EFFECT integer-power publication drift

## Summary
- **Bug id**: `BUG-FUNC-043`
- **Opened**: 2026-08-09
- **Status**: validated_local
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
3. The current production path is 32 output ULP high on that row. Across the banked grid it is `305/315` exact; nine `nominal/npery = 2^-8` rows miss by `+32..+76` ULP and one tiny-rate row misses by `+65,536` ULP.

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

## Fix Plan
1. Commit the validated production graph and deterministic pins.
2. Synchronize the catalog, calculation map, ruled-out ledger, and W109 report.

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
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml`: passed before
   the final dispatch edit;
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
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [ ] handoff filed if required
- [x] linked reports updated
