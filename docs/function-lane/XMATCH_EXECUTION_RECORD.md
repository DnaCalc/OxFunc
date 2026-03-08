# XMATCH Deterministic-Quirks Execution Record

Status: `complete-provisional`
Workset: `W6`
Conformance row: `FDEF-031`
Evidence IDs:
1. `W6-XMATCH-SEED-20260308`
2. `W6-XMATCH-BL-20260308`

## 1. Purpose
Track execution status and reproducible evidence for W6 `XMATCH` exploration closure.

## 2. Executed Baseline Scope
Execution date:
1. `2026-03-08`

Environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Workbook compatibility descriptors:
   - `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`
   - `default|CalculationVersion=191029|CheckCompatibility=True|FileFormat=56` (`compat_template` run)
4. Locale profile: `en-US`
5. Run labels:
   - `default`
   - `compat_template`

Manifest:
1. `docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv`

## 3. Output Artifacts
1. `.tmp/xmatch-results-default.csv`
2. `.tmp/xmatch-results-compat.csv`
3. `.tmp/xmatch-results-excel.csv` (combined)
4. `.tmp/xmatch-analysis-report.csv`
5. `.tmp/xmatch-analysis-summary.json`
6. `.tmp/xmatch-results-default.csv.run-metadata.json`
7. `.tmp/xmatch-results-compat.csv.run-metadata.json`
8. `.tmp/xmatch-artifacts/*`

Template:
1. `tools/xmatch-probe/results/XMATCH_RESULTS_TEMPLATE.csv`

## 4. Gate Tracking
### G1 - Contract Closure
1. Status: `closed`.
2. Evidence:
   - `docs/function-lane/FUNCTION_SLICE_XMATCH_CONTRACT_PRELIM.md`

### G2 - Formal and Runtime Closure
1. Status: `closed`.
2. Evidence:
   - Lean:
     - `formal/lean/OxFunc/Functions/Xmatch.lean`
     - `formal/lean/OxFunc/Functions/XmatchSurface.lean`
     - `lake build` (from `formal/lean`)
   - Rust:
     - `crates/oxfunc_core/src/functions/xmatch.rs`
     - `crates/oxfunc_core/src/functions/xmatch_surface.rs`
     - `cargo test -p oxfunc_core`

### G3 - Empirical Closure
1. Status: `closed-provisional`.
2. Evidence:
   - `docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/XMATCH_PROBE_RUNTIME_REQUIREMENTS.md`
   - `tools/xmatch-probe/*`
   - `.tmp/xmatch-results-default.csv`
   - `.tmp/xmatch-results-compat.csv`
   - `.tmp/xmatch-results-excel.csv`
   - `.tmp/xmatch-analysis-report.csv`
   - `.tmp/xmatch-analysis-summary.json`

### G4 - Classification Closure
1. Status: `closed-provisional`.
2. Decision:
   - retain `XMATCH` as tier-4 `high_interest` for now.
3. Rationale:
   - multi-mode behavior surface (`match_mode`/`search_mode`) has substantial parity risk.
   - coercion/comparison and shape lanes are deterministic but policy-rich.
   - W6 baseline now pins deterministic/nonvolatile/no-host-interaction assumptions under declared profile, while retaining explicit parity follow-ons for full adapter completion.
4. Evidence:
   - `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv` (`XMATCH` row)
   - `docs/worksets/WORKSET_TUX1000_XMATCH_DETERMINISTIC_QUIRKS.md`
   - `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv` (`FDEF-031`)

## 5. Baseline Outcomes
1. Suite rows: `40` (`20` default + `20` compat_template)
2. Observed rows: `38`
3. Failed rows: `2`
4. Failed expected: `2` (`XM6-005` in both run labels, intentional admission-failure sentinel)
5. Failed unexpected: `0`
6. Expectation matched: `40`
7. Expectation mismatched: `0`
8. Dual-run requirement satisfied: `true` (`default` + `compat_template`)
9. Gate status from analyzer: `green`
10. Drift count: `0`

## 6. Key Findings
1. Deterministic exact lanes are stable:
   - default exact forward (`XM6-001`, `XM6-002`),
   - reverse search (`XM6-003`),
   - not-found `#N/A` (`XM6-004`).
2. Invalid mode lanes produce `#VALUE!` (`XM6-006`, `XM6-007`).
3. Wildcard/binary/approximate lanes are observed and now pinned in manifest expectations:
   - wildcard (`XM6-008`) -> `1`,
   - binary ascending (`XM6-009`) -> `3`,
   - binary descending (`XM6-010`) -> `2`,
   - approximate larger (`XM6-011`) -> `3`,
   - approximate smaller (`XM6-012`) -> `2`.
4. Cross-type text-vs-number comparison lane returns `#N/A` (`XM6-013`).
5. Two-dimensional array-shape lane returns `#VALUE!` (`XM6-017`).
6. Lookup-array error semantics in exact mode are non-fatal for candidate elements:
   - match after embedded error returns index (`XM6-018` -> `3`),
   - no-match with embedded error returns `#N/A` (targeted replay `XMS2-004`, `XMS2-008`).
7. Reference and dynamic-array source lanes are stable in both run labels (`XM6-015`, `XM6-016`, `XM6-020`).

## 7. Recording Rules
1. Keep expected failures explicit through `expected_status` + `expected_observable`.
2. Treat `execution_failed_unexpected` as gate-blocking; expected failures are not gate failures.
3. Keep version/channel/compat metadata on every probe row for replay.
4. Keep unresolved parity lanes explicit and bounded in slice docs and runtime notes.

## 8. Post-Closure Policy Notes
1. Keep XMATCH coercion/error policy function-local for now; do not lift into a generalized cross-function abstraction yet.
2. Selective-probe/deref optimization is retained as a `to_consider` item and would require reference-subset dereference capability if pursued.
3. Admission parsing behavior is handled in formula-language lanes; this execution record captures runtime-evaluable behavior only.
