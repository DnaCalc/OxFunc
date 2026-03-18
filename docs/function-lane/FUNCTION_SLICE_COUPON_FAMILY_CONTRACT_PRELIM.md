# Function Slice Contract (Preliminary) - Coupon Family

Status: `provisional`
Workset: `W24`
Primary Functions: `COUPDAYBS`, `COUPDAYS`, `COUPDAYSNC`, `COUPNCD`, `COUPNUM`, `COUPPCD`

## 1. Scope
1. close the admitted current-baseline regular coupon-schedule slice for the coupon family,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. replace the older bounded-note standing with packet evidence for the seeded basis, end-of-month, and early-date lanes.

## 2. Admitted Current-Baseline Slice
1. Excel 1900 date system only.
2. Regular coupon schedules inferred by stepping backward from maturity.
3. Frequency limited to `1`, `2`, or `4`.
4. Basis values limited to `0` through `4`.
5. Basis behavior:
   - basis `1`: actual days within the located coupon period
   - basis `0` and `4`: 30/360-style accrued and remaining days, period size fixed at `360 / frequency`
   - basis `2`: actual accrued and remaining days, period size fixed at `360 / frequency`
   - basis `3`: actual accrued and remaining days, period size fixed at `365 / frequency`
6. Early-date boundary:
   - serial `0` is admitted as a valid pre-epoch boundary input
   - negative and out-of-range date serials return `#NUM!`
   - when backward coupon stepping crosses below serial `0`, `COUPPCD`/`COUPDAYBS` clip to `0`, `COUPNCD` still returns the first positive coupon date, and `COUPNUM` returns `#NUM!` until settlement reaches the first coupon date
7. Settlement on a coupon date advances to the following coupon period for `COUPNCD`, `COUPPCD`, and `COUPNUM`.

## 3. Explicitly Out Of Slice
1. 1904 workbook date system support.
2. broader cross-version parity sweeps.
3. irregular coupon schedules and broader serial-`60` edge investigations.
4. richer non-numeric coercion breadth beyond the admitted values-only slice.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `values_only_pre_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `none`
8. surface_fec_dependency_profile: `ref_only`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/coupon_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/CouponFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH13_COUPON_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch13-coupon-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH13_COUPON_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The closure is bounded to the admitted regular-schedule current-baseline slice above.
2. The packet now evidences the seeded basis split, end-of-month stepping, early-date boundary, and on-coupon-date advance rules directly.
3. Irregular schedule and broader version-system parity remain separate follow-on validation concerns rather than unacknowledged gaps in this packet.
