# Formula Admission Behavior Notes (Baseline)

Status: `baseline-observed`
Workset: `Wx`

## 1. Purpose
Record reproducible baseline observations for formula-admission behavior across COM entry/evaluation mechanisms and workbook-open ingress.

## 2. Baseline Run
Execution date:
1. `2026-03-05`

Environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Workbook compatibility descriptor: `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`
4. Locale profile: `en-US`

Runner:
1. `tools/formula-admission-probe/run-formula-admission-baseline.ps1`

Artifacts:
1. `.tmp/formula-admission-results.csv`
2. `C:/Temp/oxfunc_formula_admission_artifacts/*` (file-ingress mutation probes)

## 3. Scenario Coverage
1. Entry lanes:
   - `Range.Formula`
   - `Range.Formula2`
2. Evaluate lanes:
   - `Application.Evaluate`
   - `Worksheet.Evaluate`
   - `ExecuteExcel4Macro`
   - `WorksheetFunction.Pi`
3. File-ingress lane:
   - open workbook with XML-mutated formula body.

## 4. Baseline Findings
1. `Range.Formula`/`Range.Formula2` reject admission-invalid formulas immediately on set.
2. In this baseline, `=PI(123)`, `=SIN()`, and `=1+` each fail on set with COM exception `0x800A03EC`.
3. Runtime-domain-invalid formula `=ASIN(2)` is admitted and evaluates to `#NUM!` (not rejected on set).
4. `Application.Evaluate` and `Worksheet.Evaluate` do not throw for admission-invalid expressions in this baseline; they return `Int32` sentinel `-2146826273`.
5. `ExecuteExcel4Macro` throws for admission-invalid expressions (`PI(123)`, `SIN()`, `1+`) and returns values for admitted expressions.
6. `WorksheetFunction.Pi(123)` throws `Invalid number of arguments.`
7. Workbook-ingress probe:
   - XML-mutated valid formula (`ASIN(2)`) opens; after recalculation it materializes as `#NUM!`.
   - XML-mutated admission-invalid formula (`PI(123)`) fails `Workbooks.Open`.
8. File-ingress note:
   - pre-recalc cached value may be stale after mutation; recalc is required before semantic observation.

## 5. Working Interpretation (Provisional)
1. Worksheet cell-entry lanes enforce formula admission before evaluation.
2. Runtime-domain errors are evaluation outcomes, not admission outcomes.
3. Evaluate-family APIs are not equivalent in error-surfacing behavior; `Application/Worksheet.Evaluate` can marshal error-like sentinels rather than throwing.
4. Workbook-open path appears to enforce admission for XML-loaded formulas in this baseline.

## 6. Open Items
1. Add dedicated C API lane (`xlfEvaluate`/`xlCoerce`) through XLL probe exports and compare directly against COM evaluate lanes.
2. Extend sentinel mapping beyond legacy 7 worksheet errors to newer dynamic errors where applicable (`#SPILL!`, `#CALC!`, etc.).
3. Expand matrix across additional builds/channels and non-default compatibility settings.
4. Run explicit replay for non-`en-US` locales (deferred in this phase).

## 7. COM Marshaled Error Mapping (Baseline)
On `2026-03-05` (Excel `16.0 (build 19725)`), worksheet error cells surfaced through COM `Range.Value2` as `Int32` sentinels with this mapping:

1. `-2146826288` -> `#NULL!` (`ERROR.TYPE=1`)
2. `-2146826281` -> `#DIV/0!` (`ERROR.TYPE=2`)
3. `-2146826273` -> `#VALUE!` (`ERROR.TYPE=3`)
4. `-2146826265` -> `#REF!` (`ERROR.TYPE=4`)
5. `-2146826259` -> `#NAME?` (`ERROR.TYPE=5`)
6. `-2146826252` -> `#NUM!` (`ERROR.TYPE=6`)
7. `-2146826246` -> `#N/A` (`ERROR.TYPE=7`)

Direct answer for the admission probe sentinel:
1. `-2146826273` corresponds to `#VALUE!`.

Artifacts:
1. `.tmp/excel-com-error-mapping.csv`
2. `.tmp/excel-com-error-mapping-with-error-type.csv`
