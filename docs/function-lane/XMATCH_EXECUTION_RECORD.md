# XMATCH Deterministic-Quirks Execution Record

Status: `reopened`
Workset: `W6`
Conformance row: `FDEF-031`
Evidence IDs:
1. `W6-XMATCH-SEED-20260308`
2. `W6-XMATCH-BL-20260308`
3. `W6-XMATCH-EXP-20260310`

## 1. Purpose
Track execution status and reproducible evidence for W6 `XMATCH` current-phase closure.

## 2. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - array-valued `lookup_value` lifting reopened on 2026-04-08 under
     `BUG-FUNC-006` / `W079`; local correction and focused validation are
     landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`.
   - locale and alternate Excel-version sweeps remain orthogonal validation-phase work.
   - richer collation expansion remains evidence-hardening work unless it reveals a concrete semantic mismatch in the tracked baseline.

## 3. Executed Baseline Scope
Execution date:
1. `2026-03-08`
2. `2026-03-10` (expanded empirical matrix + XLL bridge parity follow-up)

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

## 4. Output Artifacts
1. `.tmp/xmatch-results-default.csv`
2. `.tmp/xmatch-results-compat.csv`
3. `.tmp/xmatch-results-excel.csv` (combined)
4. `.tmp/xmatch-analysis-report.csv`
5. `.tmp/xmatch-analysis-summary.json`
6. `.tmp/xmatch-results-default.csv.run-metadata.json`
7. `.tmp/xmatch-results-compat.csv.run-metadata.json`
8. `.tmp/xmatch-artifacts/*`
9. `.tmp/lookup-pass/xmatch-results-default.csv`
10. `.tmp/lookup-pass/xmatch-results-compat.csv`
11. `.tmp/lookup-pass/xmatch-results-excel.csv`
12. `.tmp/lookup-pass/xmatch-analysis-report.csv`
13. `.tmp/lookup-pass/xmatch-analysis-summary.json`
14. `.tmp/lookup-pass/lookup-xll-bridge-results.csv`

Template:
1. `tools/xmatch-probe/results/XMATCH_RESULTS_TEMPLATE.csv`

## 5. Gate Tracking
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
   - `docs/function-lane/LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`
   - `tools/xmatch-probe/*`
   - `tools/xll-addin/run-lookup-xll-bridge-suite.ps1`
   - `.tmp/xmatch-results-default.csv`
   - `.tmp/xmatch-results-compat.csv`
   - `.tmp/xmatch-results-excel.csv`
   - `.tmp/xmatch-analysis-report.csv`
   - `.tmp/xmatch-analysis-summary.json`
   - `.tmp/lookup-pass/xmatch-results-excel.csv`
   - `.tmp/lookup-pass/xmatch-analysis-summary.json`
   - `.tmp/lookup-pass/lookup-xll-bridge-results.csv`

### G4 - Classification Closure
1. Status: `closed`.
2. Decision:
   - retain `XMATCH` as tier-4 `high_interest`.
3. Rationale:
   - multi-mode behavior surface (`match_mode`/`search_mode`) remains policy-rich, but the current reference-baseline semantics are now pinned across workbook replay and XLL differential rows.
   - remaining locale/version expansion is orthogonal validation-phase work rather than a current-phase function-semantic blocker.
4. Evidence:
   - `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv` (`XMATCH` row)
   - `docs/worksets/W006_XMATCH_DETERMINISTIC_QUIRKS.md`
   - `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv` (`FDEF-031`)

## 6. Baseline Outcomes
1. Expanded suite rows: `56` (`28` default + `28` compat_template)
2. Observed rows: `54`
3. Failed rows: `2`
4. Failed expected: `2` (`XM6-005` in both run labels, intentional admission-failure sentinel)
5. Failed unexpected: `0`
6. Expectation matched: `56`
7. Expectation mismatched: `0`
8. Dual-run requirement satisfied: `true` (`default` + `compat_template`)
9. Gate status from analyzer: `green`
10. Drift count: `0`
11. Lookup XLL bridge rows: `15`, all relation checks matched for current manifest scope.

## 7. Key Findings
1. Deterministic exact lanes are stable:
   - default exact forward (`XM6-001`, `XM6-002`),
   - reverse search (`XM6-003`),
   - not-found `#N/A` (`XM6-004`).
2. Invalid mode lanes produce `#VALUE!` (`XM6-006`, `XM6-007`).
3. Wildcard/binary/approximate lanes are observed, implemented in the runtime, and pinned in manifest expectations:
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
8. Blank-vs-empty lookup behavior is now explicitly pinned:
   - omitted or true-blank lookup value matches true blank cells,
   - literal empty string lookup matches formula-empty text cells,
   - literal empty string does not match a true blank cell.
9. Binary duplicate-selection and selected unsorted invalid-result lanes are now pinned empirically and reproduced by the runtime.
10. Array-constant `built-in` vs `ox_XMATCH` parity is green in the dedicated
    lookup bridge manifest, but that older bridge scope did not exercise
    array-valued `lookup_value` spill behavior.
11. Live Excel COM replay on 2026-04-08 reopened a missing current-baseline
    lane:
    - `=XMATCH({1,2,3},{2,4,6,8}) -> {#N/A,1,#N/A}`
    - the composed `FILTER + ISNUMBER + XMATCH` set-intersection formula returns
      `6`
    - the pre-fix local surface rejected the same lane with `#VALUE!`

## 8. Recording Rules
1. Keep expected failures explicit through `expected_status` + `expected_observable`.
2. Treat `execution_failed_unexpected` as gate-blocking; expected failures are not gate failures.
3. Keep version/channel/compat metadata on every probe row for replay.
4. Keep unresolved parity lanes explicit and bounded in slice docs and runtime notes.

## 9. Promotion Status
1. W6 regains the `XMATCH` current-phase closure lane for the replayed
   array-valued `lookup_value` scope after the W079 correction landed on
   `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`.
2. `BUG-FUNC-006` / `W079` remains active only for landed-ref promotion of the
   adjacent `XLOOKUP` correction, which is validated in the working tree after
   fresh Excel replay on 2026-04-29.

## 10. Post-Closure Policy Notes
1. Keep XMATCH coercion/error policy function-local for now; do not lift into a generalized cross-function abstraction yet.
2. Selective-probe/deref optimization is retained as a `to_consider` item and would require reference-subset dereference capability if pursued.
3. Admission parsing behavior is handled in formula-language lanes; this execution record captures runtime-evaluable behavior only.

