# BUGREP-FUNC-004: OxFml handoff claim on IF empty-text condition

## Intake
- **Report id**: `BUGREP-FUNC-004`
- **Filed**: 2026-04-08
- **Source channel**: downstream handoff
- **Reporter/source**: `../OxFml/docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: Exact working ref pinned with `git rev-parse HEAD` at intake time.
- **Canonical bug id**: `unassigned`
- **Status**: `closed_no_action`

## Observed Symptom
The incoming handoff asserted that Excel returns the false branch for
`=IF("",1,2)` and that current OxFunc therefore diverges on empty-text
condition coercion.

## Reproduction
1. Evaluate `=IF("",1,2)` and `=IFS("",1,TRUE,2)` against a live local Excel
   instance.
2. Expected result from the handoff claim: false-branch value.
3. Actual Excel result: `#VALUE!` for both formulas. Current OxFunc already
   returns `ConditionCoercion(NonNumericText(""))` on the same lanes.

## Initial Ownership Read
- **Initial classification**: `OxFml-owned bug`
- **Reason**: The local OxFunc runtime is already aligned with the observed
  Excel behavior; the divergence was in the upstream handoff read, not in the
  current OxFunc implementation.

## Links
1. `../OxFml/docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
2. `crates/oxfunc_core/src/functions/if_fn.rs`
3. `crates/oxfunc_core/src/functions/choose_ifs_family.rs`
4. `docs/worksets/W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md`

## Triage Notes
The report is preserved for intake history, but it does not open a canonical
OxFunc bug stream because the local runtime already matches the observed Excel
result. `W077` still owns the contract/test pin so this lane cannot regress
silently later.
