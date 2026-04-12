# BUG-FUNC-012: TAKE and DROP omitted leading-count parity gap

## Summary
- **Bug id**: `BUG-FUNC-012`
- **Opened**: `2026-04-10`
- **Status**: `closed`
- **Owner workset**: `W085`

## Source Refs
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reproduced on ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `8234dce5f3e0c50a3c634466ead38c67fa93937e`
- **Ref notes**: live Excel replay on 2026-04-10 pinned omitted-leading-count
  semantics for both `TAKE` and `DROP`: omitting the row count keeps all rows
  while the third argument slices columns. On the reported local ref, both
  `eval_take_prepared(...)` and `eval_drop_prepared(...)` still parsed `args[1]`
  as required and therefore surfaced `Preparation(MissingArg)` instead. The
  landed `W085` correction on `8234dce5f3e0c50a3c634466ead38c67fa93937e` now
  normalizes those omitted-leading-count lanes and adds focused W39 witness
  rows.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the W39 reshaping packet seeded explicit count lanes
  for `TAKE` and `DROP` but never exercised the syntactically omitted leading
  row-count form, so the local prepared path stayed stricter than Excel.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `no`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the intended current-baseline semantics are straightforward:
  `TAKE(array,,n)` means “all rows, slice columns,” and `DROP(array,,n)` means
  “all rows, drop columns.” The local implementation did not regress from a
  previously pinned correct lane; it simply never normalized `MissingArg` in the
  leading count position to the same default path used for an absent optional
  column count.

## Reproduction
1. Live Excel replay on 2026-04-10:
   - `=ARRAYTOTEXT(TAKE({1,2;3,4},,1),1) -> {1;3}`
   - `=ARRAYTOTEXT(TAKE({1,2;3,4},,-1),1) -> {2;4}`
   - `=ARRAYTOTEXT(DROP({1,2;3,4},,1),1) -> {2;4}`
   - `=ARRAYTOTEXT(DROP({1,2;3,4},,-1),1) -> {1;3}`
2. Direct local OxFunc replay on the same ref:
   - `TAKE(array, MissingArg, 1) -> Err(Preparation(MissingArg))`
   - `DROP(array, MissingArg, 1) -> Err(Preparation(MissingArg))`
3. Handoff repro family:
   - `TAKE(w,,COLUMNS(w)/2)`
   - `TAKE(w,,-COLUMNS(w)/2)`
   - `TAKE(Z,,COLUMNS(Z)/2)`
   - `TAKE(Z,,-COLUMNS(Z)/2)`
   inside the helper-bound lambda from `HANDOFF-OXFUNC-004`.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/W39_SCENARIO_MANIFEST_SEED.csv`
  3. `docs/function-lane/W39_EXECUTION_RECORD.md`
- **Spec state at intake**: `correct but incomplete`
- **Notes**: the prior W39 packet admitted `TAKE` / `DROP` on explicit count
  rows only. This bug reopens the family locally until omitted-leading-count
  semantics are implemented and validated honestly.

## Investigation Log
1. 2026-04-10: `HANDOFF-OXFUNC-004` reported the remaining complex helper-lambda
   failure after the OxFml-side helper invocation bug was fixed.
2. 2026-04-10: local code review showed both `eval_take_prepared(...)` and
   `eval_drop_prepared(...)` parse `args[1]` as required.
3. 2026-04-10: live Excel replay pinned the intended omitted-leading-count
   semantics for both functions.
4. 2026-04-10: direct local OxFunc replay confirmed the current surface returns
   `Preparation(MissingArg)` instead of defaulting omitted row count to all
   rows.
5. 2026-04-11: normalized omitted leading row-count handling in
   `dynamic_array_reshape_family.rs` so `TAKE(...,,n)` / `DROP(...,,n)` and
   their negative-column counterparts retain all rows on the working tree.
6. 2026-04-11: added focused omission-shape assertions plus replayable W39
   witness rows for the bounded `TAKE` / `DROP` omitted-leading-count forms,
   and reran the focused local test lane plus `scripts/check-worksets.ps1`.
7. 2026-04-12: landed the bounded W085 repair on committed ref
   `8234dce5f3e0c50a3c634466ead38c67fa93937e`, reran the focused regression
   floor on that ref, and removed the reopened `TAKE` / `DROP` rows from W39
   overclaim and `W051`.

## Similar-Risk Scan
### Adjacent families to check
1. `TAKE(...,,n)` and `TAKE(...,,-n)`
2. `DROP(...,,n)` and `DROP(...,,-n)`
3. other reshape-family functions with omitted-leading optional controls:
   - `EXPAND`
   - `TOCOL`
   - `TOROW`
4. prior W39 seeded explicit count rows for `TAKE` / `DROP`

### Check method
1. direct live Excel replay on simple matrices
2. direct local OxFunc replay with `CallArgValue::MissingArg`
3. local helper/path review of prepared argument parsing

### Results
1. `TAKE` is a real local bug on the reported ref.
2. `DROP` shares the same omission/defaulting bug shape and belongs in the
   same canonical stream.
3. the prior explicit W39 seeded rows remain valid but were insufficient to
   justify overclaiming omitted-leading-count coverage.
4. broader reshape-family widening is not opened without direct replay evidence.

### Follow-on Openings
1. `W085`

## Fix Plan
1. normalize omitted leading row-count arguments for `TAKE` and `DROP` so
   `MissingArg` means “all rows” when a third column-count argument is present
2. add focused regression tests for:
   - `TAKE(...,,n)`
   - `TAKE(...,,-n)`
   - `DROP(...,,n)`
   - `DROP(...,,-n)`
3. add replayable W39 witness rows for the same omitted-leading-count forms
4. reconcile W39 and W51 truth so `TAKE` / `DROP` are not overclaimed

## Validation
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib dynamic_array_reshape_family -- --nocapture`
2. focused live Excel replay for omitted-leading-count `TAKE` / `DROP`
3. `powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`

## Linked Reports
1. `BUGREP-FUNC-016`

## Evidence
1. `crates/oxfunc_core/src/functions/dynamic_array_reshape_family.rs`
2. `docs/worksets/W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md`
3. `docs/function-lane/W39_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W39_EXECUTION_RECORD.md`
5. `docs/function-lane/W39_SCOPE_RECONCILIATION.csv`
6. `../OxFml/docs/handoffs/HANDOFF-OXFUNC-004_TAKE_OMITTED_ROW_COUNT_IN_COMPLEX_LAMBDA.md`

## Closure Checklist
- [x] local fix implemented
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [x] handoff filed if required
- [x] fix landed or non-OxFunc ownership recorded
