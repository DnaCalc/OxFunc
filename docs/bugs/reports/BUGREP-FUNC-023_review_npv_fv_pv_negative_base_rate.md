# BUGREP-FUNC-023: Review finding on NPV/FV/PV negative-base rates

## Intake
- **Report id**: `BUGREP-FUNC-023`
- **Filed**: 2026-06-15
- **Source channel**: local review follow-up
- **Reporter/source**: June 2026 OxFunc cleanup review
- **Reported against ref**: `w100-w102-cleanup-pass working tree`
- **Reported against kind**: unknown
- **Reported against note**: code comments reference a live-Excel probe matrix
- **Canonical bug id**: `BUG-FUNC-038`
- **Status**: triaged

## Observed Symptom
`NPV`, `FV`, and `PV` rejected rate lanes where `1 + rate <= 0` even though
Excel admits selected negative-base cases.

## Reproduction
1. See `BUG-FUNC-038`.
2. Expected: `NPV(-2,100)` returns `-100`; integer-period `FV`/`PV` negative-base
   lanes publish numbers.
3. Actual before the working-tree patch: broad `#NUM!` rejection.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: the defect is in OxFunc financial-time-value validation.

## Links
1. `docs/bugs/streams/BUG-FUNC-038_npv_fv_pv_negative_base_rate_lanes.md`
2. `crates/oxfunc_core/src/functions/financial_time_value_family.rs`

## Triage Notes
The durable live probe artifact still needs to be linked or refreshed.
