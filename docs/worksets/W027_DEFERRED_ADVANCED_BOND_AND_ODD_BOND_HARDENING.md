# WORKSET - Deferred Advanced Bond And Odd-Bond Hardening (W27)

## 1. Purpose
Own the `W24` finance rows that remained blocked after native replay showed the current kernels are not yet closure-grade for direct Excel parity.

## 2. Provenance
Opened by `W24` extraction after the `BLK-FN-007` and `BLK-FN-008` findings on `2026-03-18`.

Source artifacts:
1. `docs/worksets/W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
2. `CURRENT_BLOCKERS.md`
3. `docs/function-lane/W27_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_INVENTORY.csv`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W27_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_INVENTORY.csv`

Current total:
1. `13` extracted functions.

Members:
1. `ACCRINT`
2. `ACCRINTM`
3. `DURATION`
4. `MDURATION`
5. `PRICE`
6. `PRICEMAT`
7. `YIELD`
8. `YIELDDISC`
9. `YIELDMAT`
10. `ODDFPRICE`
11. `ODDFYIELD`
12. `ODDLPRICE`
13. `ODDLYIELD`

## 4. Entry Criteria
Functions belong in `W27` only if direct native replay proved that the current local finance kernel is not yet parity-clean on the current reference baseline.

## 5. Execution Outcome
1. `W27` now owns a dedicated direct Excel parity packet:
   - `docs/function-lane/W27_BOND_ODD_BOND_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W27_RUNTIME_REQUIREMENTS.md`
   - `tools/w27-probe/run-w27-bond-odd-bond-baseline.ps1`
   - `.tmp/w27-bond-odd-bond-results.csv`
2. `PRICEMAT` / `YIELDMAT` were corrected to use the Excel-style `DaysInYear(issue,settlement)` denominator on the admitted maturity-security slice.
3. `ODDLPRICE` / `ODDLYIELD` were corrected to use the normalized odd-last quasi-coupon accumulation that matches the direct Excel blocker lane.
4. The public ExcelFinancialFunctions F# project was used as a benchmark and structural cross-check, while direct Excel remained authoritative.

## 6. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W27` scope

## 7. Additional Note
1. The public ExcelFinancialFunctions F# project is a useful external benchmark for finance-function semantics and test methodology.
2. It is not authoritative over Excel, but `W27` should eventually compare OxFunc, the F# implementation, and direct Excel evidence explicitly.
