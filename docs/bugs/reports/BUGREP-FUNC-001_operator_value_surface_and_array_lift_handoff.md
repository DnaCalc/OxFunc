# BUGREP-FUNC-001: OxFml handoff for operator value surface and array-lift expansion

## Intake
- **Report id**: `BUGREP-FUNC-001`
- **Filed**: 2026-04-07
- **Source channel**: downstream handoff
- **Reporter/source**: `OxFml` via `HANDOFF-OXFUNC-001`
- **Reported against ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Reported against kind**: commit
- **Reported against note**: The handoff did not name a release tag; OxFunc intake bound the report to the current local `HEAD` while validating the cited operator surfaces and contracts.
- **Canonical bug id**: `BUG-FUNC-001`
- **Status**: triaged

## Observed Symptom
OxFml can now route scalar binary arithmetic into OxFunc `FUNC.OP_*` rows, but array-involved binary arithmetic still fails through the current prepared-call/value surface and requires a temporary OxFml compatibility fallback.

## Reproduction
1. Route array-involved operator calls through the current OxFunc binary operator surfaces on the current local `HEAD`.
2. Expected result:
   array-involved ordinary binary arithmetic should travel through the OxFml -> OxFunc seam as typed worksheet values with array lift preserved where admitted.
3. Actual result:
   OxFml reports worksheet `Value` outcomes such as:
   - `={1,2,3;2,3,4}*-1`
   - `={1,2;3,4}+{10,20;30,40}`
   - `={1,2;6,8}/{1,0;3,2}`

## Initial Ownership Read
- **Initial classification**: shared seam gap
- **Reason**: OxFunc owns ordinary operator semantic truth and the immediate scalar-only limit is in the current OxFunc binary value surface, but the failure matters specifically at the OxFml -> OxFunc prepared-call/value seam and should be widened coherently rather than patched row by row.

## Links
1. `../OxFml/docs/handoffs/HANDOFF_OXFUNC_001_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. `crates/oxfunc_core/src/functions/binary_numeric.rs`
4. `crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
5. `crates/oxfunc_core/src/functions/unary_numeric.rs`
6. `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
7. `docs/bugs/streams/BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md`
8. `docs/worksets/W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`

## Triage Notes
Local OxFunc intake confirmed the downstream read:
1. unary arithmetic and postfix percent already use the array-aware unary numeric surface locally,
2. binary arithmetic remains scalar-only through `eval_binary_numeric_surface(...)`,
3. compare/concat are still explicitly scalar-only and remain adjacent risk rather than the same proven bug,
4. reference operators are a different `RefsVisibleInAdapter` family and are not the same value-surface failure.
