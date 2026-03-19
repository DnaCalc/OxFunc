# W27 Execution Record - Advanced Bond And Odd-Bond Hardening

Status: `complete`
Workset: `W27`

## 1. Purpose
Record the current starting position for the advanced bond and odd-bond hardening packet after extraction from `W24`.

## 2. Packet Result
1. Direct Excel parity is now pinned for all `13` in-scope functions through one native worksheet packet.
2. The previously open blocker lanes on basis-`1` `PRICEMAT` / `YIELDMAT` and `ODDLPRICE` / `ODDLYIELD` are corrected in-core and packet-evidenced.
3. The F# ExcelFinancialFunctions project was used as a public benchmark and structural cross-check for the repaired formulas.

## 3. Executed Packet
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_BOND_CORE_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/FUNCTION_SLICE_ODD_BOND_FAMILY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W27_BOND_ODD_BOND_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W27_RUNTIME_REQUIREMENTS.md`
5. `tools/w27-probe/run-w27-bond-odd-bond-baseline.ps1`
6. `.tmp/w27-bond-odd-bond-results.csv`
7. `crates/oxfunc_core/src/functions/bond_core_family.rs`
8. `crates/oxfunc_core/src/functions/odd_bond_family.rs`
9. `formal/lean/OxFunc/Functions/OddBondFamily.lean`
10. `docs/worksets/W027_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_HARDENING.md`
11. `docs/function-lane/W27_EXECUTION_RECORD.md`

## 4. Empirical Findings
From `.tmp/w27-bond-odd-bond-results.csv`:
1. `ACCRINT` returned `90` and `ACCRINTM` returned `60.054794520547944` on the seeded baseline lanes.
2. `DURATION` returned `6.092590716034263` and `MDURATION` returned `5.8936790481589005`.
3. `PRICE` returned `99.68310513860716`, `YIELD` returned `0.1387025191967244`, and `YIELDDISC` returned `0.051938513194480875`.
4. `PRICEMAT` returned `98.59811340546048` and `YIELDMAT` returned `0.06100000000000056` on the repaired basis-`1` maturity-security lane.
5. `ODDFPRICE` returned `113.59771747407883` and `ODDFYIELD` returned `0.06249999999989627`.
6. `ODDLPRICE` returned `99.87828601472134` and `ODDLYIELD` returned `0.04050000000000125` on the repaired odd-last lane.

## 5. Targeted F# Benchmark Comparison
1. The public ExcelFinancialFunctions `priceMat` / `yieldMat` formulas use `DaysInYear(issue,settlement)` as a distinct denominator rather than the simpler year-fraction path previously used locally.
2. The public `oddLFunc` formula uses normalized quasi-coupon accumulation with the US 30/360 modify-both-dates hack on the odd-last lane; this reproduced the native Excel blocker value exactly when ported structurally.
3. The broader OxFunc-vs-F# benchmark pass remains owned by `W29`, but `W27` no longer depends on that wider pass for direct Excel closure.

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w27-probe/run-w27-bond-odd-bond-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml bond_core_family`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml odd_bond_family`
4. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
5. `lake build`

## 7. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W27` scope
