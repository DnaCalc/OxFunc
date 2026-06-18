# BUG-FUNC-034: IPMT/PPMT/CUMIPMT/CUMPRINC type=1 interest omits the beginning-of-period payment (per>=2)

## Summary
- **Bug id**: `BUG-FUNC-034`
- **Opened**: `2026-06-11`
- **Status**: `closed` (fix landed in `7a0003f` on main; live Excel 16.0 b20026 verified 2026-06-18)
- **Owner workset**: `W102A` (structural bug-fix batch, 102-A item 1)

## Live Excel Verification (2026-06-18)
OxFunc local value surface vs live Excel 16.0 build 20026, witnesses rate=0.1
nper=3 pv=1000 type=1 per=2; all match to 10 decimal places:
`IPMT(0.1,2,3,1000,0,1)` = `-63.4441087613`,
`PPMT(0.1,2,3,1000,0,1)` = `-302.1148036254`,
`CUMIPMT(0.1,3,1000,2,2,1)` = `-63.4441087613`,
`CUMPRINC(0.1,3,1000,2,2,1)` = `-302.1148036254`.

## Source Refs
- **Reported against ref**: `w100-w102-cleanup-pass` working tree
- **Reproduced on ref**: review digest `functions-numeric.md` F1 (VERDICT isReal=True conf=high),
  cross-checked by the adversarial verifier
- **Introduced in ref**: `unknown` (predates the current annuity kernels)
- **Fixed in ref**: `not yet fixed` (working-tree patch present; checkpoint not landed)
- **Ref notes**: closed-form witnesses derived from the candidate formula and
  the standard annuity-due identity; live-Excel bit pins to follow on the probe lane.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `computation_defect`
- **Root cause summary** (diagnosed 2026-06-10): for `BeginningOfPeriod` timing
  (type=1) and `period_index >= 2`, both IPMT implementations computed
  `FV(rate, per-2, pmt, pv, Begin) * rate`, which omits the beginning-of-period
  payment made at the start of the accrual period. Because
  `FV(..., pmt_due, Begin) == FV(..., pmt_ordinary, End)`, the buggy form returns
  `ipmt_type0(per-1)` instead of Excel's `ipmt_type0(per)/(1+rate)`. The per-period
  error equals `pmt * rate`. PPMT inherits the defect via `payment - interest`, and
  the identical defect existed in `cumulative_finance_family.rs ipmt_from_payment`,
  so CUMIPMT/CUMPRINC type=1 were wrong for any range touching period >= 2.
- **Correct algorithm**: `(FV(rate, per-2, pmt, pv, Begin) - pmt) * rate` for
  `per >= 2`; the `per <= 1 -> 0.0` branch was already correct and is preserved.
- **Defect sites**:
  - `crates/oxfunc_core/src/functions/financial_time_value_family.rs` `ipmt()`
    BeginningOfPeriod branch.
  - `crates/oxfunc_core/src/functions/cumulative_finance_family.rs`
    `ipmt_from_payment()` BeginningOfPeriod branch.

## Reproduction
```text
rate=0.1, nper=3, pv=1000, type=1, per=2
  Excel IPMT  = -63.44414...  (= ipmt_type0(2)/(1+rate) = -69.78855.../1.1)
  OxFunc (pre-fix) = -100.0    (= -pv*rate, i.e. ipmt_type0(1))
```

| Formula | OxFunc (pre-fix) | Excel |
| --- | --- | --- |
| `=IPMT(0.1,2,3,1000,0,1)` | `-100.0` | `-63.44410876132934` (verified analytically) |

The pre-fix path returns the previous period's ordinary interest. No test
covered type=1 beyond per=1 (the only type-1 tests asserted the per=1
zero-interest case).

## Fix Plan
Subtract the beginning-of-period payment before multiplying by rate in both
kernels:
`(FV(rate, per-2, pmt, pv, Begin) - pmt) * rate`. Keep the `per <= 1 -> 0.0`
branch. Add type=1 witnesses for per in {1,2,3,nper} on IPMT/PPMT and CUMIPMT/
CUMPRINC range witnesses touching per>=2. Do not disturb the type=0 paths — the
CUMIPMT/CUMPRINC type=0 witnesses pin exact Excel bits.

## Validation
- `cargo test -p oxfunc_core financial_time_value` — 24 passed.
- `cargo test -p oxfunc_core cumulative_finance` — 10 passed.
- New tests:
  - `financial_time_value_family.rs::ipmt_ppmt_type_one_witness_candidates_pending_excel_bit_pinning`
    (closed-form numeric pins + the `ipmt_type1 == ipmt_type0/(1+rate)` identity).
  - `financial_time_value_family.rs::ipmt_type_one_per_equal_one_is_zero_interest`.
  - `cumulative_finance_family.rs::type_one_range_witness_candidates_pending_excel_bit_pinning`.
  - `cumulative_finance_family.rs::type_one_interest_and_principal_partition_total_payment`.
- Existing type=0 Excel-bit witnesses
  (`cumipmt_and_cumprinc_exactness_witness_rows_match_excel_targets`,
  `type_one_has_zero_interest_in_first_period`) unchanged and still pass.
- **Pending**: live-Excel bit pinning of the type=1 witnesses on the probe lane
  (numeric closed-form pins use a tight tolerance; the exact f64 publication bits
  remain to be pinned).

## Similar-Risk Scan
- The type=0 IPMT/PPMT/CUMIPMT/CUMPRINC paths were already correct and were not
  touched; their Excel-bit witnesses still pass.
- The two annuity-due IPMT kernels were the only sites with the missing `- pmt`
  adjustment; the current working-tree patch updates both sites.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded (working-tree patch present; checkpoint not landed)
- [x] validation recorded (unit tests)
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] type=1 witnesses pinned to live-Excel bits (probe lane follow-up)
- [ ] KED row registered if a residual remains after bit-pinning
- [x] linked reports updated
