# W24 Batch 08 Execution Record - Lookup / Prob / Frequency Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B08-LOOKUP-PROB-FREQ-20260318`

## 1. Purpose
Record the lookup/probability/frequency family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `LOOKUP`, `FREQUENCY`, `PROB`, and `MODE.MULT` for the admitted current reference baseline,
2. bind the existing runtime and Lean substrate to a replayable native worksheet packet,
3. make the corrected `PROB` sum-mismatch rule explicit.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader mixed-type and spill-host nuances remain outside this packet.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_LOOKUP_PROB_FREQUENCY_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH08_LOOKUP_PROB_FREQUENCY_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH08_LOOKUP_PROB_FREQUENCY_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH08_LOOKUP_PROB_FREQUENCY_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch08-lookup-prob-frequency-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `docs/function-lane/W16_BATCH77_LOOKUP_PROB_FREQUENCY_NOTES.md`
10. `crates/oxfunc_core/src/functions/lookup_prob_frequency_family.rs`

## 5. Empirical Findings
From `.tmp/w24-batch08-lookup-prob-frequency-results.csv`:
1. `LOOKUP(2.9,{1,2,3},{10,20,30})` -> `20`
2. `LOOKUP(2.9,{1,10;2,20;3,30})` -> `20`
3. `LOOKUP(4,{1,2,3},{10,20,30})` -> `30`
4. `ARRAYTOTEXT(FREQUENCY({1;2;2;3;4},{2;3}),1)` -> `{3;1;1}`
5. `PROB({0;1;2},{0.2;0.3;0.5},1)` -> `0.3`
6. `PROB({0;1;2},{0.2;0.3;0.5},1,2)` -> `0.8`
7. `PROB({0;1;2},{0.2;0.3;0.4},1)` -> `#NUM!`
8. `ARRAYTOTEXT(MODE.MULT({1;2;2;3;3}),1)` -> `{2;3}`
9. `MODE.MULT({1;2;3})` -> `#N/A`

## 6. Implementation Result
1. The family was already wired through dispatch, export, and Lean surfaces.
2. The packet exposed and corrected a real semantic mismatch in `PROB`: a probability vector that does not sum to `1` returns `#NUM!` on the current baseline, not `#N/A`.
3. The array-returning members are now packet-evidenced through scalar `ARRAYTOTEXT(...,1)` witnesses rather than remaining local-only notes.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch08-lookup-prob-frequency-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml lookup_prob_frequency_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. `LOOKUP`, `FREQUENCY`, `PROB`, and `MODE.MULT` are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted lookup/probability/frequency slice.
3. The next work in `W24` continues with the regression/forecast family.
