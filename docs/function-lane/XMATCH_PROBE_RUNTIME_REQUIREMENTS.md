# XMATCH Probe Runtime Requirements

Status: `active`
Workset: `W6`

## 1. Purpose
Define runtime prerequisites and execution contract for W6 `XMATCH` empirical probes.

## 2. Required Inputs
1. Scenario manifest:
   - `docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv`
2. Probe runners:
   - `tools/xmatch-probe/run-xmatch-excel-baseline.ps1`
   - `tools/xmatch-probe/run-xmatch-suite.ps1`
   - `tools/xmatch-probe/analyze-xmatch-results.ps1`
   - `tools/xmatch-probe/new-xmatch-compat-template.ps1`
3. Underlying shared execution engine:
   - `tools/coercion-probe/run-coercion-excel-baseline.ps1`
4. Output template:
   - `tools/xmatch-probe/results/XMATCH_RESULTS_TEMPLATE.csv`

## 3. Environment Requirements
1. Windows desktop Excel installation with dynamic-array function support.
2. Capture exact Excel version/build and update channel on each run.
3. Capture workbook compatibility descriptor on each row.
4. Locale baseline in this phase: `en-US`.

## 4. Minimal Execution Commands
1. Single baseline run:
```powershell
powershell -File tools/xmatch-probe/run-xmatch-excel-baseline.ps1 -Manifest docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv -Out .tmp/xmatch-results-excel.csv
```
2. Suite run:
```powershell
powershell -File tools/xmatch-probe/run-xmatch-suite.ps1 -Manifest docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```
3. Analyzer standalone:
```powershell
powershell -File tools/xmatch-probe/analyze-xmatch-results.ps1 -Results .tmp/xmatch-results-excel.csv -OutReport .tmp/xmatch-analysis-report.csv
```

## 5. Supported XMATCH Scenario Actions
1. `calculate`
2. `calculate_formula2`
3. `save_reopen_recalc`

## 6. Manifest Expectation Contract
Required fields:
1. `expected_status`
2. `expected_observable`

Supported `expected_observable` clauses (joined by `&&`):
1. `primary_value2_eq:<value>`
2. `primary_text_eq:<value>`
3. `primary_text_len_eq:<value>`
4. `execution_status_eq:<value>`
5. `notes_contains:<substring>`

## 7. Output Contract
Output columns:
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
1. `tools/xmatch-probe/results/XMATCH_RESULTS_TEMPLATE.csv`

## 8. Current Limitations
1. Baseline execution is single-build/single-locale unless explicitly replayed.
2. `G3` evidence closure for W6 requires dual run labels in combined suite output:
   - `default`
   - `compat_template`
3. Baseline replay currently captures worksheet-observable outcomes for wildcard/binary/approximate lanes, but adapter/runtime parity for those lanes remains explicit follow-on work.
