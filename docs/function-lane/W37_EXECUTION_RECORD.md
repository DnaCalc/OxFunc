# W37 Execution Record - Reopened XIRR Large-Root Solver Precision

Status: `complete`
Workset: `W037`

## 1. Purpose
Record the final repair for the reopened large positive-root `XIRR` lane that remained after `W32`.

## 2. Executed Artifacts
1. `crates/oxfunc_core/src/functions/cashflow_rate_family.rs`
2. `crates/oxfunc_core/examples/w37_xirr_large_root_probe.rs`
3. `tools/w37-probe/run-w37-xirr-large-root-baseline.ps1`
4. `docs/function-lane/W37_SCENARIO_MANIFEST_SEED.csv`
5. `docs/function-lane/W37_RUNTIME_REQUIREMENTS.md`
6. `docs/function-lane/W37_EXECUTION_RECORD.md`
7. `.tmp/w37-xirr-large-root-results.csv`
8. updated `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`
9. updated `.tmp/w29-finance-summary.json`

## 3. Repair Result
1. The old exact closed-form shortcut for the positive-root two-cashflow lane was replaced by an Excel-like published-result policy:
   - bracket upward from the admitted positive guess,
   - preserve the last same-sign lower bound,
   - bisect until the bracket width is within the current Excel-observed relative publication tolerance.
2. This matches the installed Excel baseline on the seeded positive-guess matrix:
   - `0.0001 -> 165601347.17440003`
   - `0.01 -> 165601345.28000006`
   - `0.1 -> 165601345.60000005`
   - `1 -> 165601347`
   - `10 -> 165601346.25`
   - `100 -> 165601345.3125`
   - `1000 -> 165601346.6796875`
3. The negative-root and negative-guess lanes repaired in `W32` remain intact.

## 4. Benchmark Result After Repair
From the rerun `W29` ledger:
1. `7` lanes align across OxFunc, F#, and Excel.
2. `6` lanes now show OxFunc matching Excel while F# differs.
3. `0` lanes remain `all_diverge_or_inconclusive`.

## 5. Verification Runs
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml cashflow_rate_family -- --nocapture`
2. `powershell -ExecutionPolicy Bypass -File tools/w29-probe/run-w29-finance-benchmark-crosscheck.ps1`
3. `powershell -ExecutionPolicy Bypass -File tools/w37-probe/run-w37-xirr-large-root-baseline.ps1`

## 6. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W37` scope
