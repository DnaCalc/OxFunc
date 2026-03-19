# W29 Runtime Requirements - Finance F# Benchmark Cross-Check

## 1. Purpose
Define the replayable runner shape for the `W29` benchmark packet that compares:
1. OxFunc selected finance-family outputs,
2. the public ExcelFinancialFunctions F# library outputs,
3. direct native Excel outputs.

## 2. Required Inputs
1. local OxFunc workspace with `cargo` available,
2. cloned public ExcelFinancialFunctions source under `.tmp/ExcelFinancialFunctions`,
3. local Excel available through COM automation for the direct worksheet lanes,
4. `dotnet` with F# interactive support.

## 3. Required Runner
1. `tools/w29-probe/run-w29-finance-benchmark-crosscheck.ps1`

## 4. Required Emitted Artifacts
1. `.tmp/w29-finance-oxfunc-results.csv`
2. `.tmp/w29-finance-fsharp-results.csv`
3. `.tmp/w29-finance-excel-results.csv`
4. `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`
5. `.tmp/w29-finance-summary.json`

## 5. Minimum Case Set
1. coupon leap-year actual/actual `COUPDAYS` identity lane from the public F# compatibility note,
2. negative-rate `XNPV` examples from the public F# compatibility note,
3. `XIRR` negative-root / large-root examples from the public F# compatibility note,
4. one seeded `RATE` parity lane,
5. `W27` repaired `PRICEMAT` / `YIELDMAT` lanes,
6. `W27` repaired odd-last lanes and seeded `ODDFYIELD` lane.

## 6. Comparison Rules
1. Excel remains authoritative.
2. The F# library is a benchmark and structural comparison source, not a semantic authority.
3. Tolerance-based numeric comparison is allowed for floating-point observables.
4. Error outcomes must remain source-accurate (`#NUM!`, etc.) and must not be coerced into numeric placeholders.

## 7. Expected Outcome Classes
1. `aligned_all_three`
2. `oxfunc_matches_excel_fsharp_differs`
3. `fsharp_matches_excel_oxfunc_differs`
4. `oxfunc_matches_fsharp_not_excel`
5. `all_diverge_or_inconclusive`
