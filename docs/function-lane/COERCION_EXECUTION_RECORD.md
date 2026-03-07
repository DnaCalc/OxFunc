# Coercion and Ref->Val Seam Execution Record

Status: `complete-provisional`
Workset: `W4`
Conformance row: `FDEF-029`
Evidence id: `W4-COERCE-BL-20260307`

## 1. Purpose
Track execution status and reproducible evidence for W4 coercion and reference-resolution characterization.

## 2. Executed Baseline Scope
Execution date:
1. `2026-03-07`

Environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Locale profile: `en-US`
4. Run label: `default`

Baseline lanes:
1. `CO4-A` scalar/admission coercion seeds.
2. `CO4-B` array-lift coercion seeds.
3. `CO4-C` aggregate direct-arg vs range-scan conflict seeds.
4. `CO4-D` reference-resolution and dereference seam seeds.
5. `CO4-E` interop/text-boundary coercion crossover seeds.
6. `CO4-F` external/open-state reference seeds (`external_ref_open_state_compare`) using generated workbook artifact.

Manifest:
1. `docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv`

## 3. Output Artifacts
1. `.tmp/coercion-results-excel.csv`
2. `.tmp/coercion-analysis-report.csv`
3. `.tmp/coercion-results-excel.csv.run-metadata.json`
4. `.tmp/coercion-analysis-summary.json`
5. `.tmp/coercion-artifacts/*`

Template:
1. `tools/coercion-probe/results/COERCION_RESULTS_TEMPLATE.csv`

## 4. Gate Tracking
### G1 - Primitive Enumeration Closure
1. Status: `closed-provisional`.
2. Evidence:
   - `COERCION_AND_CONVERSION_PRELIM_SPEC.md`
   - `COERCION_DECISION_TABLE.csv`

### G2 - Seam Contract Closure
1. Status: `closed`.
2. Evidence:
   - `REF_RESOLUTION_SEAM_OPTIONS.md` (selected baseline: `capability_record_model`)

### G3 - Executable Closure
1. Status: `closed`.
2. Evidence:
   - Rust: `cargo test -p oxfunc_core` passed with coercion/resolver tests.
   - Lean: `lake build` passed with `OxFunc.CoercionPrimitives` and `OxFunc.RefResolverSeam`.

### G4 - Integration Closure
1. Status: `closed-provisional`.
2. Evidence:
   - W5/W6 required behavior lanes now explicitly consume W4 `capability_record_model` seam baseline.

### G5 - Empirical Closure
1. Status: `closed-provisional`.
2. Evidence:
   - baseline run recorded with manifest hash and environment metadata:
     `.tmp/coercion-results-excel.csv.run-metadata.json`
   - analysis report:
     `.tmp/coercion-analysis-report.csv`
   - summary:
     `.tmp/coercion-analysis-summary.json`
   - baseline outcome counts:
     - rows: `18`
     - observed: `17`
     - failed_total: `1`
     - failed_expected: `1`
     - failed_unexpected: `0`
     - expectation matched: `18`
     - expectation mismatched: `0`
     - drift: `0`

## 5. Recording Rules
1. Do not mark empirical closure as complete without explicit `expected_status`/`expected_observable` verdict columns.
2. Keep unresolved outcomes explicit by scenario id; do not collapse into narrative-only summaries.
3. Capture compatibility descriptors per run so behavior drift can be version-scoped.
4. Preserve contradiction triggers for provisional assumptions (for example scalar-array lift outcome class).

## 6. Next Run Checklist
1. Keep `CO4-004` as intentional admission-failure case (`SIN()` omission), validated by expected failure match.
2. External/open-state row now captured as observed:
   - closed state observed `#REF!` (`-2146826265`)
   - open state observed value (`11`) with `primary_changed_closed_to_open=True`.
3. Add compatibility-template and multi-locale reruns before any `validated` promotion.
4. Keep aggregate precedence global-freeze deferred; maintain per-family matrix policy.
