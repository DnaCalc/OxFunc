# W24 Batch 02 Execution Record - Date Value Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B02-DATEVALUE-20260318`

## 1. Purpose
Record the date-value family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `DATEVALUE`, `TIMEVALUE`, `DAYS360`, and `DATEDIF` for the admitted current reference baseline,
2. bind the existing runtime and Lean substrate to a replayable native worksheet packet,
3. make the locale-profile boundary explicit rather than implicit.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader locale/version sweeps remain orthogonal validation work.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_DATE_VALUE_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH02_DATEVALUE_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH02_DATEVALUE_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH02_DATEVALUE_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch02-datevalue-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`

## 5. Empirical Findings
From `.tmp/w24-batch02-datevalue-results.csv`:
1. `DATEVALUE("2024-02-03")` -> `45325`
2. `DATEVALUE("2024-02-03 6:35 AM")` -> `45325`
3. `DATEVALUE("6:35 AM")` -> `0`
4. `DATEVALUE("1/2/2024")` -> `#VALUE!`
5. `TIMEVALUE("2:24 AM")` -> `0.1`
6. `TIMEVALUE("22-Aug-2008 6:35 AM")` -> `0.2743055555555556`
7. `TIMEVALUE("2024-02-03")` -> `0`
8. `TIMEVALUE("1/2/2024 6:35 AM")` -> `#VALUE!`
9. `DAYS360(DATE(2024,2,29),DATE(2024,3,31),FALSE)` -> `30`
10. `DAYS360(DATE(2024,2,29),DATE(2024,3,31),TRUE)` -> `31`
11. `DATEDIF(DATE(2001,1,31),DATE(2001,2,28),"MD")` -> `28`
12. `DATEDIF(DATE(2001,1,31),DATE(2001,3,1),"MD")` -> `-2`
13. `DATEDIF(DATE(2001,6,1),DATE(2002,8,15),"Q")` -> `#NUM!`

## 6. Implementation Result
1. The date-value kernels were already integrated through `date_value_family.rs`, `surface_dispatch.rs`, and `xll_export_specs.rs`.
2. The current packet closes the remaining evidence and contract gap instead of introducing a new kernel family.
3. The admitted current-baseline slice now has an explicit standalone contract and replay artifact for the four functions.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch02-datevalue-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml date_value_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`
5. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`

## 8. Standing
1. `DATEVALUE`, `TIMEVALUE`, `DAYS360`, and `DATEDIF` are now function-phase-complete for the current reference Excel baseline.
2. Locale-sensitive admission remains explicit scope boundary rather than an unrecorded gap.
3. The next work in `W24` continues with the remaining Wave 1 families.
