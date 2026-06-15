# BUGREP-FUNC-022: Review finding on XNPV sign-change validation

## Intake
- **Report id**: `BUGREP-FUNC-022`
- **Filed**: 2026-06-15
- **Source channel**: local review follow-up
- **Reporter/source**: June 2026 OxFunc cleanup review
- **Reported against ref**: `caffd0f`
- **Reported against kind**: commit
- **Reported against note**: stream note records live Excel 16.0 build 20026 verification
- **Canonical bug id**: `BUG-FUNC-037`
- **Status**: triaged

## Observed Symptom
`XNPV` reused XIRR-style sign-change and two-cashflow validation even though
XNPV is direct discounting and Excel accepts one-sided or single-element series.

## Reproduction
1. See `BUG-FUNC-037`.
2. Expected: `XNPV(0.1,{100,200},{45000,45100})` returns a number.
3. Actual before the working-tree patch: `#NUM!`.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: the defect is in OxFunc cashflow validation sharing.

## Links
1. `docs/bugs/streams/BUG-FUNC-037_xnpv_wrongly_inherits_sign_change_gate.md`
2. `crates/oxfunc_core/src/functions/cashflow_rate_family.rs`

## Triage Notes
W102A working-tree tests cover all-positive, single-element, all-negative, and
XIRR contrast lanes.
