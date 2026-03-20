# W32 Runtime Requirements - Reopened Finance Parity Repair

Status: `provisional`
Workset: `W32`

## 1. Purpose
Define the replayable runtime surface for the reopened finance parity packet.

## 2. Required Inputs
1. local OxFunc workspace with `cargo` available,
2. local Excel available through COM automation,
3. public ExcelFinancialFunctions benchmark clone under `.tmp/ExcelFinancialFunctions`,
4. `tools/w29-probe/run-w29-finance-benchmark-crosscheck.ps1`

## 3. Required Evidence Surface
1. rerun the `W29` three-way benchmark runner,
2. use the updated `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv` and `.tmp/w29-finance-*.csv` artifacts as the deterministic replay witness for the reopened lanes,
3. accompany the replay with targeted Rust tests for:
   - `coupon_family`
   - `cashflow_rate_family`

## 4. Expected Current Outcome
1. `COUPDAYS` leap-year actual/actual lane aligns with direct Excel,
2. `XNPV` negative-rate lanes align with direct Excel by returning `#NUM!`,
3. `XIRR` negative-root and negative-guess rejection lanes align with direct Excel,
4. any remaining large-root `XIRR` precision drift must be explicitly extracted rather than silently tolerated.
