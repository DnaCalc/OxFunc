# BUG-FUNC-011: COUNTBLANK range-only parity gap

## Summary
- **Bug id**: `BUG-FUNC-011`
- **Opened**: `2026-04-10`
- **Status**: `validated_local`
- **Owner workset**: `W084`

## Source Refs
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reproduced on ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: live Excel replay on 2026-04-10 pinned that `COUNTBLANK`
  accepts a real range and counts both empty cells and `""`, but rejects an
  array-valued substitute with `#VALUE!`. On the reported local ref,
  `countblank_fn.rs` still used aggregate-style direct/range expansion via
  `expand_aggregate_arg(...)`; the local `W084` correction now rejects direct
  array-valued substitutes on the working tree while preserving true-range
  blank counting, but the correction is not yet landed on a committed ref.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the admitted helper/aggregate packet pinned
  `COUNTBLANK` counting semantics for empty cells and `""`, but did not
  exercise the array-valued substitute lane that distinguishes Excel's
  range-only policy from the more permissive `COUNT` / `COUNTA` aggregate
  behavior.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `no`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the local implementation inherited the aggregate direct/range
  expansion shape that works for `COUNT` and `COUNTA`, but `COUNTBLANK` is
  narrower on the current Excel baseline. The policy was not empirically pinned
  during the original helper/control packet, so the code stayed over-permissive
  rather than regressing from a previously pinned correct lane.

## Reproduction
1. Live Excel replay on 2026-04-10:
   - `=LET(d,{"";1},COUNTBLANK(d)) -> #VALUE!`
   - `=COUNTBLANK(A1:A3) -> 2` with `A1=""`, `A2=1`, `A3` empty
2. Contrast controls on the same Excel baseline:
   - `=COUNT({"2",TRUE}) -> 0`
   - `=COUNTA({"";1}) -> 2`
   - `=LET(d,{1,2},ROWS(d)) -> 1`
   - `=LET(d,{1,2},COLUMNS(d)) -> 2`
3. Adjacent same-direction rejection controls on the same Excel baseline:
   - `=LET(d,{1,2},AREAS(d)) -> #VALUE!`
   - `=LET(d,{1,2},ISFORMULA(d)) -> #VALUE!`
   - `=LET(d,{1,2},FORMULATEXT(d)) -> #VALUE!`
   - `=LET(d,{1,2,3},SUBTOTAL(9,d)) -> #VALUE!`
   - `=LET(d,{1,2,3},AGGREGATE(9,0,d)) -> #VALUE!`

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/W16_EXECUTION_RECORD.md`
  2. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
  3. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
- **Spec state at intake**: `correct but incomplete`
- **Notes**: the prior helper/control packet pinned that `COUNTBLANK` counts
  empty cells and empty strings within a range but did not explicitly pin the
  array-valued substitute rejection lane. This bug reopens `COUNTBLANK`
  locally until the direct-array policy is aligned and validated honestly.

## Investigation Log
1. 2026-04-10: user asked whether `COUNTBLANK` should only accept a range and
   not an array for Excel compatibility.
2. 2026-04-10: live Excel replay pinned `COUNTBLANK` array-valued substitutes
   as `#VALUE!` while true ranges still count blanks.
3. 2026-04-10: local code review showed `countblank_fn.rs` still expands direct
   aggregate arguments through `expand_aggregate_arg(...)`.
4. 2026-04-10: contrast replay showed `COUNT`, `COUNTA`, `ROWS`, and `COLUMNS`
   remain array-permissive controls, so the likely fix scope is `COUNTBLANK`
   and nearest same-direction reference-sensitive neighbors rather than a
   blanket aggregate-family policy change.
5. 2026-04-11: tightened `countblank_fn.rs` so direct array-valued substitutes
   now raise the bounded `countblank_array_substitute` preparation error while
   true-range inputs retain blank-counting semantics.
6. 2026-04-11: focused local validation passed for `countblank_fn`, including
   the direct-array rejection lane, and `scripts/check-worksets.ps1` passed.

## Similar-Risk Scan
### Adjacent families to check
1. same-direction reference-sensitive rejection surfaces:
   - `AREAS`
   - `ISFORMULA`
   - `FORMULATEXT`
   - `SUBTOTAL`
   - `AGGREGATE`
2. contrast controls that should remain permissive:
   - `COUNT`
   - `COUNTA`
   - `ROWS`
   - `COLUMNS`
3. criteria/database range-semantics neighbors:
   - `COUNTIF`, `COUNTIFS`
   - `SUMIF`, `SUMIFS`
   - `AVERAGEIF`, `AVERAGEIFS`
   - `DCOUNTA`

### Check method
1. direct live Excel replay with inline arrays vs true ranges
2. local code-path review for aggregate-direct expansion vs reference-visible
   adapter policy
3. contrast checks to avoid over-widening the restriction beyond `COUNTBLANK`

### Results
1. `COUNTBLANK` is a real local parity gap candidate:
   - Excel rejects array-valued substitutes with `#VALUE!`
   - current local implementation shape is still array-permissive
2. `COUNT`, `COUNTA`, `ROWS`, and `COLUMNS` are explicit contrast controls and
   should not be silently narrowed into the same rule.
3. `AREAS`, `ISFORMULA`, `FORMULATEXT`, `SUBTOTAL`, and `AGGREGATE` already
   align directionally with the same kind of array rejection and should be used
   as policy neighbors, not reopened blindly.
4. broader criteria/database review remains a consistency scan only; no new
   widening is opened without direct replay evidence.

### Follow-on Openings
1. `W084`

## Fix Plan
1. tighten `COUNTBLANK` so array-valued substitutes are rejected with `#VALUE!`
   while true ranges retain current blank-counting semantics
2. add focused regression tests for:
   - direct inline-array/LET-array rejection
   - range acceptance and blank counting
   - contrast controls showing `COUNT` / `COUNTA` stay permissive
3. reconcile `W051` and workset truth so `COUNTBLANK` is not overclaimed while
   the correction is still only on the working tree

## Validation
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib countblank_fn -- --nocapture`
2. focused live Excel replay for the pinned contrast matrix
3. `powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`

## Linked Reports
1. `BUGREP-FUNC-015`

## Evidence
1. `crates/oxfunc_core/src/functions/countblank_fn.rs`
2. `crates/oxfunc_core/src/functions/count.rs`
3. `crates/oxfunc_core/src/functions/counta.rs`
4. `crates/oxfunc_core/src/functions/rows_fn.rs`
5. `crates/oxfunc_core/src/functions/columns_fn.rs`
6. `crates/oxfunc_core/src/functions/reference_metadata_family.rs`
7. `crates/oxfunc_core/src/functions/subtotal_aggregate_family.rs`
8. `docs/worksets/W084_COUNTBLANK_RANGE_ONLY_PARITY.md`

## Closure Checklist
- [x] local fix implemented
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [ ] handoff filed if required
