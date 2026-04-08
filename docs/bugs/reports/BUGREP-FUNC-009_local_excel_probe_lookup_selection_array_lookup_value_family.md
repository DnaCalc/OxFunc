# BUGREP-FUNC-009: Local Excel probe on lookup-selection array-valued lookup_value family

## Intake
- **Report id**: `BUGREP-FUNC-009`
- **Filed**: 2026-04-08
- **Source channel**: local test
- **Reporter/source**: live Excel COM probe on current workstation
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reported against kind**: `commit`
- **Reported against note**: Exact working ref pinned with `git rev-parse HEAD`
  at intake time.
- **Canonical bug id**: `BUG-FUNC-006`
- **Status**: `triaged`

## Observed Symptom
Live Excel confirms that the lookup-selection family spills array-valued lookup
needles rather than rejecting them as invalid scalar inputs.

## Reproduction
1. Live Excel COM replay on 2026-04-08 observed:
   - `=XMATCH({1,2,3},{2,4,6,8}) -> {#N/A,1,#N/A}`
   - `=MATCH({1,2,3},{2,4,6,8},0) -> {#N/A,1,#N/A}`
   - `=XLOOKUP({1,2,3},{2,4,6,8},{20,40,60,80}) -> {#N/A,20,#N/A}`
2. The composed target formula also matched Excel:
   - `=SUM(FILTER({1,2,3,4,5},ISNUMBER(XMATCH({1,2,3,4,5},{2,4,6,8})))) -> 6`
3. Current local code review showed `XMATCH`, `MATCH`, and `XLOOKUP` all still
   prepare `lookup_value` through scalar-only paths.

## Initial Ownership Read
- **Initial classification**: `OxFunc-owned bug`
- **Reason**: the reproduced divergence is in the local lookup-selection
  surface/runtime stack, not in OxFml parse/bind transport. OxFml can already
  pass array-valued ordinary arguments through to OxFunc.

## Links
1. `crates/oxfunc_core/src/functions/xmatch_surface.rs`
2. `crates/oxfunc_core/src/functions/match_fn.rs`
3. `crates/oxfunc_core/src/functions/xlookup.rs`
4. `docs/bugs/streams/BUG-FUNC-006_lookup_selection_array_lookup_value_lifting_gap.md`

## Triage Notes
This local probe widens the original XMATCH report into a lookup-selection
family stream. `XMATCH` and `MATCH` are suitable for the first local fix pass;
`XLOOKUP` remains a follow-on lane because array-valued lookup needles can
interact with array-valued return payload selection.
