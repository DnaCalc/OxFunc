# ABS Full-Formality Execution Record

Status: `complete-provisional`
Workset: `W5`
Conformance row: `FDEF-030`
Evidence ID: `W5-ABS-BL-20260308`
Entrypoint Evidence ID: `W5-ABS-ENTRY-20260308`

## 1. Purpose
Track execution status and reproducible evidence for W5 `ABS` full-formality closure.

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
1. `docs/function-lane/ABS_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/ABS_ENTRYPOINT_SCENARIO_MANIFEST_SEED.csv`

## 3. Output Artifacts
1. `.tmp/abs-results-default.csv`
2. `.tmp/abs-results-compat.csv`
3. `.tmp/abs-results-excel.csv` (combined)
4. `.tmp/abs-analysis-report.csv`
5. `.tmp/abs-analysis-summary.json`
6. `.tmp/abs-results-default.csv.run-metadata.json`
7. `.tmp/abs-results-compat.csv.run-metadata.json`
8. `.tmp/abs-entrypoint-results.csv`
9. `.tmp/abs-entrypoint-analysis-report.csv`
10. `.tmp/abs-entrypoint-analysis-summary.json`
11. `.tmp/abs-artifacts/*`

Template:
1. `tools/abs-probe/results/ABS_RESULTS_TEMPLATE.csv`
2. `tools/abs-probe/results/ABS_ENTRYPOINT_RESULTS_TEMPLATE.csv`

## 4. Gate Tracking
### G1 - Contract Closure
1. Status: `closed`.
2. Evidence:
   - `docs/function-lane/FUNCTION_SLICE_ABS_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_ADAPTER_LAYERING_PRELIM_SPEC.md`

### G2 - Formal Closure
1. Status: `closed`.
2. Evidence:
   - Lean modules:
     - `formal/lean/OxFunc/Functions/Abs.lean`
     - `formal/lean/OxFunc/Functions/AbsSurface.lean`
   - Lean build: `lake build`

### G3 - Runtime and Verification Closure
1. Status: `closed`.
2. Evidence:
   - Rust modules:
     - `crates/oxfunc_core/src/functions/abs.rs`
     - `crates/oxfunc_core/src/functions/abs_surface.rs`
   - Rust tests: `cargo test -p oxfunc_core`

### G4 - Evidence Closure
1. Status: `closed-provisional`.
2. Evidence:
   - `docs/function-lane/ABS_PROBE_RUNTIME_REQUIREMENTS.md`
   - `tools/abs-probe/*`
   - `.tmp/abs-results-default.csv`
   - `.tmp/abs-results-compat.csv`
   - `.tmp/abs-results-excel.csv`
   - `.tmp/abs-analysis-report.csv`
   - `.tmp/abs-analysis-summary.json`
   - `.tmp/abs-results-default.csv.run-metadata.json`
   - `.tmp/abs-results-compat.csv.run-metadata.json`
   - `.tmp/abs-entrypoint-results.csv`
   - `.tmp/abs-entrypoint-analysis-summary.json`

### G5 - Promotion Closure
1. Status: `closed-provisional`.
2. Evidence:
   - `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv` (`FDEF-030`)
   - `docs/worksets/W005_ABS_FULL_FORMALITY.md`
   - `tools/function-lane-check/run-correlation-integrity-check.ps1`
   - `.tmp/function-slice-correlation-integrity-report.csv` (`PASS`)

## 5. Baseline Outcomes
1. Suite scenario rows: `32` (`16` default + `16` compat_template).
2. Observed rows: `30`
3. Failed rows: `2`
4. Failed expected: `2` (`ABS5-004` in both run labels, intentional admission-failure sentinel)
5. Failed unexpected: `0`
6. Expectation matched: `32`
7. Expectation mismatched: `0`
8. Dual-run requirement satisfied: `true` (`default` + `compat_template`)
9. Gate status from analyzer: `green`
10. Drift count: `0`
11. Entrypoint probe rows: `8`
12. Entrypoint expectation matched: `8`
13. Entrypoint failed expected: `3`
14. Entrypoint failed unexpected: `0`
15. Entrypoint gate status: `green`

## 6. Key Findings
1. Scalar unary behavior aligns with contract (`ABS5-001`, `ABS5-002`, `ABS5-005`).
2. Error propagation lanes are explicit and stable in baseline:
   - non-numeric text to `#VALUE!` (`ABS5-003`)
   - upstream `#DIV/0!` propagation (`ABS5-007`)
3. Layered adapter decomposition is now explicit:
   - kernel: `num -> num` (`abs_kernel`)
   - coercion/lift adapter: declarative unary numeric profile
   - arg preparation: `values_only_pre_adapter` (reference seam consumed pre-adapter in surface module)
   - strict runtime split: adapter/kernel in `functions::abs`, surface composition in `functions::abs_surface`
4. Floating-point seed lanes observed:
   - `-0` normalized at worksheet surface (`ABS5-006`)
   - subnormal seed path observed as `0` at this baseline boundary (`ABS5-012`)
5. Array-lift, spill obstruction, and reference lanes observed without mismatches (`ABS5-008..ABS5-011`, `ABS5-015`).
6. Persistence lane (`save_reopen_recalc`) and external/open-state lane (`external_ref_open_state_compare`) retained expected outcomes (`ABS5-013`, `ABS5-014`).
7. Locale-sensitive text coercion seed (`ABS5-016`) observed `#VALUE!` for `en-US` baseline.
8. Entrypoint probe confirms mechanism-specific behavior:
   - `Range.Formula` omission failure sentinel retained (`ABS-EP-002`).
   - `Evaluate` lanes observed expected value/error outcomes.
   - `WorksheetFunction.Abs` method is not exposed in this COM dispatch surface for this baseline (`ABS-EP-007`, `ABS-EP-008`).

## 7. Runtime/Formal Alignment Map
1. Rust surface path (`functions::abs_surface::eval_abs_scalar`) corresponds to Lean surface path (`Functions.AbsSurface.evalAbsSurfaceScalar`).
2. Rust pre-adapter preparation (`functions::adapters::prepare_args_values_only`) corresponds to Lean preparation seam (`Functions.AbsSurface.prepareAbsSurfaceArgValuesOnly` + `RefResolverSeam.resolveRefToInput`).
3. Rust adapter scalar path (`functions::abs::eval_abs_adapter_scalar_prepared`) corresponds to Lean adapter scalar path (`Functions.Abs.evalAbsAdapterScalar` / `evalAbsAdapterArg`).
4. Rust kernel (`functions::abs::abs_kernel`) corresponds to Lean kernel (`Functions.Abs.absKernel`).
5. Correlation obligations are maintained in `docs/function-lane/FUNCTION_SLICE_CORRELATION_LEDGER.csv` for both adapter/kernel and surface-path theorem/test IDs.

## 8. Recording Rules
1. Keep expected failures explicit through `expected_status` + `expected_observable`.
2. Treat `execution_failed_unexpected` as gate-blocking; expected failures are not gate failures.
3. Keep version/channel/compat metadata on every probe row for replay.
4. Keep unresolved lanes explicit and bounded in notes.

