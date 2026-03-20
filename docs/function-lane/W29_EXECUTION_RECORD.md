# W29 Execution Record - Finance Functions F# Benchmark Cross-Check

Status: `complete`
Workset: `W29`

## 1. Purpose
Record the benchmark packet that compared OxFunc finance-family behavior against the public ExcelFinancialFunctions F# library and direct native Excel evidence.

## 2. Executed Packet
Artifacts created or updated:
1. `tools/w29-probe/run-w29-finance-benchmark-crosscheck.ps1`
2. `crates/oxfunc_core/examples/w29_finance_probe.rs`
3. `docs/function-lane/W29_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`
5. `.tmp/w29-finance-oxfunc-results.csv`
6. `.tmp/w29-finance-fsharp-results.csv`
7. `.tmp/w29-finance-excel-results.csv`
8. `.tmp/w29-finance-summary.json`
9. `docs/worksets/W029_FINANCE_FUNCTIONS_FSHARP_BENCHMARK_CROSSCHECK.md`
10. `docs/function-lane/W29_FINANCE_FUNCTIONS_FSHARP_BENCHMARK_SCOPE.csv`
11. `docs/function-lane/W29_EXECUTION_RECORD.md`

## 3. External Benchmark Baseline
1. The public ExcelFinancialFunctions README still states `199,252` tests.
2. The public compatibility note still flags known divergence areas for:
   - `COUPDAYS`,
   - `RATE` / `ODDFYIELD` qualitatively,
   - `XIRR` / `XNPV`.
3. The local `dotnet test` run for the public F# unit-test project passed:
   - `3221` tests, `0` failures.

## 4. Benchmark Result
From `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`:
1. `7` targeted lanes now align across OxFunc, F#, and Excel:
   - seeded `RATE`,
   - repaired `PRICEMAT` / `YIELDMAT`,
   - repaired `ODDLPRICE` / `ODDLYIELD`,
   - seeded `ODDFYIELD`,
   - repaired negative-root `XIRR`.
2. `5` targeted lanes now show OxFunc matching Excel while F# differs:
   - `COUPDAYS` leap-year actual/actual lane,
   - derived coupon identity lane,
   - two negative-rate `XNPV` lanes,
   - negative-guess `XIRR` rejection on the positive-root-only case.
3. `1` additional targeted lane now also shows OxFunc matching Excel while F# differs:
   - the large positive-root `XIRR` lane where Excel publishes a guess-sensitive bracketed result rather than the exact mathematical root.

## 5. Classification
1. `W29` is complete as a benchmark-and-classification packet.
2. It does not claim that the discrepant OxFunc families are semantically closed.
3. `W29` originally reopened current OxFunc parity concerns for:
   - `COUPDAYS`,
   - `XNPV`,
   - `XIRR`.
4. After the `W32` repair rerun and the `W37` follow-up, no benchmark residual lane remains open.
5. The already repaired `W27` bond and odd-last lanes now have explicit three-way evidence showing that the current OxFunc fixes align with both the public F# benchmark and direct Excel.

## 6. Successor Ownership
1. The reopened OxFunc parity gaps moved to `W032`.
2. `W037` closed the last residual benchmark lane, so no declared-scope `W29` residual remains open.
3. The public F# qualitative notes on `RATE` and `ODDFYIELD` remain watchlist evidence only in this pass because no concrete OxFunc-vs-Excel mismatch was established on the seeded benchmark lanes.

## 7. Verification Runs
1. `dotnet test .tmp/ExcelFinancialFunctions/tests/ExcelFinancialFunctions.Tests/ExcelFinancialFunctions.Tests.fsproj -v minimal`
2. `powershell -ExecutionPolicy Bypass -File tools/w29-probe/run-w29-finance-benchmark-crosscheck.ps1`
3. `cargo run --manifest-path crates/oxfunc_core/Cargo.toml --example w29_finance_probe`

## 8. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W29` scope; successor repair work was owned by `W032`, and the former extracted `XIRR` publication lane is now closed by `W037`
