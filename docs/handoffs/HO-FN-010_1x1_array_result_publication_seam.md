# HO-FN-010 - 1x1 array result publication seam

## 1. Direction
- **From**: `OxFunc`
- **To**: `OxFml`
- **Filed date**: `2026-05-02`
- **Source workset**: `W089`
- **Related bugs**: `BUG-FUNC-026`, `BUG-FUNC-023`

## 2. Purpose
Record that direct OxFunc function results and final worksheet-cell publication
must not be compared as the same semantic surface for `1x1` array-returning
functions. OxFunc should preserve the internal array result; OxFml/DNA Calc
should own any final result-completion or comparator projection that makes an
anchor cell display as a scalar.

## 3. Evidence
Excel `16.0` build `19929`, workbook Compatibility Version `2`:

1. `=TAKE({1,2;3,4},1,1)` publishes worksheet value `1`.
2. `=TYPE(TAKE({1,2;3,4},1,1))` returns `64`.
3. `=ROWS(TAKE({1,2;3,4},1,1))` returns `1`.
4. `=COLUMNS(TAKE({1,2;3,4},1,1))` returns `1`.
5. `=HSTACK(TAKE({1,2;3,4},1,1),9)` spills `{1,9}`.
6. Adjacent probes classify `CHOOSECOLS({1},1)`, `MMULT(5,2)`, and
   `MINVERSE(5)` as arrays under nested `TYPE`.

## 4. OxFunc Local Changes
OxFunc has undone the mislocalized function-level scalarization repair and now
preserves internal `1x1` arrays for these result constructors:

1. `dynamic_array_reshape_family::build_array` for `TAKE` and adjacent
   dynamic-array reshapers,
2. `matrix_family::value_from_matrix` for `MINVERSE`, `MMULT`, and `MUNIT`,
3. `trimrange_fn::trimrange_kernel` for single-cell trimmed dynamic-array
   results.

## 5. Validation Run
Local OxFunc validation after the undo:

1. `cargo test -p oxfunc_core dynamic_array_reshape_family --lib`
2. `cargo test -p oxfunc_core matrix_family --lib`
3. `cargo test -p oxfunc_core trimrange --lib`
4. `cargo test -p oxfunc_core eval_surface_value_call_ftc_100 --lib`
5. `cargo test -p oxfunc_core --lib`

## 6. OxFml Ask
1. Identify the correct OxFml/DNA Calc publication or result-completion layer
   for final worksheet-cell scalar projection of `1x1` array results.
2. Keep nested formula evaluation array-aware so probes such as
   `TYPE(TAKE(...))`, `ROWS(TAKE(...))`, and `HSTACK(TAKE(...),9)` continue to
   observe array semantics.
3. Update smart-fuzzer comparison mode so direct OxFunc calls are not marked as
   function mismatches when the only difference is Excel final-cell publication.

W092 interim comparator note: OxFunc's local smart-fuzzer runner now classifies
matching direct `1x1` array vs Excel final-cell scalar rows as
`adapter_or_seam_mismatch`, preserving the handoff while preventing duplicate
function-bug pressure in long-run fuzzing. OxFml/DNA Calc still owns the actual
publication/result-completion behavior. W092 replay
`smart-fuzzer/runs/w092-scenario-math-cycle-001/` records the adjacent
`=MINVERSE(5)` and `=MMULT(5,2)` rows under that seam classification.

## 7. Affected OxFunc Truth Surfaces
1. `docs/bugs/streams/BUG-FUNC-026_take_1x1_scalar_publication_mismatch.md`
2. `docs/bugs/streams/BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md`
3. `docs/function-lane/FUNCTION_SLICE_MATH_MATRIX_AND_ROUNDING_FAMILY_CONTRACT_PRELIM.md`
4. `smart-fuzzer/planning/AXIS_WITNESS_SWEEP_RUN_PLAN.md`
