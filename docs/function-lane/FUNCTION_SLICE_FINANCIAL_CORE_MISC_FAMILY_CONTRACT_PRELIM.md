# Function Slice - Financial Core Misc Family Contract (Prelim)

Status: `active`
Owner lane: `OxFunc`
Workset: `W064`

## 1. Purpose
Define the current-phase contract for the `W064` financial core miscellaneous wave.

## 2. Covered Surface
1. `CUMIPMT`
2. `CUMPRINC`
3. `DB`
4. `DDB`
5. `DISC`
6. `DOLLARFR`
7. `INTRATE`
8. `PRICEDISC`
9. `RECEIVED`
10. `SLN`
11. `SYD`
12. `TBILLEQ`
13. `TBILLPRICE`
14. `TBILLYIELD`
15. `VDB`

## 3. Cumulative Finance Contract
1. `CUMIPMT` and `CUMPRINC` use the ordinary values-only pre-adapter seam over the current cumulative-finance substrate.
2. `rate`, `periods`, `pv`, `start_period`, `end_period`, and `type` must be finite numeric inputs; invalid timing values outside `0` and `1` return `#NUM!`.
3. `periods`, `start_period`, and `end_period` follow Excel truncation toward zero before validation.
4. `CUMIPMT` returns the cumulative interest lane and `CUMPRINC` returns the cumulative principal lane over the same payment schedule.

## 4. Depreciation Contract
1. `SLN`, `SYD`, `DB`, `DDB`, and `VDB` use the ordinary values-only pre-adapter seam over the current depreciation substrate.
2. `DB` defaults `month` to `12`, `DDB` defaults `factor` to `2`, and `VDB` defaults `factor` to `2` and `no_switch` to `FALSE`.
3. `DB`, `DDB`, and `VDB` reject invalid domain lanes with `#NUM!`; `SLN` publishes `#DIV/0!` when `life = 0`.
4. `VDB` follows the current baseline declining-balance interval lane with the existing switch/no-switch behavior already pinned in runtime evidence.

## 5. Discount Security / Treasury Bill Contract
1. `DISC`, `INTRATE`, `RECEIVED`, and `PRICEDISC` use the ordinary values-only pre-adapter seam over the current discount-security substrate.
2. `TBILLEQ`, `TBILLPRICE`, and `TBILLYIELD` use the current Treasury-bill lane with the one-calendar-year maturity bound.
3. invalid basis values publish `#NUM!`; invalid or non-positive security inputs publish `#NUM!`; invalid date serials publish `#VALUE!`.
4. `DISC` round-trips the `PRICEDISC` current-baseline lane inside the admitted slice.

## 6. Dollar Fraction Contract
1. `DOLLARFR` uses the current dollar-fraction substrate with the ordinary values-only pre-adapter seam.
2. the denominator truncates toward zero before validation.
3. zero or truncated-zero denominator lanes publish `#DIV/0!`, negative denominator lanes publish `#NUM!`, logical arguments are rejected, numeric text is admitted, and omitted arguments publish `#N/A`.

## 7. Runtime / Formal Anchors
Runtime anchors:
1. `crates/oxfunc_core/src/functions/cumulative_finance_family.rs`
2. `crates/oxfunc_core/src/functions/depreciation_family.rs`
3. `crates/oxfunc_core/src/functions/discount_bill_yearfrac_family.rs`
4. `crates/oxfunc_core/src/functions/dollar_fraction_family.rs`

Formal anchors:
1. `formal/lean/OxFunc/Functions/CumulativeFinanceFamily.lean`
2. `formal/lean/OxFunc/Functions/DepreciationFamily.lean`
3. `formal/lean/OxFunc/Functions/DiscountBillYearfracFamily.lean`
4. `formal/lean/OxFunc/Functions/DollarFractionFamily.lean`

Native replay anchors:
1. `docs/function-lane/W64_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w64-probe/run-w64-financial-core-misc-baseline.ps1`
3. `.tmp/w64-financial-core-misc-results.csv`
