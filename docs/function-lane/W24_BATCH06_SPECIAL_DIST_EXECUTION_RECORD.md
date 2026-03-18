# W24 Batch 06 Execution Record - Special Distribution Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B06-SPECIAL-DIST-20260318`

## 1. Purpose
Record the special-distribution family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `ERF`, `ERF.PRECISE`, `ERFC`, `ERFC.PRECISE`, `GAMMA`, `GAMMALN`, `GAMMALN.PRECISE`, `WEIBULL`, and `WEIBULL.DIST` for the admitted current reference baseline,
2. bind the existing runtime and Lean substrate to a replayable native worksheet packet,
3. make the corrected `WEIBULL.DIST` zero-density rule explicit.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader statistical-family harmonization remains outside this packet.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_SPECIAL_DIST_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH06_SPECIAL_DIST_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH06_SPECIAL_DIST_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH06_SPECIAL_DIST_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch06-special-dist-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `crates/oxfunc_core/src/functions/special_dist_family.rs`
10. `docs/function-lane/W16_BATCH54_SPECIAL_DISTRIBUTIONS_NOTES.md`

## 5. Empirical Findings
From `.tmp/w24-batch06-special-dist-results.csv`:
1. `ERF(1)` -> `0.842700792949715`
2. `ERF(1,2)` -> `0.152621472069238`
3. `ERFC.PRECISE(-1)` -> `1.84270079294971`
4. `GAMMA(0.5)` -> `1.77245385090552`
5. `GAMMA(-0.5)` -> `-3.54490770181103`
6. `GAMMALN.PRECISE(0.5)` -> `0.5723649429247`
7. `WEIBULL(2,3,4,TRUE)` -> `0.117503097415405`
8. `WEIBULL.DIST(2,3,4,FALSE)` -> `0.165468169234612`
9. `WEIBULL.DIST(0,0.5,4,FALSE)` -> `0`
10. `WEIBULL.DIST(0,1,4,FALSE)` -> `0`
11. `WEIBULL.DIST(-1,3,4,TRUE)` -> `#NUM!`

## 6. Implementation Result
1. The special-distribution family was already wired through dispatch, export, and Lean surfaces.
2. The packet exposed and corrected a real semantic bug in `WEIBULL.DIST`: Excel returns `0` at `x = 0` for the pinned density lanes, including `alpha < 1` and `alpha = 1`.
3. The remaining family semantics in the admitted slice now have a replayed native witness packet rather than only spot notes.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch06-special-dist-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml special_dist_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. `ERF`, `ERF.PRECISE`, `ERFC`, `ERFC.PRECISE`, `GAMMA`, `GAMMALN`, `GAMMALN.PRECISE`, `WEIBULL`, and `WEIBULL.DIST` are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted numeric and domain slice.
3. The next work in `W24` continues with the remaining Wave 2 breadth families.
