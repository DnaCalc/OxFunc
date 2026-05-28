# BUG-FUNC-029: Unary-plus operator over-coerces text and logical

## Summary
- **Bug id**: `BUG-FUNC-029`
- **Opened**: `2026-05-28`
- **Status**: `fixed` (2026-05-28)
- **Owner workset**: `W074` (operator broadcast/semantics family)

## Source Refs
- **Reported against ref**: working tree at run `operator-structural-sweep-001`
- **Reproduced on ref**: run `operator-structural-sweep-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: live Excel COM, Excel `16.0` build `20026`, workbook
  Compatibility Version `2`, exact typed equality.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `spec_mismatch`
- **Root cause summary**: Excel's unary-plus operator (`+x`) is a
  type-preserving identity — it returns the operand unchanged, including
  text and logical operands. OxFunc's `OP_UNARY_PLUS` coerces the operand
  to a number. Unary *minus* (`OP_NEGATE`) does coerce-and-negate; unary
  plus must not coerce.

## Reproduction
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-OperatorStructuralProbes.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId operator-structural-sweep-001 `
  -CaseSetPath smart-fuzzer\cache\operator-structural-probes-v0.json
```

Run `operator-structural-sweep-001`: `99` cases over `15` scalar/value
operators, `97` exact typed bit matches, `2` structural mismatches — both
`OP_UNARY_PLUS`:

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=+"2"` | `number:2` | `text:2` |
| `=+TRUE` | `number:1` | `logical:TRUE` |

The other `14` scalar/value operators (`OP_ADD`, `OP_SUBTRACT`,
`OP_MULTIPLY`, `OP_DIVIDE`, `OP_POWER`, `OP_CONCAT`, the six comparisons,
`OP_NEGATE`, `OP_PERCENT`) matched bit-exactly across baseline, array-lift,
broadcast, error, text-coercion, and logical probes.

## Fix
Fixed. `eval_op_unary_plus_surface` no longer routes through the coercing
`eval_unary_numeric_surface`; it is now a dedicated type-preserving identity:
text→text, logical→logical, number→underflow-normalized number, error
propagated, blank/empty→0, arrays mapped elementwise with the same rules.
`crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`. Operand
semantics across number/text/logical/error/array/blank were first measured
empirically against Excel (`unary-plus-operand-001`) before coding.

## Validation
- Rust unit tests `unary_plus_is_type_preserving_identity` and
  `unary_plus_maps_arrays_elementwise_preserving_type` (and the retained
  `negate_follows_numeric_coercion`); full `oxfunc_core` lib suite green
  (`1314 passed`).
- Excel differential: re-ran `operator-structural-sweep-002` — **99/99
  `exact_typed_bit_match`**; the two prior `OP_UNARY_PLUS` mismatches now
  match (`=+"2"`→`text:2`, `=+TRUE`→`logical:TRUE`).

## Similar-Risk Scan
- `OP_NEGATE` and `OP_PERCENT` were checked in the same run and match —
  their coercion is correct (they are not identity operators).
- The reference operators (`OP_RANGE_REF`, `OP_INTERSECTION_REF`,
  `OP_UNION_REF`, `OP_SPILL_REF`, `OP_TRIM_REF_*`,
  `OP_IMPLICIT_INTERSECTION`) are not covered by this run; they need a
  reference-fixture generator.

## Evidence
1. `smart-fuzzer/tools/Build-OperatorStructuralProbes.ps1`
2. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
3. ignored run artifacts under `smart-fuzzer/runs/operator-structural-sweep-001/`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
