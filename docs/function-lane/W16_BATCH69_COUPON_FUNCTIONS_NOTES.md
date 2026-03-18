# W16 Batch 69 - Coupon Functions

Status: `packet-evidenced-by-w24`

Scope: bounded current-baseline packet for `COUPDAYBS`, `COUPDAYS`, `COUPDAYSNC`, `COUPNCD`, `COUPNUM`, and `COUPPCD`.

Admitted slice:
- Excel 1900 date system only
- regular coupon schedules inferred by stepping backward from maturity
- frequency limited to `1`, `2`, or `4`
- basis values `0` through `4`
- basis behavior bounded as follows:
  - basis `1`: actual days within the located coupon period
  - basis `0` and `4`: 30/360-style accrued / remaining days, period size fixed at `360 / frequency`
  - basis `2`: actual accrued / remaining days, period size fixed at `360 / frequency`
  - basis `3`: actual accrued / remaining days, period size fixed at `365 / frequency`

Local unit coverage pins:
- a regular semiannual example across all six functions
- basis-specific `COUPDAYS` / `COUPDAYBS` / `COUPDAYSNC` lanes
- quarterly end-of-month coupon stepping
- invalid frequency, basis, order, and invalid-date lanes

Open beyond this bounded slice: 1904 workbook system, cross-version parity sweeps, any Excel-specific irregular-edge behavior around serial `60`, and any nuanced basis-`1` differences if native Excel uses a more specialized Actual/Actual convention in edge schedules.

Current standing:
- `W24` Batch 13 now adds native Excel replay evidence for the admitted slice.
- The local packet was corrected to match the observed serial-`0` boundary and settlement-on-coupon-date advance behavior.
- This note remains the original bounded batch snapshot, not the current closure record.
