# String Probe Runtime Requirements

Status: `active`
Workset: `W7`

## 1. Purpose
Define runtime prerequisites and execution contract for W7 string characterization probes.

## 2. Required Inputs
1. Scenario manifest:
   - `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`
2. Probe runners:
   - `tools/string-probe/run-string-excel-baseline.ps1`
   - `tools/string-probe/run-string-suite.ps1`
   - `tools/string-probe/analyze-string-results.ps1`
3. Output path or output root.
4. Optional compatibility-template workbook path.

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
powershell -File tools/string-probe/run-string-excel-baseline.ps1 -Manifest docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv -Out .tmp/string-results-excel.csv
```
2. Default + optional compatibility profile suite:
```powershell
powershell -File tools/string-probe/run-string-suite.ps1 -Manifest docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```
3. Analyzer standalone:
```powershell
powershell -File tools/string-probe/analyze-string-results.ps1 -Results .tmp/string-results-all.csv -OutReport .tmp/string-analysis-report.csv
```

## 5. Manifest Expectation Contract
Required process fields:
1. `expected_status`
2. `expected_observable`

`expected_observable` clause forms:
1. `primary_value2_eq:<value>`
2. `primary_text_eq:<value>`
3. `primary_text_len_eq:<value>`
4. `execution_status_eq:<value>`
5. `notes_contains:<substring>`

Clause separator:
1. `&&`

## 6. Output Contract
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
27. `notes`

Metadata sidecar:
1. `<results.csv>.run-metadata.json` containing manifest hash, runner version, git revision, run label, environment metadata.

Template:
1. `tools/string-probe/results/STRING_RESULTS_TEMPLATE.csv`

## 7. Current Limitations
1. Baseline run still covers one build/channel and one locale unless suite is run with additional profiles.
2. Compatibility-template coverage depends on explicit template provisioning.
3. Full collation and multi-locale ordering closure remains expansion work.

## 8. Evidence Split Policy
1. Record default-workbook runs and compatibility-template runs as separate evidence slices.
2. Use run labels from tooling (`default`, `compat_template`) as the binding key in execution records.
3. Promote combined policy claims only after both slices are present or an explicit single-slice scope bound is declared.
