# BUG-FUNC-003: Multi-area reference seam collapses to area/string reinterpretation

## Summary
- **Bug id**: `BUG-FUNC-003`
- **Opened**: 2026-04-07
- **Status**: handed_off
- **Owner workset**: `W076`

## Source Refs
- **Reported against ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Reproduced on ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: The OxFml upstream note identified a current seam mismatch on the
  working tree: OxFunc exposed `ReferenceKind::MultiArea` helpers but did not
  use them consistently across union formation, reference-sensitive consumers,
  and current value-required materialization lanes.

## Ownership And Root Cause
- **Ownership class**: shared seam gap
- **Root cause class**: initial_impl_gap
- **Root cause summary**: OxFunc had already introduced a first-class
  `ReferenceKind::MultiArea` carrier, but `OP_UNION_REF` still emitted
  `ReferenceKind::Area` and downstream consumers such as `INDEX` and `AREAS`
  still started from raw parenthesized target parsing rather than the
  first-class helper APIs. Value-required callers likewise depended on
  downstream OxFml-local aggregation instead of OxFunc-owned resolver-driven
  combination semantics.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: yes
- **Spec vague or missing?**: no
- **Code once correct and later regressed?**: no
- **Likely introduced in ref**: `unknown`
- **Explanation**: the intended seam shape was already visible in
  `ReferenceLike::multi_area(...)` and related helpers, but the implementation
  stopped halfway. That left the type-level distinction present in the value
  model while operator/reference consumers still relied on a string convention.

## Reproduction
1. Evaluate same-sheet union composition through the current reference-operator
   path:
   - `OP_UNION_REF(A1:A2,G1:G2)`
2. Expected behavior:
   - result is `EvalValue::Reference(ReferenceLike { kind: MultiArea, ... })`
3. Actual pre-fix OxFunc behavior:
   - result kind was `ReferenceKind::Area`
   - target carried the parenthesized multi-area shape as a string convention
4. Consequence:
   - `INDEX(..., area_num)` and `AREAS` could still work on some seeded paths,
     but only by reparsing the raw target string instead of consuming the
     first-class seam shape first.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/FUNCTION_SLICE_INDEX_CONTRACT_PRELIM.md`
  3. `docs/function-lane/FUNCTION_SLICE_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_CONTRACT_PRELIM.md`
  4. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
- **Spec state at intake**: correct
- **Notes**: the seam direction OxFml requested matches the existing OxFunc type
  model more closely than the old implementation did.

## Investigation Log
1. 2026-04-07: Read and processed the current OxFml upstream note on same-sheet
   multi-area references.
2. 2026-04-07: Confirmed `ReferenceLike::multi_area(...)`,
   `multi_area_targets()`, and `area_count()` already existed in
   `crates/oxfunc_core/src/value.rs`.
3. 2026-04-07: Confirmed `eval_op_union_ref_surface(...)` still returned
   `ReferenceKind::Area` with a parenthesized target string.
4. 2026-04-07: Confirmed `INDEX` and `AREAS` still started from raw target
   string reinterpretation rather than consulting `MultiArea` helpers first.
5. 2026-04-07: Opened bounded owner `W075`.
6. 2026-04-07: Changed `OP_UNION_REF` to return first-class
   `ReferenceKind::MultiArea` and flatten existing multi-area operands.
7. 2026-04-07: Updated `INDEX(..., area_num)` and `AREAS` to consume
   `multi_area_targets()` as the admitted carrier and stop accepting the old
   non-`MultiArea` parenthesized wrapper form.
8. 2026-04-07: Updated adapter / scalarization seams to handle `MultiArea`
   explicitly and added focused dispatcher-level coverage.
9. 2026-04-07: Filed `HO-FN-006` back to OxFml with the local correction and
   remaining landed-ref / downstream-ack lane.
10. 2026-04-07: Opened bounded follow-on owner `W076` for same-sheet
    `MultiArea` value-required materialization through OxFunc-owned resolver
    combination semantics.
11. 2026-04-07: Implemented local resolver-driven same-sheet `MultiArea`
    materialization, preserving member order and combining resolved payloads as
    one row-major row vector for current value-required lanes.
12. 2026-04-07: Updated values-only, aggregate, and lookup-vector callers to
    consume that local resolver-owned materialization path rather than relying
    on a downstream OxFml-local helper.
13. 2026-04-07: Filed `HO-FN-007` back to OxFml with the Style A local
    materialization result and remaining landed-ref / downstream-ack lane.
14. 2026-04-07: Re-ran the existing `W45` native Excel reference probe floor
    and focused local Rust validation to confirm the local correction remains
    consistent with the admitted reference packet behavior.

## Similar-Risk Scan
### Adjacent families to check
1. `AREAS`
2. `INDEX(..., area_num)`
3. values-only / aggregate / lookup-vector preparation paths
4. implicit intersection and any other `ReferenceKind` pattern matches
5. resolver normalization and capability checks

### Check method
1. searched all current OxFunc `ReferenceKind` pattern matches,
2. inspected the value model, resolver, operator-reference family, and
   reference-sensitive consumers,
3. added focused local runtime tests at function and dispatch level.

### Results
1. `AREAS` and `INDEX` were the first concrete consumer gaps and are now
   updated.
2. values-only, aggregate, and lookup-vector preparation were the follow-on
   current-value gap and now route through OxFunc-owned resolver combination.
3. resolver normalization/capability handling already preserved
   `ReferenceKind::MultiArea`.
4. `op_implicit_intersection` needed an explicit `MultiArea` path; it now
   rejects that source rather than relying on a non-exhaustive match.
5. no broader reference-family consumer was changed in this pass because the
   current concrete seam pressure was limited to union formation and the named
   consumers; the remaining local consumers continue to treat unsupported
   multi-area use as out of slice rather than reparsing a wrapper string.

### Follow-on Openings
1. `W075`
2. `W076`
3. `HO-FN-006`
4. `HO-FN-007`

## Validation
1. focused local validation passed on the current working tree:
   - `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib resolver -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib adapters -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib sum -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib xmatch_surface -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_reference_family -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib index -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
   - `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`
   - `powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`

## Linked Reports
1. `BUGREP-FUNC-003`

## Evidence
1. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
2. `crates/oxfunc_core/src/value.rs`
3. `crates/oxfunc_core/src/resolver.rs`
4. `crates/oxfunc_core/src/functions/operator_reference_family.rs`
5. `crates/oxfunc_core/src/functions/index.rs`
6. `crates/oxfunc_core/src/functions/reference_metadata_family.rs`
7. `crates/oxfunc_core/src/functions/op_implicit_intersection.rs`
8. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
9. `docs/handoffs/HO-FN-006_multi_area_reference_seam_correction.md`
10. `docs/handoffs/HO-FN-007_multiarea_value_materialization_style_a.md`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required
- [x] linked reports updated
