# BUGREP-FUNC-008: XMATCH array-valued lookup_value lift missing

## Intake
- **Report id**: `BUGREP-FUNC-008`
- **Filed**: 2026-04-08
- **Source channel**: user
- **Reporter/source**: downstream OxFml corpus issue relayed by the user
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: Exact working ref pinned with `git rev-parse HEAD`
  at intake time.
- **Canonical bug id**: `BUG-FUNC-006`
- **Status**: `triaged`

## Observed Symptom
`XMATCH` currently rejects an array-valued `lookup_value` with `#VALUE!`, while
live Excel spills one result per lookup-value element. The reported practical
failure is `FILTER + XMATCH + ISNUMBER` set-intersection logic collapsing
because `XMATCH({1,2,3,4,5},...)` fails instead of producing a mixed
`{#N/A,match,#N/A,...}` result vector.

## Reproduction
1. Evaluate `=XMATCH({1,2,3},{2,4,6,8})`.
2. Expected Excel result: spill `{#N/A,1,#N/A}`.
3. Actual current OxFunc result before the local fix: `#VALUE!` because the
   lookup-selection surface treated `lookup_value` as scalar-only.

## Initial Ownership Read
- **Initial classification**: `OxFunc-owned bug`
- **Reason**: local `xmatch_surface.rs` prepares `lookup_value` through the
  scalar `prepare_arg_values_only(...)` path and the `XMATCH` adapter rejects
  `PreparedArgValue::Eval(EvalValue::Array(_))` explicitly.

## Links
1. `crates/oxfunc_core/src/functions/xmatch_surface.rs`
2. `crates/oxfunc_core/src/functions/xmatch.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md`

## Triage Notes
The initial intake is specific to `XMATCH`, but the same lookup-selection
family shape appears in `MATCH`, and adjacent-risk review also identifies
`XLOOKUP` as a likely sibling lane.
