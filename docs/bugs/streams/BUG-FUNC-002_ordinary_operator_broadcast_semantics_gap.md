# BUG-FUNC-002: Ordinary operator broadcast semantics gap

## Summary
- **Bug id**: `BUG-FUNC-002`
- **Opened**: 2026-04-07
- **Status**: handed_off
- **Owner workset**: `W074`

## Source Refs
- **Reported against ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Reproduced on ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: The broadcast rule was observed on the current working tree atop `HEAD` using local Excel comparison probes. No release tag was involved, so the intake ref is recorded as the exact commit SHA plus local empirical notes.

## Ownership And Root Cause
- **Ownership class**: shared seam gap
- **Root cause class**: spec_mismatch
- **Root cause summary**: OxFunc and its surviving `W45` contract/evidence surfaces had modeled ordinary operator array behavior as scalar-only or same-shape-only, while the current Excel baseline uses broader broadcast semantics with `#N/A` padding for unsupported coordinates.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: no
- **Spec vague or missing?**: yes
- **Code once correct and later regressed?**: no
- **Likely introduced in ref**: `unknown`
- **Explanation**: the first `W45` operator packet seeded scalar lanes and same-shape arithmetic evidence strongly enough to admit a useful local slice, but it did not probe row-vs-column broadcast or non-broadcastable extent padding. `W073` repaired the immediate scalar-only arithmetic transport failure reported by OxFml, yet the local contract still lagged the actual Excel rule until explicit broadcast probes were run. This reads as a characterization/admission miss, not a later regression.

## Reproduction
1. Run local Excel comparison for broadcast-sensitive ordinary operator formulas:
   - `={1,2}+{1;2}` -> `2|3|3|4`
   - `={"a","b"}&{"x";"y"}` -> `ax|bx|ay|by`
   - `={1,2}={1;2}` -> `TRUE|FALSE|FALSE|TRUE`
   - `={1,2}+{1,2,3}` -> `2|4|#N/A`
2. Expected behavior:
   singleton dimensions broadcast across the larger opposing dimension, row-vs-column combinations spill as outer-product grids, and non-broadcastable coordinates surface `#N/A`.
3. Actual pre-fix local OxFunc behavior:
   arithmetic rejected some array shape combinations and compare/concat did not accept arrays at all.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`
  3. `docs/function-lane/W45_EXECUTION_RECORD.md`
- **Spec state at intake**: vague
- **Notes**: the existing operator contracts correctly preserved scalar seed lanes, but they did not yet encode Excel's broader broadcast rule or the `#N/A` padding behavior for unsupported coordinates.

## Investigation Log
1. 2026-04-07: Ran local Excel comparison formulas for arithmetic, compare, and concat array combinations beyond the original `W073` handoff cases.
2. 2026-04-07: Confirmed that row-vs-column ordinary operator inputs spill as 2-D broadcast grids on the current Excel baseline.
3. 2026-04-07: Confirmed that extra coordinates beyond a non-singleton extent surface `#N/A` rather than collapsing the operator result.
4. 2026-04-07: Confirmed the current OxFunc arithmetic surface still rejected some mismatched shapes and compare/concat remained scalar-only.
5. 2026-04-07: Opened `W074` as the bounded owner for the broader broadcast reconciliation pass.
6. 2026-04-07: Began widening arithmetic and compare/concat onto a shared broadcast helper in the current working tree.
7. 2026-04-07: Refreshed `W45` arithmetic and compare/concat manifests/runners so native evidence now captures spill-grid text and shape for broadcast-sensitive lanes.
8. 2026-04-07: Passed focused Rust and refreshed native `W45` validation on the current working tree.
9. 2026-04-07: Filed `HO-FN-005` to OxFml so the downstream temporary operator fallback can be removed against an agreed landed ref.
10. 2026-04-07: Added representative dispatcher-level concat and range-ref coverage, then promoted the remaining lane to handoff-follow-up only.

## Similar-Risk Scan
### Adjacent families to check
1. unary arithmetic and postfix percent
2. structural reference operators
3. downstream OxFml temporary operator fallbacks

### Check method
1. re-read the current `W45` operator-family contracts and probe manifests,
2. compared local Rust surfaces to direct Excel `Formula2` observations,
3. separated value-broadcast operator behavior from structural reference-operator behavior.

### Results
1. unary arithmetic and postfix percent already operate elementwise over array inputs; they still require broadcast-aware empirical revalidation but are not the newly proven gap.
2. structural reference operators remain a separate `RefsVisibleInAdapter` family and were not implicated by the broadcast probe.
3. the downstream OxFml fallback story depends directly on this broadcast rule and therefore requires a fresh handoff once the local packet is validated.

### Follow-on Openings
1. `W074`
2. `HO-FN-005` if the broadened operator broadcast packet lands and requires OxFml fallback removal

## Fix Plan
1. land a shared ordinary-operator broadcast helper in the current OxFunc value surface,
2. widen arithmetic and compare/concat operators to the observed rule,
3. refresh `W45` manifests/runners so native evidence captures spill grids,
4. revalidate local Rust and native Excel floors,
5. file a downstream OxFunc -> OxFml handoff once the local packet is exercised.

## Validation
1. focused local validation passed on the current working tree:
   - `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib binary_numeric -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib op_add -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_arithmetic_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_compare_concat_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
   - `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavea-operator-arithmetic-baseline.ps1`
   - `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
   - `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`

## Linked Reports
1. `BUGREP-FUNC-002`

## Evidence
1. `docs/bugs/reports/BUGREP-FUNC-002_local_excel_probe_operator_broadcast_semantics.md`
2. `docs/bugs/streams/BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md`
3. `crates/oxfunc_core/src/functions/binary_numeric.rs`
4. `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
5. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
6. `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`
7. `docs/function-lane/W45_WAVEB_OPERATOR_COMPARE_CONCAT_SCENARIO_MANIFEST_SEED.csv`
8. `docs/handoffs/HO-FN-005_ordinary_operator_broadcast_semantics_and_fallback_removal.md`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required
- [x] linked reports updated
