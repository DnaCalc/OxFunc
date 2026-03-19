# W24 Batch 15 Execution Record - Misc Ordinary Conversion Packet

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B15-MISC-ORDINARY-CONVERSION-20260318`

## 1. Purpose
Record the final ordinary packet in `W24`, closing the ordinary misc conversion triad and extracting the add-in / dynamic-array outliers.

## 2. Scope
1. close `BAHTTEXT`, `CONVERT`, and `PERCENTOF` for the admitted current reference baseline,
2. capture native host evidence that `EUROCONVERT` and `RANDARRAY` are not ordinary current-host worksheet surfaces,
3. hand explicit successor ownership of those outliers to `W025`.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader locale/version sweeps remain outside this packet,
   - `EUROCONVERT` and `RANDARRAY` now belong to `W025`.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_MISC_ORDINARY_CONVERSION_TRIAD_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH15_MISC_ORDINARY_CONVERSION_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH15_MISC_ORDINARY_CONVERSION_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH15_MISC_ORDINARY_CONVERSION_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch15-misc-ordinary-conversion-baseline.ps1`
6. `docs/worksets/W025_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_OUTLIERS.md`
7. `docs/function-lane/W25_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_INVENTORY.csv`
8. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
9. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
10. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
11. `docs/function-lane/W16_BATCH82_MISC_CONVERSION_NOTES.md`

## 5. Empirical Findings
From `.tmp/w24-batch15-misc-ordinary-conversion-results.csv`:
1. `BAHTTEXT(1234)` and `BAHTTEXT(1234.56)` matched the seeded Thai-script outputs.
2. `CONVERT(1,"lbm","kg")`, `CONVERT(68,"F","C")`, and `CONVERT(3.5,"km","m")` matched the seeded numeric outputs.
3. `PERCENTOF(15,60)` matched `0.25`.
4. `EUROCONVERT(10,"DEM","EUR")` returned `#NAME!` on the current host baseline.
5. `RANDARRAY()` returned `#NAME!` on the current host baseline.

## 6. Implementation Result
1. `BAHTTEXT`, `CONVERT`, and `PERCENTOF` are now packet-evidenced for the admitted ordinary current-baseline slice.
2. `EUROCONVERT` and `RANDARRAY` are not honestly ordinary-closure members of `W24`.
3. Those outliers are extracted to `W025`.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch15-misc-ordinary-conversion-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml misc_conversion_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. `BAHTTEXT`, `CONVERT`, and `PERCENTOF` are now function-phase-complete for the admitted current reference baseline.
2. `EUROCONVERT` and `RANDARRAY` no longer belong to `W24`.
3. `W24` must reconcile the extracted successors before the mega-batch can close.
