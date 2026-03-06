# String Characterization Execution Record

Status: `complete-provisional`
Workset: `W7`
Conformance row: `FDEF-032`
Evidence ID: `W7-STR-BL-20260305`

## 1. Purpose
Track execution status and reproducible evidence for W7 string characterization.

## 2. Executed Baseline
Execution date:
1. `2026-03-05`

Environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Compatibility descriptors observed:
   - `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`
   - `default|CalculationVersion=0|CheckCompatibility=False|FileFormat=6` (CSV reopen rows)
4. Locale profile: `en-US`

Output artifacts:
1. `.tmp/string-results-excel.csv`
2. `.tmp/string-summary-key.csv`
3. `.tmp/string-summary-focus.csv`
4. `.tmp/string-artifacts/*`
5. `.tmp/string-results-default.csv` (suite default profile output)
6. `.tmp/string-results-all.csv` (suite combined output)
7. `.tmp/string-analysis-report.csv` (expectation-aware analyzer output)
8. `.tmp/string-analysis-summary.json`
9. `.tmp/string-results-default.csv.run-metadata.json`

Tooling artifacts:
1. `tools/string-probe/run-string-excel-baseline.ps1`
2. `tools/string-probe/run-string-suite.ps1`
3. `tools/string-probe/analyze-string-results.ps1`
4. `tools/string-probe/README.md`
5. `tools/string-probe/results/STRING_RESULTS_TEMPLATE.csv`
6. `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`

Process-hardening artifacts:
1. expectation contract fields in manifest:
   - `expected_status`
   - `expected_observable`
2. cross-boundary checklist template:
   - `docs/function-lane/CROSS_BOUNDARY_INVARIANT_CHECKLIST_TEMPLATE.md`

## 3. Observation Summary
1. Scenario rows executed: `46`
2. Observed rows: `44`
3. Failed rows: `2`
   - `STR8-023`, `STR8-024` (expected formula-literal admission failure at COM set boundary)
4. Expectation verdicts:
   - `matched=46`
   - `mismatched=0`
   - `unexpected failures=0`
5. Lane coverage:
   - `STR-C`: `26` rows (`24` observed, `2` failed)
   - `STR-D`: `6` rows (`6` observed)
   - `STR-E`: `14` rows (`14` observed)

## 4. Key Outcomes
1. `=` and `EXACT` divergence confirmed (`STR8-001`, `STR8-002`).
2. Tested accent/punctuation/space comparisons are not normalized into equality (`STR8-003..STR8-007`).
3. `TRIM`/`CLEAN` behavior aligns with documented ASCII-focused semantics and leaves several non-ASCII/control variants unchanged (`STR8-008..STR8-011`, `STR8-034..STR8-036`).
4. `LEN(UNICHAR(128512))=2` for this baseline scope (`STR8-012`).
5. Formula-generated overflow beyond 32,767 produced `#VALUE!` sentinel (`STR8-020`, `STR8-021`).
6. Direct interop assignment beyond cap truncates to 32,767 UTF-16 code units without set-time exception (`STR8-028..STR8-030`).
7. Emoji overflow can leave a dangling surrogate tail in this baseline (`STR8-045`).
8. Reference reuse, XLSX save/reopen, and tested CSV roundtrip paths preserved observed baseline values (`STR8-037..STR8-044`).

## 5. Gate Tracking
### G1 - Source Map Closure
1. Status: `closed-provisional`.
2. Evidence: `STRING_BEHAVIOR_RESEARCH_NOTES.md` source ledger (`STR-SRC-001..009`).

### G2 - Scenario Closure
1. Status: `closed`.
2. Evidence: `STRING_SCENARIO_MANIFEST_SEED.csv` (`STR8-001..STR8-046`).

### G3 - Observation Closure
1. Status: `closed` (single build/channel + default compatibility + locale baseline).
2. Evidence: `.tmp/string-results-excel.csv`.

### G4 - Characterization Closure
1. Status: `closed-provisional`.
2. Evidence: `STRING_NORMALIZATION_AND_COMPARISON_POLICY_MAP.md`.

### G5 - Integration Closure
1. Status: `closed-provisional`.
2. Evidence: W3 and W6 dependency docs updated with consumed W7 outputs.

## 6. Next Expansion Actions
1. Replay across additional Excel channels/builds.
2. Replay with non-default compatibility templates.
3. Replay with additional locales.
4. Expand ordering/collation matrix for lookup-sensitive behavior.

## 7. Process Improvements Implemented
1. Expected-behavior contract is now explicit in the scenario manifest (`expected_status`, `expected_observable`).
2. Suite runner now splits default and compatibility-template runs with stable run labels.
3. Analyzer emits verdict and optional drift reports, turning intentional failures into explicit matched expectations.
4. Per-run metadata sidecars capture manifest hash, runner version, git revision, and environment context.
5. Cross-boundary invariant checklist template is now available for all future empirical worksets.
