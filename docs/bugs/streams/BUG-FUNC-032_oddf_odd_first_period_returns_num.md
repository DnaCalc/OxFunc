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

## Root cause (diagnosed 2026-06-20)
`oddfprice_kernel` (`crates/oxfunc_core/src/functions/odd_bond_family.rs`) rejects the case via
the guard `if issue <= prev_coupon { return Err(Num) }`, where
`prev_coupon = first_coupon - 1 quasi-coupon period`. For the witness
(`first_coupon = 2021-01-01`, freq 2 → `prev_coupon = 2020-07-01`; `issue = 2020-01-01`), the
issue is **more than one quasi-coupon period before the first coupon** — a *long* odd first
coupon spanning multiple quasi-coupon periods. The current kernel only models a short odd first
coupon. `ODDFYIELD` inherits the same `#NUM!` because it inverts `oddfprice_kernel` in a solver.

## Fix (2026-06-20) — structural `#NUM!` resolved
The earlier worry that "even removing the guard would compute a wrong price" was **incorrect**:
the closed form already generalizes. With just the guard removed, the witness computes
`98.51287878857207` vs live Excel `98.51287878857208` — **1 ULP** — because
`odd_coupon_fraction = DFC/E` is simply allowed to exceed 1 (it equals the number of
quasi-coupon periods in the long odd first coupon, prorated), and the single discount factor
`base^(DSC/E)` is correct for a fractional number of periods to `first_coupon`. So the fix is:

1. **Remove the `issue <= prev_coupon` guard** — long odd first coupons now compute.
2. **Correct the period-length basis normalization** (`coupon_period_length`): the discounting
   denominator E must use the `dc` convention (`360/freq`, `365/freq`, or actual days for
   act/act) matching the bit-exact bond_core PRICE kernel, not `day_count` (which returned
   actual days for all actual bases, making bases 1/2/3 collapse to one value).

`ODDFYIELD` is fixed transitively (its solver now succeeds). The rejection test was repointed to
`odd_bond_family::tests::long_odd_first_coupon_now_computes`.

## Validation (live Excel 16.0 b20026, 13-case matrix: settlement position × basis × Nq)
- Structural: `#NUM!` gone on every case — both functions return numbers.
- 30/360 (basis 0/4): bit-exact or `~1` ULP (operation order). Actual/360 and Actual/365 moved
  much closer after the `dc` fix but are not yet bit-exact; `act/act` is the furthest residual.
- **Open numeric residual (tracked on catalog G6, not accepted):** ODDFPRICE/ODDFYIELD bit-exact
  parity for actual-day bases + the 30/360 1-ULP — a focused exactness pass (Excel's exact
  day-count conventions for these bases and operation order), companion to ACCRINT's act/act.

## Similar-Risk Scan
- `ODDLPRICE` / `ODDLYIELD` (odd-last-period) matched — not affected.
- `PRICE` / `YIELD` regular-bond price matched (`YIELD` itself fails to
  converge — separate stream BUG-FUNC-031).

## Evidence
1. `smart-fuzzer/tools/Build-TypedArgProbes.ps1`
2. ignored run artifacts under `smart-fuzzer/runs/typed-arg-001/`
3. `smart-fuzzer/planning/UNPOKED_SURFACE_COMPLETION_SWEEP_FINDINGS_2026-05-28.md` §4.1

## Closure Checklist
- [x] fix landed — structural `#NUM!` resolved (guard removed + `dc` basis normalization); numeric exactness residual tracked on catalog G6
- [x] validation recorded (13-case live-Excel matrix; regression test)
- [x] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
