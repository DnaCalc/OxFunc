# BUG-FUNC-031: YIELD returns #NUM! where Excel converges

## Summary
- **Bug id**: `BUG-FUNC-031`
- **Opened**: `2026-05-28`
- **Status**: `validated_local` (structural `#NUM!` fixed 2026-06-20; `~19` ULP numeric residual)
- **Owner workset**: `W090` (smart-fuzzer un-poked completion sweep)

## Fix (2026-06-20)
Root cause: `yield_kernel`'s `p.n > 1` branch solved over `price_kernel`, which validates
its yield argument with `rate(yld)` — and `rate()` rejects **any negative value** with `#NUM!`.
The root-finder (`solve`) brackets candidate yields down to `-frequency` and bisects through
negative yields, so the very first negative midpoint made `price_kernel` error and the whole
`YIELD` returned `#NUM!`. (`PRICE` matched bit-exact because it is only ever called at the
user's single non-negative yield.)

Repair (`crates/oxfunc_core/src/functions/bond_core_family.rs`): the solver now evaluates the
price via `pcomp` directly — `pcomp`'s own guards (`yld <= -frequency`, `base <= 0`) keep the
domain correct while admitting the negative candidate yields the bisection must probe. The
`solve` low-endpoint guard also treats an un-evaluable endpoint as `+∞` price, and the
convergence tolerance was tightened (`1e-12 → 1e-15` on price) to let the bracket collapse to
the true root.

Result vs live Excel 16.0 b20026: `=YIELD(44013,44562,0.05,95,100,2,0)` →
`0.0862487399523155` (`0x3fb61465bd6a9970`) vs Excel `0x3fb61465bd6a9983` — a number, not
`#NUM!`. The structural defect is resolved; a `~19` ULP residual remains (bisection vs Excel's
own solver), reclassified to the catalog G6 YIELD NUM-L lane. Regression test:
`bond_core_family::tests::yield_converges_for_well_posed_multi_period_discount_bond`. Full lib
suite green (1418).

## Source Refs
- **Reported against ref**: working tree at run `typed-arg-001`
- **Reproduced on ref**: runs `typed-arg-001`, `typed-arg-002`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: live Excel COM, Excel `16.0` build `20026`, workbook
  Compatibility Version `2`, exact typed equality.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `solver_non_convergence`
- **Root cause summary**: `YIELD` returns `#NUM!` for a well-posed bond
  (price below redemption, positive coupon, semiannual frequency) where
  Excel converges to a yield. The price-form inverse, `PRICE`, matched
  Excel **bit-exactly on the same dates and parameters** in the same run,
  so the cashflow model and date handling are sound; the defect is in the
  `YIELD` root-finder (iteration bounds / seed / convergence test), not the
  inputs.

## Reproduction
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-TypedArgProbes.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId typed-arg-001 `
  -CaseSetPath smart-fuzzer\cache\typed-arg-probes-v0.json
```

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=YIELD(44013,44562,0.05,95,100,2,0)` | `#NUM!` | `number ≈ 0.0857` (`0x3fb61465bd6a9983`) |
| `=PRICE(44013,44562,0.05,0.06,100,2,0)` | matched Excel bit-exactly | (cross-check: inverse converges) |

Inputs: settlement `44013`, maturity `44562`, coupon `0.05`, price `95`,
redemption `100`, frequency `2`, basis `0`.

## Fix
Not yet fixed. Repair direction: make the `YIELD` solver converge for
well-posed inputs and match Excel's yield. Audit the root-finder in
`crates/oxfunc_core/src/functions/bond_core_family.rs` (bracketing, seed,
max-iterations, tolerance / failure-to-`#NUM!` path).

## Validation
Pending repair. Re-run `typed-arg-001` and show the `YIELD` row moving to
`exact_typed_bit_match`.

## Similar-Risk Scan
- `PRICE` (the value inverse) matched — not affected.
- Other yield solvers in the same run: `YIELDDISC` and `YIELDMAT` returned
  values (with small numeric drift, BUG-FUNC pending) rather than `#NUM!`,
  so the non-convergence is specific to the coupon-bond `YIELD` path.
  `ODDFYIELD` also returns `#NUM!` but via the odd-first-period path
  (BUG-FUNC-032).

## Evidence
1. `smart-fuzzer/tools/Build-TypedArgProbes.ps1`
2. ignored run artifacts under `smart-fuzzer/runs/typed-arg-001/`
3. `smart-fuzzer/planning/UNPOKED_SURFACE_COMPLETION_SWEEP_FINDINGS_2026-05-28.md` §4.1

## Closure Checklist
- [x] fix landed (structural; `~19` ULP residual on the catalog G6 NUM-L lane)
- [x] validation recorded (live Excel b20026 + regression test + 1418 lib tests)
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
