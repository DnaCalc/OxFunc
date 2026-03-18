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

## 5. First Work Streams
1. characterize the precise basis-`1` maturity-security convention gap on `PRICEMAT` / `YIELDMAT`,
2. pin direct native parity packets for the odd-bond `ODDL*` lanes,
3. reopen the shared bond substrate only after explicit native-valued parity rows exist.

## 6. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - the shared bond/odd-bond substrate is not yet closure-grade,
   - direct parity packets still need to be widened beyond the seeded blocker rows.
