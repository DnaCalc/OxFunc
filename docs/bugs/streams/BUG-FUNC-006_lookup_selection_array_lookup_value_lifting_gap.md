# BUG-FUNC-006: Lookup-family array-valued lookup_value lifting gap

## Summary
- **Bug id**: `BUG-FUNC-006`
- **Opened**: 2026-04-08
- **Status**: `validated_local`
- **Owner workset**: `W079`

## Source Refs
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reproduced on ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `pending_landed_ref`
- **Ref notes**: intake pinned the current working ref with `git rev-parse HEAD`.
  Live Excel COM replay on 2026-04-08 reproduced the spill behavior directly
  against the installed baseline. Local correction for `XMATCH`, `MATCH`,
  `VLOOKUP`, and `HLOOKUP` is landed on
  `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`. Fresh Excel COM replay on
  2026-04-29 pinned the adjacent `XLOOKUP` multi-needle return shape, and the
  local `XLOOKUP` correction is validated in the working tree with landed-ref
  promotion pending.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the earlier lookup-family closures exercised only
  scalar lookup-value lanes. `XMATCH`, `MATCH`, `VLOOKUP`, `HLOOKUP`, and
  adjacent `XLOOKUP` therefore kept scalar-only `lookup_value` assumptions in
  local surface code, and the missing dynamic-array needle rows were never
  pinned in the empirical packet before completion language was used.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `yes`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the repo target is empirical Excel parity, but the admitted
  lookup-family evidence set overfocused on lookup-array shape, wildcard,
  binary, blank-vs-empty, and scalar selection lanes. Array-valued lookup
  needles were omitted, so local code encoded scalar-only assumptions and the
  function records overclaimed closure.

## Reproduction
1. Live Excel on 2026-04-08 observed:
   - `=XMATCH({1,2,3},{2,4,6,8}) -> {#N/A,1,#N/A}`
   - `=MATCH({1,2,3},{2,4,6,8},0) -> {#N/A,1,#N/A}`
   - `=VLOOKUP({1,2,3},{2,20;4,40;6,60;8,80},2,FALSE) -> {#N/A,20,#N/A}`
   - `=HLOOKUP({1,2,3},{2,4,6,8;20,40,60,80},2,FALSE) -> {#N/A,20,#N/A}`
   - `=SUM(FILTER({1,2,3,4,5},ISNUMBER(XMATCH({1,2,3,4,5},{2,4,6,8})))) -> 6`
2. Actual pre-fix OxFunc behavior:
   - `XMATCH` rejected array-valued `lookup_value` with `#VALUE!`
   - `MATCH` shared the same local scalar-only assumption
   - `VLOOKUP` and `HLOOKUP` still assumed the shared match result was scalar-only
3. Local structural cause:
   - `xmatch_surface.rs` and `match_fn.rs` prepared `lookup_value` as a single
     scalar `PreparedArgValue`,
   - `vhlookup_family.rs` called into that family but still accepted only a
     scalar numeric match index.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/XMATCH_EXECUTION_RECORD.md`
  2. `docs/function-lane/W10_EXECUTION_RECORD.md`
  3. `docs/function-lane/FUNCTION_SLICE_XMATCH_CONTRACT_PRELIM.md`
  4. `docs/function-lane/FUNCTION_SLICE_MATCH_CONTRACT_PRELIM.md`
  5. `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md`
- **Spec state at intake**: `vague`
- **Notes**: the prior records claimed current-phase closure for `XMATCH` and
  the W10 lookup packet without any explicit array-valued lookup-value evidence.
  This bug reopens those claims for the current baseline.

## Investigation Log
1. 2026-04-08: live Excel COM replay confirmed `XMATCH`, `MATCH`, and
   `XLOOKUP` all spill array-valued lookup needles rather than rejecting them.
2. 2026-04-08: live Excel COM replay confirmed `VLOOKUP` and `HLOOKUP` share
   the same array-valued `lookup_value` spill direction on the current
   baseline.
3. 2026-04-08: confirmed the composed `FILTER + ISNUMBER + XMATCH` set-
   intersection formula returns `6` in live Excel.
4. 2026-04-08: confirmed local `XMATCH` rejects
   `PreparedArgValue::Eval(EvalValue::Array(_))` for `lookup_value`.
5. 2026-04-08: confirmed local `MATCH` shares the same surface shape and same
   underlying scalar-needle assumption.
6. 2026-04-08: confirmed local `VLOOKUP` / `HLOOKUP` share the same local
   family direction because they still accept only a scalar match-selection
   result from the underlying `MATCH` surface.
7. 2026-04-08: confirmed local `XLOOKUP` prepares `lookup_value` the same way
   and therefore remains an adjacent-risk lane even after the bounded
   lookup-family fix.
8. 2026-04-08: opened bounded owner `W079` and bead `oxf-hi5t`.
9. 2026-04-08: corrected local `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP`
   lookup-value array lifting, added focused unit and surface-dispatch tests,
   and reopened current-gap truth for `MATCH`, `XMATCH`, `VLOOKUP`,
   `HLOOKUP`, and `XLOOKUP`.
10. 2026-04-29: promoted the `XMATCH` / `MATCH` / `VLOOKUP` / `HLOOKUP`
    correction to landed-ref status on
    `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`; focused tests were replayed
    and current-gap truth narrowed to the adjacent `XLOOKUP` follow-on.
11. 2026-04-29: fresh Excel COM replay on Excel 16.0 build 19929 confirmed
    `XLOOKUP` spills array-valued lookup needles with the lookup-value input
    shape, uses the top-left `if_not_found` fallback value for each missing
    needle, and for matrix return arrays returns the first cell of the selected
    row or column for multi-needle lookup.
12. 2026-04-29: corrected local `XLOOKUP` to preserve array-valued
    `lookup_value` shape and select one scalar return cell per lookup needle;
    focused `xlookup`, `surface_dispatch`, `xmatch_surface`, `match_fn`, and
    `vhlookup_family` tests pass in the working tree. Landed-ref promotion
    remains pending.

## Similar-Risk Scan
### Adjacent families to check
1. `MATCH`
2. `XMATCH`
3. `VLOOKUP`
4. `HLOOKUP`
5. `XLOOKUP`

### Check method
1. live Excel COM replay for scalar and array-valued lookup-needle cases,
2. local code review of `prepare_arg_values_only(...)` lookup-value surfaces,
3. focused Rust/unit/surface-dispatch tests for the first bounded correction.

### Results
1. `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP` are the same local failure
   family and are corrected together in `W079`.
2. `XLOOKUP` is confirmed as an adjacent semantic family in live Excel. Fresh
   replay showed its multi-needle return rule is shape-preserving over
   `lookup_value` and scalarizes matrix return selections to the first cell of
   the selected row or column.
3. `XLOOKUP` is locally corrected and validated in the working tree; the
   remaining open lane is landed-ref promotion.

### Follow-on Openings
1. `W079`

## Fix Plan
1. correct `XMATCH` and `MATCH` so array-valued `lookup_value` spills one local
   result per element using the existing scalar semantics per element,
2. correct `VLOOKUP` and `HLOOKUP` so array-valued `lookup_value` spills one
   local selected-cell result per element using the existing scalar semantics
   per element,
3. add focused unit and dispatch tests for the spilled lookup-family lanes,
4. reopen the stale lookup-family closure records and current-gap surfaces,
5. promote the local `XLOOKUP` correction to a landed ref and then close the
   canonical bug stream.

## Validation
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib xmatch_surface -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib match_fn -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib vhlookup_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib xlookup -- --nocapture`
6. live Excel COM replay on 2026-04-08 for `XMATCH`, `MATCH`, `VLOOKUP`,
   `HLOOKUP`, `XLOOKUP`, and
   the composed `FILTER + ISNUMBER + XMATCH` set-intersection formula
7. 2026-04-29 replayed the four focused local validation commands above:
   - `xmatch_surface`: 11 passed
   - `match_fn`: 11 passed
   - `vhlookup_family`: 11 passed
   - `surface_dispatch`: 75 passed
8. 2026-04-29 fresh `XLOOKUP` validation:
   - `xlookup`: 16 passed
   - `surface_dispatch`: 76 passed
   - `xmatch_surface`: 11 passed
   - `match_fn`: 11 passed
   - `vhlookup_family`: 11 passed

## Linked Reports
1. `BUGREP-FUNC-008`
2. `BUGREP-FUNC-009`
3. `BUGREP-FUNC-010`

## Evidence
1. `crates/oxfunc_core/src/functions/xmatch_surface.rs`
2. `crates/oxfunc_core/src/functions/match_fn.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `crates/oxfunc_core/src/functions/vhlookup_family.rs`
5. `crates/oxfunc_core/src/functions/xlookup.rs`
6. `docs/function-lane/XMATCH_EXECUTION_RECORD.md`
7. `docs/function-lane/W10_EXECUTION_RECORD.md`
8. `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [ ] handoff filed if required
- [x] linked reports updated
