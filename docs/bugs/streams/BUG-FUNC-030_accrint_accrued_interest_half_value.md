# BUG-FUNC-030: ACCRINT returns half of Excel's accrued interest

## Summary
- **Bug id**: `BUG-FUNC-030`
- **Opened**: `2026-05-28`
- **Status**: `validated_local` (half-value defect fixed 2026-06-20; narrow `act/act` exactness residual tracked on the catalog ACCRINT row)
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
- **Root cause summary** (diagnosed 2026-05-28): the bug is in the
  `settlement <= first_interest` (odd-first-stub) branch of `accrint_kernel`
  (`crates/oxfunc_core/src/functions/bond_core_family.rs`). It returns
  `coup * dd(issue, settlement) / dd(issue, first)` with `coup = par*rate/freq`.
  The denominator `dd(issue, first)` is the **entire** issue→first-interest
  span, which can be **more than one** quasi-coupon period. For the witness
  (issue 2020-01-01, first 2021-01-01, freq 2 → a 1-year, 2-period stub;
  settlement 2020-07-01) this gives `25 * 180/360 = 12.5`, whereas Excel sums
  over quasi-coupon periods: settlement is exactly one full quasi-period after
  issue, so `par*(rate/freq)*1 = 25`. The single linear interpolation is only
  correct when issue→first is exactly one period; it mishandles multi-quasi-
  coupon-period first stubs.
- **Correct algorithm**: MS ACCRINT first-stub formula
  `par * (rate/freq) * Σ_i (A_i / NL_i)` over the quasi-coupon periods (defined
  backward from `first_interest` by `12/freq` months) that the issue→settlement
  span touches, with day-counts per `basis`.
- **Lane (re-triaged 2026-05-28)**: `needs-analysis`, not the localized
  code-fix originally assumed. Requires implementing the quasi-coupon-period
  summation and verifying against an Excel matrix across basis conventions and
  1-period vs multi-period stubs (and the partial-end-period case). `ACCRINTM`
  matched, so the defect is specific to periodic `ACCRINT`.

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

## Fix (2026-06-20)
Landed. `accrint_kernel` was rewritten from the single issue->first linear
interpolation to the MS quasi-coupon-period summation
(`crates/oxfunc_core/src/functions/bond_core_family.rs`): the accrual span is
walked over quasi-coupon periods (each `12/freq` months, anchored on
first_interest via `addm(first, k*m)` so end-of-month clamping never drifts); a
full period contributes one coupon, a partial period contributes
`accrued_days / normal_length`, and the whole coupon-fraction sum is scaled by
`par·rate/freq` once. `calc_method` was corrected to Excel's empirical
behaviour — TRUE accrues from issue, FALSE from one quasi-coupon period before
first_interest (signed, so a settlement before that start is negative).

## Validation (live Excel 16.0 build 20026, 2026-06-20)
A 15-case matrix over both branches, all five bases, freq 1/2/4, calc TRUE/FALSE,
end-of-month dates, and 1-period vs multi-period stubs:

- `13/15` exact typed bit matches, including the reported witness
  `=ACCRINT(43831,44197,44013,0.05,1000,2,0)` → `25` (was `12.5`); the
  multi-period stub forward case (old kernel under-counted by 90); and
  `calc_method=FALSE` (verified against a 6-case calc-method battery).
- `2/15` residual: `S5` `act/360` is 1 ULP (`137.5` vs Excel `137.50000000000003`);
  `S4` `act/act` (basis 1/3) is `~0.07%` off because Excel's normal-period-length
  for a *later* coupon period in a multi-coupon span deviates from the actual
  period length when that period crosses a leap February (isolated single
  periods use the actual length and match). This is a distinct, pre-existing
  `act/act` convention residual (the old kernel was equally off there), now
  tracked on the catalog ACCRINT row; not the half-value defect this stream opened.

Regression: `bond_core_family::tests::accrint_slices` updated to pin the witness
(`25`), `calc TRUE == FALSE` for a regular first coupon, and `TRUE > FALSE` for a
long first coupon. Full `oxfunc_core` lib suite green (1417 passed).

## Fix (2026-06-21) — act/act leap-February residual closed
The forward per-period loop measured each leap-crossing period by its *own* actual length
(182 days), but Excel normalises the settlement-side fraction by the **canonical** last
coupon period length `CoupDays(first - 1 period, first)` (184 days) — a single length, so a
leap-crossing period is never measured by its actual length. `accrint_kernel` was rewritten
as a faithful port of ExcelFinancialFunctions `accrInt` (bonds.fs):
- **settlement ≤ first** (odd first coupon): backward from `pcd = first - 1 period`, whole
  periods counting as `int(calc_method)`, settlement tail normalised by the canonical length,
  the issue period by its own length.
- **settlement > first** (a regime F#'s public API rejects, but Excel computes): a forward
  accrual from the accrual start, whole periods = 1 and the final partial by the canonical
  length. OxFunc now matches Excel here where F# throws.

Helpers ported: `change_month_flag`, `find_pcd_ncd_accr`, `diff360_us` (both 30/360 modes),
`days_between_num`/`days_between_denum`, `actual_coup_days_accr`/`coup_days_accr`.

## Validation (live Excel 16.0 b20026, G6 three-way harness, 2026-06-21)
Bit-exact across a **24-case sweep**: all five bases; leap-crossing act/act, act/365, act/360
partials; settlement before *and* after first_interest; quarterly/annual/semiannual; EOM
dates; deep multi-period; issue mid-period. **Residual: 1 ULP** on a single `us30360`
triple-edge (issue mid-period AND settlement past first_interest) — an operation-order artifact
(constant-length bases want sum-then-divide), reclassified NUM-S on the catalog G6 row; not
accepted. Regression `bond_core_family::tests::accrint_leap_february_and_settlement_after_first_bit_exact`
pins the act/act, act/365 leap partials and the settlement-after-first case.

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
- [x] fix landed or non-OxFunc ownership recorded (half-value defect, 2026-06-20)
- [x] validation recorded (15-case Excel matrix + 6-case calc-method battery)
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required (catalog row reclassified to the act/act residual)
- [ ] handoff filed if required
