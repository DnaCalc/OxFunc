# Function Slice Contract (Preliminary) - Odd Bond Family

Status: `provisional`
Workset: `W27`
Primary Functions: `ODDFPRICE`, `ODDFYIELD`, `ODDLPRICE`, `ODDLYIELD`

## 1. Scope
1. promote the admitted current-baseline odd-bond family from bounded note status to packet-evidenced status,
2. correct the `ODDLPRICE` / `ODDLYIELD` odd-last slice to direct Excel parity,
3. bind the integrated runtime and Lean substrate to a replayable native worksheet packet.

## 2. Admitted Current-Baseline Slice
1. scalar-only worksheet arguments
2. 1900 date system only
3. `basis` admitted only for `0..4`
4. `frequency` admitted only for `1`, `2`, `4`
5. `ODDF*` current admitted short odd-first slice
6. `ODDL*` current admitted odd-last slice using the Excel-style normalized quasi-coupon accumulation with the US 30/360 modify-both-dates hack

## 3. Explicitly Out Of Slice
1. broader long odd-first investigations beyond the admitted packet
2. broader cross-build/version sweeps
3. host/publication concerns outside the scalar worksheet packet

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `values_only_pre_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `none`
8. surface_fec_dependency_profile: `none`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/odd_bond_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/OddBondFamily.lean`
3. Native worksheet packet in `docs/function-lane/W27_BOND_ODD_BOND_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w27-probe/run-w27-bond-odd-bond-baseline.ps1`
5. Packet execution record in `docs/function-lane/W27_EXECUTION_RECORD.md`
6. Public benchmark comparison note via `W29`

## 6. Scope Boundary
1. The packet closes the previously open direct-Excel parity gap on `ODDLPRICE` / `ODDLYIELD`.
2. The F# ExcelFinancialFunctions project was used as a public benchmark and structural cross-check, not as semantic authority over Excel.
