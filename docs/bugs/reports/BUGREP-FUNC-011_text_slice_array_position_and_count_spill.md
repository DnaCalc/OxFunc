# BUGREP-FUNC-011: User follow-on: MID, LEFT, and RIGHT array-valued position/count spill

## Intake
- **Report id**: `BUGREP-FUNC-011`
- **Filed**: `2026-04-08`
- **Source channel**: user
- **Reporter/source**: local follow-on direction from the user after lookup-family review
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: exact working ref pinned with `git rev-parse HEAD`
  at intake time.
- **Canonical bug id**: `BUG-FUNC-007`
- **Status**: `triaged`

## Observed Symptom
Live Excel spills ordinary text-slice functions when a scalar position/count
parameter is itself an array. The concrete prompt case is
`MID("MISSISSIPPI",SEQUENCE(11),1)`, which spills one character per row in
Excel. The current local OxFunc text-slice surface still routes those
parameters through scalar-only numeric coercion, so the same lane would surface
`#VALUE!` instead of a spill array.

## Reproduction
1. Evaluate `=MID("MISSISSIPPI",SEQUENCE(11),1)`.
2. Expected Excel result: spill `M;I;S;S;I;S;S;I;P;P;I`.
3. Evaluate `=LEFT("MISSISSIPPI",SEQUENCE(3))`.
4. Expected Excel result: spill `M;MI;MIS`.
5. Evaluate `=RIGHT("MISSISSIPPI",SEQUENCE(3))`.
6. Expected Excel result: spill `I;PI;PPI`.

## Initial Ownership Read
- **Initial classification**: `OxFunc-owned bug`
- **Reason**: `text_slice_family.rs` still prepared `LEFT` / `RIGHT` / `MID`
  through the generic values-only scalar coercion path, and array-valued
  count/start inputs were not admitted locally even though Excel spills them.

## Links
1. `crates/oxfunc_core/src/functions/text_slice_family.rs`
2. `crates/oxfunc_core/src/functions/text_b_compat_family.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`

## Triage Notes
The immediate prompted functions are `LEFT`, `RIGHT`, and `MID`, but the
current-baseline `*B` delegates inherit the same local runtime path. This
report therefore opens a text-slice family stream rather than a single-formula
fix note.
