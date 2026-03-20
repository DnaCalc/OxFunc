# WORKSET - Reopened XIRR Large-Root Solver Precision (W37)

## 1. Purpose
Own the remaining direct-Excel precision drift on the large positive-root `XIRR` lane that remained after `W32`.

## 2. Provenance
Opened by `W32` scope reconciliation on `2026-03-19`.

Source artifacts:
1. `docs/function-lane/W32_SCOPE_RECONCILIATION.csv`
2. `docs/function-lane/W32_EXECUTION_RECORD.md`
3. `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`
4. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W37_XIRR_LARGE_ROOT_PRECISION_INVENTORY.csv`

Current total:
1. `1` deferred function.

Members:
1. `XIRR`

## 4. Executed Work Streams
1. characterized Excel's large-root published-result policy directly on a positive-guess matrix for the two-cashflow rooted lane,
2. compared the old OxFunc exact closed-form/two-cashflow result against Excel's published observable and confirmed the residual was publication-level rather than a missing root,
3. repaired the admitted lane with an Excel-like bracket-and-bisection publication solver without regressing the already repaired negative-root behavior,
4. reran the `W29` benchmark so the residual lane is no longer classified as `all_diverge_or_inconclusive`.

## 5. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W37` scope.
