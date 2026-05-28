# BUG-FUNC-032: ODDFPRICE / ODDFYIELD (odd first period) return #NUM!

## Summary
- **Bug id**: `BUG-FUNC-032`
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
- **Root cause class**: `odd_first_period_handling`
- **Root cause summary**: the odd-**first**-period bond functions
  `ODDFPRICE` and `ODDFYIELD` return `#NUM!` for inputs where Excel
  computes a value. The odd-**last**-period siblings `ODDLPRICE` and
  `ODDLYIELD` matched Excel bit-exactly on equivalent inputs in the same
  run, so the defect is specific to the odd-first-period quasi-coupon
  computation (the `first_coupon` / issue handling), not the odd-period
  bond machinery in general.

## Reproduction
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-TypedArgProbes.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId typed-arg-001 `
  -CaseSetPath smart-fuzzer\cache\typed-arg-probes-v0.json
```

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ODDFPRICE(44013,44562,43831,44197,0.05,0.06,100,2,0)` | `#NUM!` | `number` (`0x4058a0d3018deeab`) |
| `=ODDFYIELD(44013,44562,43831,44197,0.05,95,100,2,0)` | `#NUM!` | `number` (`0x3fb5e2057522e596`) |

Inputs (odd first period): settlement `44013`, maturity `44562`, issue
`43831`, first_coupon `44197`, rate/coupon `0.05`, yld/pr `0.06`/`95`,
redemption `100`, frequency `2`, basis `0`. Ordering
issue < settlement < first_coupon < maturity holds.

## Fix
Not yet fixed. Repair direction: match Excel for well-posed odd-first
inputs. Audit the odd-first-period quasi-coupon / accrual computation in
`crates/oxfunc_core/src/functions/` (odd-bond family) — likely a
date-fraction or quasi-coupon-count edge that trips the `#NUM!` guard.

## Validation
Pending repair. Re-run `typed-arg-001` and show `ODDFPRICE` / `ODDFYIELD`
moving to `exact_typed_bit_match`.

## Similar-Risk Scan
- `ODDLPRICE` / `ODDLYIELD` (odd-last-period) matched — not affected.
- `PRICE` / `YIELD` regular-bond price matched (`YIELD` itself fails to
  converge — separate stream BUG-FUNC-031).

## Evidence
1. `smart-fuzzer/tools/Build-TypedArgProbes.ps1`
2. ignored run artifacts under `smart-fuzzer/runs/typed-arg-001/`
3. `smart-fuzzer/planning/UNPOKED_SURFACE_COMPLETION_SWEEP_FINDINGS_2026-05-28.md` §4.1

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
