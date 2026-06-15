# BUGREP-FUNC-020: Review finding on IPMT/PPMT type=1 interest

## Intake
- **Report id**: `BUGREP-FUNC-020`
- **Filed**: 2026-06-15
- **Source channel**: local review follow-up
- **Reporter/source**: June 2026 OxFunc cleanup review
- **Reported against ref**: `w100-w102-cleanup-pass working tree`
- **Reported against kind**: unknown
- **Reported against note**: review digest and adversarial verification summary
- **Canonical bug id**: `BUG-FUNC-034`
- **Status**: triaged

## Observed Symptom
For `type=1` and periods after the first period, IPMT/PPMT and cumulative
finance variants omitted the beginning-of-period payment in the interest
accrual calculation.

## Reproduction
1. See `BUG-FUNC-034`.
2. Expected: annuity-due interest for period `per >= 2` subtracts the beginning
   payment before multiplying by rate.
3. Actual before the working-tree patch: type-1 interest followed the previous
   period ordinary-interest lane.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: the defect is in OxFunc financial kernels.

## Links
1. `docs/bugs/streams/BUG-FUNC-034_ipmt_ppmt_type_one_interest_omits_beginning_payment.md`
2. `crates/oxfunc_core/src/functions/financial_time_value_family.rs`
3. `crates/oxfunc_core/src/functions/cumulative_finance_family.rs`

## Triage Notes
W102A working-tree tests exist; live Excel bit pinning remains open.
