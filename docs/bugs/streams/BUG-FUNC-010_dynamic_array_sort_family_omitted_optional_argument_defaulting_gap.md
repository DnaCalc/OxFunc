# BUG-FUNC-010: Dynamic-array sort family omitted optional-argument defaulting gap

## Summary
- **Bug id**: `BUG-FUNC-010`
- **Opened**: `2026-04-10`
- **Status**: `closed`
- **Owner workset**: `W083`

## Source Refs
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reproduced on ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `8234dce5f3e0c50a3c634466ead38c67fa93937e`
- **Ref notes**: intake pinned the current committed local ref on 2026-04-10.
  On that ref, `SORT({2;3;7;5},,-1)` failed locally with
  `Err(Preparation(MissingArg))` on the OxFunc surface, while the explicit
  equivalent `SORT({2;3;7;5},1,-1)` returned `{7;5;3;2}`. The local `W083`
  correction now treats omitted optional sort parameters as defaults on landed
  ref `8234dce5f3e0c50a3c634466ead38c67fa93937e` and also aligns the adjacent
  `SORTBY(..., by_array,)` lane.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: the W39 dynamic-array reshape packet pinned explicit
  `SORT` / `SORTBY` rows but never exercised the prepared-surface shape where
  an optional argument is present syntactically as an omitted slot and
  therefore arrives as `PreparedArgValue::MissingArg`.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `no`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: the intended current-baseline semantics are straightforward:
  omitted optional `SORT` / `SORTBY` controls should default exactly as though
  the argument were absent. The local implementation did not regress from a
  previously pinned correct lane; it simply never normalized
  `PreparedArgValue::MissingArg` to the same default path used for absent
  optional arguments.

## Reproduction
1. Direct local OxFunc-side replay on 2026-04-10:
   - `eval_sort_surface([array({2;3;7;5}), MissingArg, -1], NoResolver)`
   - observed: `Err(Preparation(MissingArg))`
   - worksheet-surface outcome: `#VALUE!`
2. Control replay on the same ref:
   - `eval_sort_surface([array({2;3;7;5}), 1, -1], NoResolver)`
   - observed: `{7;5;3;2}`
3. Adjacent helper replay on the same ref:
   - `SORTBY({"alpha";"beta";"gamma"},{2;3;1},)` also depends on the same
     omitted sort-order helper path.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/W39_SCENARIO_MANIFEST_SEED.csv`
  3. `docs/function-lane/W39_EXECUTION_RECORD.md`
- **Spec state at intake**: `correct but incomplete`
- **Notes**: `W39` admitted explicit `SORT` / `SORTBY` rows but did not pin the
  omitted-optional-argument defaulting lane. This bug reopens the sort-family
  packet locally until the omission/default normalization is implemented and
  validated honestly.

## Investigation Log
1. 2026-04-10: user asked to check `=SORT({2;3;7;5},,-1)` on the OxFunc side.
2. 2026-04-10: direct local replay proved the local surface returns
   `Preparation(MissingArg)` rather than sorting.
3. 2026-04-10: control replay proved explicit `sort_index=1` with the same
   descending order succeeds, narrowing the failure to omitted optional-arg
   defaulting.
4. 2026-04-10: adjacent code-path review showed `SORTBY` shares the same
   omitted sort-order helper and belongs in the same canonical stream.
5. 2026-04-10: corrected the local helper path so `MissingArg` and `EmptyCell`
   default the sort family optional controls the same way as absent arguments,
   and added focused unit coverage for both `SORT` and `SORTBY`.
6. 2026-04-12: landed the bounded W083 repair on committed ref
   `8234dce5f3e0c50a3c634466ead38c67fa93937e`, reran the focused regression
   floor on that ref, and removed the reopened `SORT` / `SORTBY` rows from
   `W051`.

## Similar-Risk Scan
### Adjacent families to check
1. `SORTBY(..., by_array,)`
2. other dynamic-array reshape functions with optional controls defaulted by
   local helper logic:
   - `TOCOL`
   - `TOROW`
   - `TAKE`
   - `DROP`
   - `EXPAND`
3. existing explicit W39 seeded rows for `SORT` / `SORTBY`

### Check method
1. direct code-path review of optional prepared-argument parsing in
   `dynamic_array_reshape_family.rs`
2. focused local replay with `CallArgValue::MissingArg`
3. comparison against explicit non-omitted control rows on the same surface

### Results
1. `SORT` was a real local bug on the reported ref:
   - omitted middle argument produced `Preparation(MissingArg)`
   - explicit `sort_index=1` succeeded
2. `SORTBY` shares the same omitted sort-order helper and is corrected by the
   same local patch.
3. `TOCOL`, `TOROW`, `TAKE`, `DROP`, and `EXPAND` also contain optional-default
   helper shapes that should be reviewed explicitly, but they are not widened
   into this bug stream without direct replay evidence.
4. the original explicit W39 seeded rows remain valid but were insufficient to
   justify overclaiming omitted-argument coverage.

### Follow-on Openings
1. `W083`

## Fix Plan
1. normalize omitted optional sort arguments on the local sort-family helper
   path
2. add focused regression tests for the exact `MissingArg` prepared-shape
3. add a replayable W39 witness row for the omitted `SORT(...,,-1)` lane
4. reconcile `W051` and workset truth so `SORT` / `SORTBY` are not overclaimed

## Validation
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib dynamic_array_reshape_family -- --nocapture`
2. focused direct local replay through a throwaway harness for:
   - `SORT({2;3;7;5},,-1)`
   - `SORT({2;3;7;5},1,-1)`
3. `powershell -ExecutionPolicy Bypass -File tools/w39-probe/run-w39-dynamic-array-reshape-baseline.ps1`

## Linked Reports
1. `BUGREP-FUNC-014`

## Evidence
1. `crates/oxfunc_core/src/functions/dynamic_array_reshape_family.rs`
2. `docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W39_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W39_EXECUTION_RECORD.md`
5. `docs/worksets/W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md`

## Closure Checklist
- [x] local fix implemented on working tree
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [x] handoff filed if required
- [x] fix landed or non-OxFunc ownership recorded
