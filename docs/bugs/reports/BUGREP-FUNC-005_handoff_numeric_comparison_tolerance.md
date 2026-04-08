# BUGREP-FUNC-005: OxFml handoff on numeric comparison tolerance

## Intake
- **Report id**: `BUGREP-FUNC-005`
- **Filed**: 2026-04-08
- **Source channel**: downstream handoff
- **Reporter/source**: `../OxFml/docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: Exact working ref pinned with `git rev-parse HEAD` at intake time.
- **Canonical bug id**: `BUG-FUNC-004`
- **Status**: `triaged`

## Observed Symptom
The incoming handoff correctly identified that current OxFunc ordinary numeric
operator comparisons still use exact double comparison and therefore miss
observed Excel near-equality tolerance such as `=0.1+0.2=0.3`.

## Reproduction
1. Compare `=0.1+0.2=0.3` against live Excel and current local operator compare
   helper behavior.
2. Expected result: `TRUE`.
3. Actual pre-fix OxFunc read: exact `partial_cmp`-driven comparison inside
   `operator_compare_concat_family.rs`.

## Initial Ownership Read
- **Initial classification**: `OxFunc-owned bug`
- **Reason**: The current local comparison helper is inside OxFunc and does not
  yet match the observed Excel operator semantics.

## Links
1. `../OxFml/docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
2. `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
3. `docs/worksets/W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md`

## Triage Notes
Initial handoff scope was widened during local Excel reproduction. Criteria
families, database criteria matching, and `SWITCH` share the same observed
tolerant lane, while `MATCH` / `XMATCH` / `DELTA` exact-match paths remain
exact on the tested cases.
