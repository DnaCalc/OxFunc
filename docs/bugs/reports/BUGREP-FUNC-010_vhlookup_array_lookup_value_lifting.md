# BUGREP-FUNC-010: User follow-on: VLOOKUP and HLOOKUP array-valued lookup_value lifting

## Intake
- **Report id**: `BUGREP-FUNC-010`
- **Filed**: 2026-04-08
- **Source channel**: user
- **Reporter/source**: local follow-on direction from the user during lookup-family review
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: exact working ref pinned with `git rev-parse HEAD`
  at intake time.
- **Canonical bug id**: `BUG-FUNC-006`
- **Status**: `triaged`

## Observed Symptom
`VLOOKUP` and `HLOOKUP` were identified as likely siblings of the already-open
`XMATCH` / `MATCH` array-valued `lookup_value` gap. Live Excel replay confirms
both functions spill one result per lookup-value element, while the current
local OxFunc `vhlookup_family.rs` path still assumes the shared match result is
scalar-only.

## Reproduction
1. Evaluate `=VLOOKUP({1,2,3},{2,20;4,40;6,60;8,80},2,FALSE)`.
2. Expected Excel result: spill `{#N/A,20,#N/A}`.
3. Evaluate `=HLOOKUP({1,2,3},{2,4,6,8;20,40,60,80},2,FALSE)`.
4. Expected Excel result: spill `{#N/A,20,#N/A}`.
5. Pre-fix local OxFunc structural read: `match_index(...)` accepted only a
   scalar `EvalValue::Number`, so an array-valued `MATCH` result would collapse
   to `#VALUE!` rather than selecting one cell per lookup needle.

## Initial Ownership Read
- **Initial classification**: `OxFunc-owned bug`
- **Reason**: the failure is in the OxFunc `VLOOKUP` / `HLOOKUP` surface logic,
  not in OxFml parsing or binding. The local surface called into the corrected
  `MATCH` family but still rejected array-valued match results.

## Links
1. `crates/oxfunc_core/src/functions/vhlookup_family.rs`
2. `crates/oxfunc_core/src/functions/match_fn.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md`

## Triage Notes
This report widens the existing canonical lookup-family stream rather than
opening a second duplicate stream. `VLOOKUP` and `HLOOKUP` share the same local
implementation family and the same empirical Excel direction as `MATCH` /
`XMATCH`.
