# Coercion Probe Runtime Requirements

Status: `active`
Workset: `W4`

## 1. Purpose
Define runtime prerequisites and execution contract for W4 coercion/reference-resolution empirical probes.

## 2. Required Inputs
1. Scenario manifest:
   - `docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv`
2. Probe runners:
   - `tools/coercion-probe/run-coercion-excel-baseline.ps1`
   - `tools/coercion-probe/run-coercion-suite.ps1`
   - `tools/coercion-probe/analyze-coercion-results.ps1`
3. Coercion probe output template:
   - `tools/coercion-probe/results/COERCION_RESULTS_TEMPLATE.csv`
4. Output path or output root for coercion run CSV.

## 3. Environment Requirements
1. Windows desktop Excel installation (local installed Excel instance).
2. Capture exact Excel version/build and channel on each run.
3. Capture workbook compatibility descriptor on each row.
4. Locale baseline in this phase: `en-US`.
5. Non-script code policy reminder:
   - non-script code in this repo should be Rust.

## 4. Minimal Execution Commands
1. Single run profile:
```powershell
powershell -File tools/coercion-probe/run-coercion-excel-baseline.ps1 -Manifest docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv -Out .tmp/coercion-results-excel.csv
```
2. Suite run:
```powershell
powershell -File tools/coercion-probe/run-coercion-suite.ps1 -Manifest docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```
3. Analyzer standalone:
```powershell
powershell -File tools/coercion-probe/analyze-coercion-results.ps1 -Results .tmp/coercion-results-excel.csv -OutReport .tmp/coercion-analysis-report.csv
```

## 5. Supported Scenario Actions
1. `calculate`
2. `calculate_formula2`
3. `save_reopen_recalc`
4. `csv_roundtrip_values`
5. `external_ref_open_state_compare`

## 6. Manifest Expectation Contract
Required process fields:
1. `expected_status`
2. `expected_observable`

Supported `expected_observable` clauses (joined by `&&`):
1. `primary_value2_eq:<value>`
2. `primary_text_eq:<value>`
3. `primary_text_len_eq:<value>`
4. `execution_status_eq:<value>`
5. `notes_contains:<substring>`

## 7. Output Contract
Output CSV columns:
1. `scenario_id`
2. `lane`
3. `scenario_kind`
4. `mode`
5. `execution_status`
6. `observed_class`
7. `expected_status`
8. `expected_observable`
9. `expectation_status`
10. `expectation_detail`
11. `excel_version`
12. `excel_channel`
13. `compat_version`
14. `locale_profile`
15. `runner_version`
16. `run_label`
17. `source_manifest`
18. `artifact_ref`
19. `primary_cell`
20. `primary_formula2`
21. `primary_value2`
22. `primary_text`
23. `primary_text_len`
24. `observed_cells`
25. `comparison_bools`
26. `objective`
27. `coercion_axis`
28. `ref_resolution_axis`
29. `notes`

Template:
1. `tools/coercion-probe/results/COERCION_RESULTS_TEMPLATE.csv`

## 8. Current Limitations
1. External/open-state row support is baseline-enabled (`external_ref_open_state_compare`) and currently validated with generated local workbook artifacts in the scenario directory.
2. Baseline coverage remains single build/channel and locale unless explicitly rerun under additional profiles.
3. Aggregate direct-vs-range precedence remains intentionally unresolved and must be reported as matrix outcomes, not collapsed into one global rule.
