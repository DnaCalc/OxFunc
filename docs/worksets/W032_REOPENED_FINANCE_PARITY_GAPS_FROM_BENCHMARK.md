# WORKSET - Reopened Finance Parity Gaps From Benchmark (W32)

## 1. Purpose
Own the direct Excel parity gaps that were reopened by the `W29` benchmark packet after comparison against the public ExcelFinancialFunctions F# library and direct native Excel evidence.

## 2. Provenance
Opened by `W29` on `2026-03-18`.

Primary source artifacts:
1. `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`
2. `docs/function-lane/W29_EXECUTION_RECORD.md`
3. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W32_FINANCE_PARITY_REOPENED_INVENTORY.csv`

Current members:
1. `COUPDAYS`
2. `XNPV`
3. `XIRR`

## 4. In Scope
1. repair or re-scope the leap-year actual/actual `COUPDAYS` lane so OxFunc matches direct Excel,
2. repair or re-scope the negative-rate/root-finding `XNPV` and `XIRR` lanes so OxFunc matches direct Excel,
3. update any historical closure claims that were invalidated by the benchmark evidence.

## 5. Out Of Scope
1. treating the F# library as semantic authority,
2. broad finance-family redesign outside the benchmark-proven gaps,
3. speculative `RATE` / `ODDFYIELD` follow-on work without a concrete mismatch lane.

## 6. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no repair packet has executed yet,
   - historical closure surfaces still need full reconciliation against the reopened evidence.
