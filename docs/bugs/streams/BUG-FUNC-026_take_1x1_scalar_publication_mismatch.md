# BUG-FUNC-026: TAKE 1x1 scalar publication mismatch

## Summary
- **Bug id**: `BUG-FUNC-026`
- **Opened**: `2026-05-02`
- **Status**: `handed_off`
- **Owner workset**: `W089`
- **Bead**: `oxf-vkg8.1`

## Source Refs
- **Reported against ref**: `w089-axis-witness-20260501-002`
- **Reproduced on ref**: `w089-axis-witness-20260501-002`; Excel nested `TYPE` probe
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `HO-FN-010 filed`

## Ownership And Root Cause
- **Ownership class**: `seam-owned follow-up`
- **Root cause class**: `shape_publication_gap`
- **Root cause summary**: the original comparator compared direct OxFunc
  function output against Excel worksheet-cell publication. Follow-up Excel
  probes show the distinction matters: `TYPE(TAKE({1,2;3,4},1,1))` returns
  `64`, so Excel still exposes the nested `TAKE` result as an array even though
  the anchor cell publishes the single value `1`.

## Reproduction
Run:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w089-axis-witness-20260501-002 `
  -CaseSetPath smart-fuzzer\cache\axis-witness-case-set-v0.json
```

Representative row:

1. `=TAKE({1,2;3,4},1,1)`: local
   `array:1x1:[number:0x3ff0000000000000]`, Excel
   `number:0x3ff0000000000000`.

## Repair Outcome
Superseded on `2026-05-02`: the earlier function-level scalarization repair
was wrong for nested formula semantics and has been undone. `TAKE` now preserves
the generated `1x1` `EvalValue::Array`; worksheet-cell scalar publication must
be modeled above OxFunc, in the OxFml/DNA Calc result-completion or comparison
seam.

Evidence that forced reclassification:

1. `=TAKE({1,2;3,4},1,1)` publishes worksheet value `1`.
2. `=TYPE(TAKE({1,2;3,4},1,1))` returns `64`.
3. `=ROWS(TAKE({1,2;3,4},1,1))` returns `1`.
4. `=COLUMNS(TAKE({1,2;3,4},1,1))` returns `1`.
5. `=HSTACK(TAKE({1,2;3,4},1,1),9)` spills `{1,9}`.

Similar-risk repair scan:

1. `dynamic_array_reshape_family::build_array`: scalarization removed.
2. `matrix_family::value_from_matrix`: computed `1x1` matrix scalarization
   removed for `MINVERSE`, `MMULT`, and `MUNIT` array-return results.
3. `trimrange_fn::trimrange_kernel`: single-cell trimmed result scalarization
   removed to match the local TRIMRANGE contract's dynamic-array result model.
4. Other `1x1` checks found in the scan were input coercion, vector-orientation,
   or scalar-return-function logic and were not changed without function-specific
   evidence.

Validation after undoing the mislocalized scalarization:

1. `cargo test -p oxfunc_core dynamic_array_reshape_family --lib`
2. `cargo test -p oxfunc_core matrix_family --lib`
3. `cargo test -p oxfunc_core trimrange --lib`
4. `cargo test -p oxfunc_core eval_surface_value_call_ftc_100 --lib`
5. `cargo test -p oxfunc_core --lib`

## Evidence
1. `smart-fuzzer/runs/w089-axis-witness-20260501-002/`
2. `smart-fuzzer/runs/w089-axis-witness-take-repair-20260502-001/`
   (superseded comparator result; retained as evidence of the mislocalized
   repair)
3. Handoff: `docs/handoffs/HO-FN-010_1x1_array_result_publication_seam.md`
4. Bead: `oxf-vkg8.1`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded: non-OxFunc seam follow-up
  recorded; function-level repair undone
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required: `HO-FN-010`
