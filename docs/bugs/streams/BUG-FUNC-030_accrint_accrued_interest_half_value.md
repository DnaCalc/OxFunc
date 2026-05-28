# BUG-FUNC-030: ACCRINT returns half of Excel's accrued interest

## Summary
- **Bug id**: `BUG-FUNC-030`
- **Opened**: `2026-05-28`
- **Status**: `open`
- **Owner workset**: `W090` (smart-fuzzer un-poked completion sweep)

## Source Refs
- **Reported against ref**: working tree at run `typed-arg-001`
- **Reproduced on ref**: runs `typed-arg-001`, `typed-arg-002`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: live Excel COM, Excel `16.0` build `20026`, workbook
  Compatibility Version `2`, exact typed equality.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `computation_defect`
- **Root cause summary**: `ACCRINT` returns exactly half of Excel's accrued
  interest, consistent with an erroneous divide-by-frequency (or a missing
  multiply-by-period-count) in the accrual-from-issue path. This is a value
  defect, not a ULP drift. The maturity-form sibling `ACCRINTM` matched
  Excel bit-exactly on equivalent inputs, so the defect is specific to the
  periodic `ACCRINT` accrual.

## Reproduction
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-TypedArgProbes.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId typed-arg-001 `
  -CaseSetPath smart-fuzzer\cache\typed-arg-probes-v0.json
```

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ACCRINT(43831,44197,44013,0.05,1000,2,0)` | `number:12.5` (`0x4029000000000000`) | `number:25` (`0x4039000000000000`) |

Inputs: issue `43831` (2020-01-01), first_interest `44197`, settlement
`44013` (2020-07-01), rate `0.05`, par `1000`, frequency `2`, basis `0`.
Half-year accrual at 5% on par 1000 ≈ `25` (Excel). OxFunc returns `12.5`.

## Fix
Not yet fixed. Repair direction: match Excel's accrued-interest value.
Audit the frequency handling in the `ACCRINT` accrual path
(`crates/oxfunc_core/src/functions/bond_core_family.rs`).

## Validation
Pending repair. Re-run `typed-arg-001` and show the `ACCRINT` row moving to
`exact_typed_bit_match`.

## Similar-Risk Scan
- `ACCRINTM` matched on equivalent inputs (not affected).
- Other coupon-period functions (`COUPDAYBS`, `COUPDAYS`, `COUPDAYSNC`,
  `COUPNCD`, `COUPNUM`, `COUPPCD`) matched bit-exactly in the same run, so
  the period/frequency machinery they share is not uniformly broken — the
  defect is local to `ACCRINT`.

## Evidence
1. `smart-fuzzer/tools/Build-TypedArgProbes.ps1`
2. ignored run artifacts under `smart-fuzzer/runs/typed-arg-001/`
3. `smart-fuzzer/planning/UNPOKED_SURFACE_COMPLETION_SWEEP_FINDINGS_2026-05-28.md` §4.2

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
