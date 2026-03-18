# W24 Batch 07 Execution Record - Statistical Test Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B07-STAT-TESTS-20260318`

## 1. Purpose
Record the statistical test family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `CHISQ.TEST`, `CHITEST`, `F.TEST`, `FTEST`, `T.TEST`, `TTEST`, and `ZTEST` for the admitted current reference baseline,
2. bind the existing runtime and Lean substrate to a replayable native worksheet packet,
3. make the `CHISQ.TEST` equal-cardinality reshape rule explicit.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader statistical-family harmonization remains outside this packet.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_STATISTICAL_TEST_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH07_STATISTICAL_TESTS_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH07_STATISTICAL_TESTS_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH07_STATISTICAL_TESTS_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch07-statistical-tests-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `crates/oxfunc_core/src/functions/statistical_tests_family.rs`
10. `docs/function-lane/W16_BATCH72_TEST_ALIASES_NOTES.md`
11. `docs/function-lane/W16_BATCH78_STATISTICAL_TESTS_NOTES.md`

## 5. Empirical Findings
From `.tmp/w24-batch07-statistical-tests-results.csv`:
1. `CHISQ.TEST({58,35;11,25;10,23},{45.35,47.65;17.56,18.44;16.09,16.91})` -> `0.000308192017008309`
2. `CHITEST(...)` matches `CHISQ.TEST(...)`
3. `CHISQ.TEST({1},{1})` -> `#N/A`
4. `CHISQ.TEST({58,35;11,25;10,23},{45.35,47.65,17.56,18.44,16.09,16.91})` -> `0.000308192017008309`
5. `CHISQ.TEST({58,35,11,25,10,23},{45.35,47.65;17.56,18.44;16.09,16.91})` -> `0.00637624221502609`
6. `F.TEST({6;"x";28;TRUE;14},{17;1;27;13;15})` -> `0.680555555555556`
7. `F.TEST({6;#N/A;28;22;14},{17;1;27;13;15})` -> `#N/A`
8. `T.TEST({27;"x";24;TRUE;27;29;33;32;35},{19;28;31;30;27;29;30;18;19},2,1)` -> `0.170534661923814`
9. `T.TEST({27;#N/A;24;30;27;29;33;32;35},{19;28;31;30;27;29;30;18;19},2,1)` -> `#N/A`
10. `T.TEST(...,3,1)` -> `#NUM!`
11. `ZTEST({3;6;7;8;6},4,1.5)` -> `0.00143455639603831`

## 6. Implementation Result
1. The family was already wired through dispatch, export, and Lean surfaces.
2. The packet exposed and corrected a real semantic mismatch: `CHISQ.TEST` is not exact-shape on the current baseline; it accepts equal-cardinality arguments and reshapes the second argument row-major to the first argument's layout.
3. The legacy names `CHITEST`, `FTEST`, `TTEST`, and `ZTEST` are accepted on the current baseline and now have packet evidence instead of stale open-lane notes.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch07-statistical-tests-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml statistical_tests_family`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml test_alias_family`
4. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
5. `lake build`

## 8. Standing
1. `CHISQ.TEST`, `CHITEST`, `F.TEST`, `FTEST`, `T.TEST`, `TTEST`, and `ZTEST` are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted current-baseline statistical-test slice.
3. The next work in `W24` continues with the Wave 3 families.
