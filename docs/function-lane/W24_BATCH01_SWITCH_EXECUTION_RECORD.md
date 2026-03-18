# W24 Batch 01 Execution Record - SWITCH

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B01-SWITCH-20260318`

## 1. Purpose
Record the first executable family closure inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `SWITCH` for the current reference baseline,
2. promote it from the residual `W17` inventory into a function-phase-complete ordinary family row,
3. bind the existing runtime/Lean substrate to a replayable native worksheet packet.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader locale/version sweeps remain orthogonal validation work.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_SWITCH_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH01_SWITCH_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH01_SWITCH_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH01_SWITCH_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch01-switch-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`

## 5. Empirical Findings
From `.tmp/w24-batch01-switch-results.csv`:
1. `SWITCH(2,1,"a",2,"b","other")` -> `b`
2. `SWITCH(3,1,"a",2,"b","other")` -> `other`
3. `SWITCH(3,1,"a",2,"b")` -> `#N/A`
4. `SWITCH("1",1,"a","1","b")` -> `b`
5. `SWITCH(TRUE,1,"a",TRUE,"b")` -> `b`
6. `SWITCH("A","a",1,"A",2)` -> `1`
7. `SWITCH(1,1,1/0,2,3,4)` -> `#DIV/0!`
8. `SWITCH(2,1,1/0,2,3,4)` -> `3`

## 6. Implementation Result
1. `SWITCH` already had integrated runtime/export surfaces through `misc_switch_info_family.rs`, `surface_dispatch.rs`, and `xll_export_specs.rs`.
2. The current packet closes the evidence and contract gap rather than introducing a new kernel.
3. The admitted current-baseline slice now has an explicit standalone contract and replay artifact.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch01-switch-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml misc_switch_info_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`
5. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`

## 8. Standing
1. `SWITCH` is now function-phase-complete for the current reference Excel baseline.
2. `ISFORMULA` remains outside this ordinary packet and stays owned by `W023`.
3. The next work in `W24` continues with the remaining Wave 1 families.
