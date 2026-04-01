# W16 Batch 76 - Discount, Treasury Bill, and YEARFRAC Slice

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH76-DISCOUNT-BILL-YEARFRAC-20260316`

## Scope
1. `DISC`
2. `INTRATE`
3. `RECEIVED`
4. `PRICEDISC`
5. `TBILLEQ`
6. `TBILLPRICE`
7. `TBILLYIELD`
8. `YEARFRAC`

## Bounded Contract
1. This family is intentionally scalar-only and self-contained.
2. Financial security dates are truncated to integer serials and reject invalid serial dates with `#VALUE!`.
3. `basis` is truncated to integer and admitted only for `0..4`; other values return `#NUM!`.
4. `YEARFRAC` currently uses:
   - basis `0`: US (NASD) `30/360`
   - basis `1`: bounded Actual/Actual via calendar-year splitting
   - basis `2`: Actual/360
   - basis `3`: Actual/365
   - basis `4`: European `30/360`
5. `TBILL*` functions use actual day differences, require settlement strictly before maturity, and reject maturities more than one calendar year after settlement.
6. The security discount functions annualize through the bounded year fraction slice rather than through any broader bond/coupon substrate.

## Pinned Native Baselines
1. `YEARFRAC(DATE(2012,1,1),DATE(2012,7,30),0) -> 0.58055556`
2. `YEARFRAC(DATE(2012,1,1),DATE(2012,7,30),1) -> 0.57650273`
3. `YEARFRAC(DATE(2012,1,1),DATE(2012,7,30),3) -> 0.57808219`
4. `PRICEDISC(DATE(2008,2,16),DATE(2008,3,1),5.25%,100,2) -> 99.80` (rounded display)
5. `INTRATE(DATE(2008,2,15),DATE(2008,5,15),1000000,1014420,2) -> 5.77%`
6. `RECEIVED(DATE(2008,2,15),DATE(2008,5,15),1000000,5.75%,2) -> 1014584.65`
7. `TBILLPRICE(DATE(2008,3,31),DATE(2008,6,1),9%) -> 98.45`
8. `TBILLYIELD(DATE(2008,3,31),DATE(2008,6,1),98.45) -> 9.1416963%` on current-baseline `value2` output (`9.14%` rounded display in support-style prose).
9. `TBILLEQ(DATE(2008,3,31),DATE(2008,6,1),9.14%) -> 9.4151494%` on current-baseline `value2` output (`9.42%` rounded display in support-style prose).
