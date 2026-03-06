# String Probe Tooling (W7/W8)

Purpose:
1. run reproducible Excel baseline scenarios for string behavior characterization,
2. produce expectation-aware verdicts,
3. emit metadata and drift-ready analysis artifacts.

## Primary scripts
1. `run-string-excel-baseline.ps1`
   - executes one run profile (default or compatibility-template).
2. `run-string-suite.ps1`
   - orchestrates default + optional compatibility-template runs and analysis.
3. `analyze-string-results.ps1`
   - summarizes verdicts and optional drift versus prior baseline.

## Inputs
1. scenario manifest: `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`
2. output path(s) for CSV artifacts.
3. optional workbook template for compatibility-profile runs.

## Baseline run example
```powershell
powershell -File tools/string-probe/run-string-excel-baseline.ps1 -Manifest docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv -Out .tmp/string-results-excel.csv
```

## Suite run examples
Default-only:
```powershell
powershell -File tools/string-probe/run-string-suite.ps1 -Manifest docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```

Default + compatibility-template:
```powershell
powershell -File tools/string-probe/run-string-suite.ps1 -Manifest docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp -WorkbookTemplate path/to/template.xlsx
```

With drift comparison:
```powershell
powershell -File tools/string-probe/run-string-suite.ps1 -Manifest docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp -Baseline .tmp/previous/string-results-all.csv
```

## Manifest expectation contract
Manifest columns:
1. `expected_status`
2. `expected_observable`

Supported `expected_observable` clauses (joined by `&&`):
1. `primary_value2_eq:<value>`
2. `primary_text_eq:<value>`
3. `primary_text_len_eq:<value>`
4. `execution_status_eq:<value>`
5. `notes_contains:<substring>`

## Output contract columns
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
27. `notes`

Run metadata sidecar:
1. `<results.csv>.run-metadata.json`

Template:
1. `results/STRING_RESULTS_TEMPLATE.csv`
