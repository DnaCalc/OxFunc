# BUG-FUNC-007: Text-slice array-valued position/count spill gap

## Summary
- **Bug id**: `BUG-FUNC-007`
- **Opened**: 2026-04-08
- **Status**: `closed`
- **Owner workset**: `W080`

## Source Refs
- **Reported against ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Reproduced on ref**: `7989fafaef703f15f2bfbdded323c03345da1072`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Ref notes**: intake pinned the current working ref with `git rev-parse HEAD`.
  Live Excel COM replay on 2026-04-08 reproduced the spill behavior directly
  against the installed baseline. The local correction for `LEFT` / `LEFTB`,
  `MID` / `MIDB`, and `RIGHT` / `RIGHTB` is landed on the fixed ref above; the
  broader function-array-support review remains open under `W080` separately.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the earlier text-core closure packets pinned scalar
  UTF-16 and domain lanes but omitted dynamic-array count/position inputs.
  Local text-slice runtime therefore kept scalar-only coercion assumptions for
  `LEFT`, `RIGHT`, and `MID`.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `yes`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the repo target is empirical Excel parity, but the admitted
  text-core evidence set overfocused on UTF-16 unit handling, surrogate-edge
  behavior, and scalar domain checks. Array-valued position/count lanes were
  omitted, so the local scalar-only values-only seam remained unchallenged and
  the old closure language overclaimed the text-slice family.

## Reproduction
1. Live Excel on 2026-04-08 observed:
   - `=MID("MISSISSIPPI",SEQUENCE(11),1) -> {M;I;S;S;I;S;S;I;P;P;I}`
   - `=LEFT("MISSISSIPPI",SEQUENCE(3)) -> {M;MI;MIS}`
   - `=RIGHT("MISSISSIPPI",SEQUENCE(3)) -> {I;PI;PPI}`
2. Actual pre-fix OxFunc structural behavior:
   - `LEFT`, `RIGHT`, and `MID` all routed count/start inputs through scalar
     `coerce_prepared_to_number(...)` calls,
   - array-valued count/start inputs therefore surfaced `#VALUE!` instead of a
     spill array.
3. Local structural cause:
   - `text_slice_family.rs` used `run_values_only_prepared(...)` with scalar
     prepared arguments only, and the generic scalar coercion helpers reject
     `PreparedArgValue::Eval(EvalValue::Array(_))`.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
- **Spec state at intake**: `vague`
- **Notes**: the prior text-core records claimed current-baseline closure
  without any explicit dynamic-array position/count evidence. This bug reopens
  the text-slice group for the current target until the fix lands on a committed
  ref and the broader review packet runs honestly.

## Investigation Log
1. 2026-04-08: live Excel COM replay confirmed `MID`, `LEFT`, and `RIGHT`
   spill over array-valued start/count inputs rather than rejecting them.
2. 2026-04-08: confirmed local `text_slice_family.rs` still used scalar-only
   values-only preparation for count/start arguments.
3. 2026-04-08: confirmed current-baseline `LEFTB`, `MIDB`, and `RIGHTB`
   delegate to the same local text-slice runtime and therefore share the same
   correction path.
4. 2026-04-08: corrected local `LEFT`, `RIGHT`, and `MID` to lift over a
   single array-valued parameter while preserving the existing scalar semantics
   per element.
5. 2026-04-08: opened bounded owner `W080` to carry the immediate text-slice
   fix and the broader systematic array-support review seed.
6. 2026-04-29: promoted the text-slice correction to landed-ref status on
   `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`; focused tests were replayed and
   current-gap truth no longer carries these fixed rows.

## Similar-Risk Scan
### Adjacent families to check
1. `LEFT` / `LEFTB`
2. `MID` / `MIDB`
3. `RIGHT` / `RIGHTB`
4. scalar-parameter text families such as `FIND`, `SEARCH`, and `REPLACE`
5. broader supported ordinary functions that may admit array-valued scalar
   parameters under current Excel dynamic-array behavior

### Check method
1. live Excel COM replay for the prompted `MID`, `LEFT`, and `RIGHT` formulas,
2. local code review of values-only scalar coercion sites,
3. focused Rust/unit/surface-dispatch tests for the corrected text-slice family,
4. new bounded review owner `W080` for broader family-by-family scan planning.

### Results
1. `LEFT` / `LEFTB`, `MID` / `MIDB`, and `RIGHT` / `RIGHTB` are corrected
   together on landed ref `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`.
2. adjacent scalar-parameter text/search functions remain unprobed in this pass
   and therefore stay open for systematic review rather than assumed.
3. the broader array-support question clearly exceeds a one-family bug fix and
   now has an explicit local owner in `W080`.

### Follow-on Openings
1. `W080`

## Fix Plan
1. correct `LEFT`, `RIGHT`, and `MID` so a single array-valued count/start
   input spills one local result per element using the existing scalar semantics
   for each element,
2. carry the same correction through the current-baseline `*B` delegate rows,
3. add focused unit and surface-dispatch tests for the seeded spill lanes,
4. reopen the stale text-core closure records and current-gap surfaces,
5. open a bounded systematic review packet for broader function-array-support
   investigation rather than silently assuming this family is isolated.

## Validation
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib text_slice_family -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
3. live Excel COM replay on 2026-04-08 for `MID("MISSISSIPPI",SEQUENCE(11),1)`,
   `LEFT("MISSISSIPPI",SEQUENCE(3))`, and `RIGHT("MISSISSIPPI",SEQUENCE(3))`
4. 2026-04-29 replayed focused local validation:
   - `text_slice_family`: 10 passed
   - `surface_dispatch`: 75 passed

## Linked Reports
1. `BUGREP-FUNC-011`

## Evidence
1. `crates/oxfunc_core/src/functions/text_slice_family.rs`
2. `crates/oxfunc_core/src/functions/text_b_compat_family.rs`
3. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`
5. `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [x] handoff filed if required
