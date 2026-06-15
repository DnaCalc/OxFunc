# BUG-FUNC-038: NPV/FV/PV negative-base rate lanes

## Summary
- **Bug id**: `BUG-FUNC-038`
- **Opened**: `2026-06-11`
- **Status**: `fix_in_progress`
- **Owner workset**: `W102A`

## Source Refs
- **Reported against ref**: review pass `2026-06-10` / branch `w100-w102-cleanup-pass`
- **Reproduced on ref**: working-tree tests in `financial_time_value_family.rs`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed` (working-tree patch present; checkpoint not landed)
- **Ref notes**: code comments identify a live-Excel probe matrix for the rate
  `<= -1` lanes; the durable probe artifact still needs to be linked or refreshed
  during W102B probe evidence work.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `spec_mismatch`
- **Root cause summary**: `NPV`, `FV`, and `PV` rejected all rates where
  `1 + rate <= 0`, but Excel admits some negative-base cases. `NPV(-2, ...)`
  discounts by an alternating negative base. `FV` and `PV` admit integer-period
  negative-base cases, while fractional-period lanes remain `#NUM!`. The PMT,
  IPMT, and NPER solver/payment lanes still reject rate `<= -1`.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `unknown`
- **Spec vague or missing?**: `yes`
- **Code once correct and later regressed?**: `unknown`
- **Likely introduced in ref**: `unknown`
- **Explanation**: The local implementation used a broad nonpositive-growth
  rejection rule that is correct for several solver/payment functions but too
  broad for direct discounting and integer-period future/present value lanes.

## Reproduction
1. `=NPV(-2,100)` should publish `-100`; the old local path returned `#NUM!`.
2. `=FV(-1,3,-100,0)` should publish `100`; the old local path returned `#NUM!`.
3. `=PV(-2,3,-100,0)` should publish `-100`; the old local path returned `#NUM!`.
4. `PMT`, `IPMT`, and `NPER` at rate `<= -1` remain `#NUM!`.

## Spec And Contract Relationship
- **Spec references**:
  1. `crates/oxfunc_core/src/functions/financial_time_value_family.rs`
- **Spec state at intake**: `vague`
- **Notes**: The fix must preserve the split between direct discounting lanes and
  payment/solver lanes; it must not reopen PMT exactness work under this stream.

## Investigation Log
1. 2026-06-11: W102A working-tree patch added negative-base helper paths and
   focused tests.
2. 2026-06-15: register reconciliation identified this stream as referenced in
   code but missing from `docs/bugs/streams/` and both bug registers.

## Similar-Risk Scan
### Adjacent families to check
1. `PMT`, `IPMT`, `NPER`, `RATE`
2. `NPV` cashflow discounting

### Check method
1. Focused Rust tests in `financial_time_value_family.rs`.
2. Follow-up: link or regenerate the live-Excel probe matrix.

### Results
1. Working-tree tests preserve `PMT`/`IPMT`/`NPER` `#NUM!` behavior at rate
   `<= -1`.

### Follow-on Openings
1. none yet

## Fix Plan
1. Route `NPV`, `FV`, and `PV` through a negative-base-aware growth helper.
2. Keep payment/solver lanes on the existing stricter growth validation.
3. Link or refresh live Excel probe evidence under `W102B` before promoting the
   stream status.

## Validation
1. Working-tree tests:
   - `npv_negative_base_lanes_match_excel_probe_matrix`
   - `fv_negative_base_lanes_match_excel_probe_matrix`
   - `pv_negative_base_lanes_match_excel_probe_matrix`
   - `pmt_ipmt_nper_keep_num_at_rate_at_or_below_minus_one`

## Linked Reports
1. `BUGREP-FUNC-023`

## Evidence
1. `crates/oxfunc_core/src/functions/financial_time_value_family.rs`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded (working-tree tests; durable live-probe artifact still open)
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required by W102B probe evidence
- [ ] handoff filed if W102B evidence shows a cross-repo contract change is required
- [x] linked reports updated
