# W24 Batch 04 Execution Record - Array Text Split Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B04-ARRAY-TEXT-SPLIT-20260318`

## 1. Purpose
Record the array/text split family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `ARRAYTOTEXT` and `TEXTSPLIT` for the admitted current reference baseline,
2. bind the existing runtime and Lean substrate to a replayable native worksheet packet,
3. keep the witness strategy honest by using scalar `ARRAYTOTEXT(TEXTSPLIT(...),1)` projections.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader spill-host publication and locale/version sweeps remain orthogonal validation work.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_ARRAY_TEXT_SPLIT_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH04_ARRAY_TEXT_SPLIT_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH04_ARRAY_TEXT_SPLIT_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH04_ARRAY_TEXT_SPLIT_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch04-array-text-split-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `docs/function-lane/W16_BATCH67_ARRAY_TEXT_SPLIT_NOTES.md`

## 5. Empirical Findings
From `.tmp/w24-batch04-array-text-split-results.csv`:
1. `ARRAYTOTEXT({TRUE,#VALUE!;"Hello",2},0)` -> `TRUE, #VALUE!, Hello, 2`
2. `ARRAYTOTEXT({TRUE,#VALUE!;"Hello",2},1)` -> `{TRUE,#VALUE!;"Hello",2}`
3. `ARRAYTOTEXT(TEXTSPLIT("Dakota Lennon Sanchez"," "),1)` -> `{"Dakota","Lennon","Sanchez"}`
4. `ARRAYTOTEXT(TEXTSPLIT("1,2,3;4,5",",",";"),1)` -> `{"1","2","3";"4","5",#N/A}`
5. `ARRAYTOTEXT(TEXTSPLIT("Do. Or do not. There is no try. -Anonymous",{".","-"},,TRUE),1)` -> `{"Do"," Or do not"," There is no try"," ","Anonymous"}`
6. `ARRAYTOTEXT(TEXTSPLIT("aXbxc","x",,FALSE,1,"pad"),1)` -> `{"a","b","c"}`
7. `ARRAYTOTEXT(TEXTSPLIT("1,2,3;4,5",",",";",FALSE,0,"pad"),1)` -> `{"1","2","3";"4","5","pad"}`

## 6. Implementation Result
1. `ARRAYTOTEXT` and `TEXTSPLIT` were already wired through dispatch, export, and Lean surfaces.
2. The current packet closes the evidence and contract gap for the admitted current-baseline slice.
3. The witness strategy deliberately uses `ARRAYTOTEXT(TEXTSPLIT(...),1)` so the packet captures the array semantics without overclaiming broader spill-host behavior.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch04-array-text-split-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml array_text_split_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`
5. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`

## 8. Standing
1. `ARRAYTOTEXT` and `TEXTSPLIT` are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted array-render and scalar-witness slice rather than full spill-host publication behavior.
3. Wave 1 of `W24` is now fully packeted and evidenced.
