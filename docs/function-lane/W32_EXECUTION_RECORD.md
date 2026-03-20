# W32 Execution Record - Reopened Finance Parity Gaps From Benchmark

Status: `complete`
Workset: `W32`

## 1. Purpose
Record the repair-and-reconciliation packet for the finance parity gaps reopened by `W29`.

## 2. Scope
1. `COUPDAYS`
2. `XNPV`
3. `XIRR`

## 3. Executed Artifacts
1. `crates/oxfunc_core/src/functions/coupon_family.rs`
2. `crates/oxfunc_core/src/functions/cashflow_rate_family.rs`
3. `crates/oxfunc_core/examples/w29_finance_probe.rs`
4. `docs/function-lane/W32_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W32_SCOPE_RECONCILIATION.csv`
6. `docs/function-lane/W32_EXECUTION_RECORD.md`
7. `docs/worksets/W032_REOPENED_FINANCE_PARITY_GAPS_FROM_BENCHMARK.md`
8. updated `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`
9. updated `.tmp/w29-finance-oxfunc-results.csv`
10. updated `.tmp/w29-finance-summary.json`

## 4. Repair Result
1. `COUPDAYS` now matches direct Excel on the reopened leap-year actual/actual lane by using the maturity-day nominal previous coupon date for period-size calculation while leaving `COUPDAYBS` and `COUPDAYSNC` on the actual clamped schedule.
2. `XNPV` now matches direct Excel on the reopened negative-rate lanes by rejecting negative worksheet rates with `#NUM!`.
3. `XIRR` now matches direct Excel on:
   - the negative-root two-cashflow lane,
   - the negative-guess rejection lane for the positive-root-only two-cashflow case.

## 5. Residual Extraction
1. `W32` originally extracted the positive large-root `XIRR` lane to successor `W037`.
2. That extracted publication-level residual is now repaired and closed by `W037`.

## 6. Benchmark Result After Repair
From the rerun `W29` ledger:
1. `7` lanes align across OxFunc, F#, and Excel.
2. `6` lanes now show OxFunc matching Excel while F# differs.
3. `0` lanes remain `all_diverge_or_inconclusive`.

## 7. Scope Reconciliation
See:
1. `docs/function-lane/W32_SCOPE_RECONCILIATION.csv`

## 8. Verification Runs
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml coupon_family -- --nocapture`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml cashflow_rate_family -- --nocapture`
3. `powershell -ExecutionPolicy Bypass -File tools/w29-probe/run-w29-finance-benchmark-crosscheck.ps1`

## 9. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W32` scope after reconciliation
