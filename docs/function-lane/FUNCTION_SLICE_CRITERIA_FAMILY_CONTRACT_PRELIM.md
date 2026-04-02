# Function Slice Contract (Prelim) - Criteria Family (`COUNTIF`, `COUNTIFS`, `SUMIF`, `SUMIFS`, `AVERAGEIF`, `AVERAGEIFS`, `MAXIFS`, `MINIFS`)

## 1. Slice Identity
1. `function_ids`:
   - `FUNC.COUNTIF`
   - `FUNC.COUNTIFS`
   - `FUNC.SUMIF`
   - `FUNC.SUMIFS`
   - `FUNC.AVERAGEIF`
   - `FUNC.AVERAGEIFS`
   - `FUNC.MAXIFS`
   - `FUNC.MINIFS`
2. `display_family_name`: `Criteria Family`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. admitted signatures:
   - `COUNTIF(range, criteria)`
   - `COUNTIFS(criteria_range1, criteria1, [criteria_range2, criteria2]...)`
   - `SUMIF(range, criteria, [sum_range])`
   - `SUMIFS(sum_range, criteria_range1, criteria1, [criteria_range2, criteria2]...)`
   - `AVERAGEIF(range, criteria, [average_range])`
   - `AVERAGEIFS(average_range, criteria_range1, criteria1, [criteria_range2, criteria2]...)`
   - `MAXIFS(max_range, criteria_range1, criteria1, [criteria_range2, criteria2]...)`
   - `MINIFS(min_range, criteria_range1, criteria1, [criteria_range2, criteria2]...)`
2. arity:
   - `COUNTIF`: exact `2`
   - `COUNTIFS`: even pair structure, min `2`, max `254`
   - `SUMIF`: min `2`, max `3`
   - `SUMIFS`: target + even pair structure, min `3`, max `255`
   - `AVERAGEIF`: min `2`, max `3`
   - `AVERAGEIFS`: target + even pair structure, min `3`, max `255`
   - `MAXIFS`: target + even pair structure, min `3`, max `255`
   - `MINIFS`: target + even pair structure, min `3`, max `255`
3. admission policy:
   - criteria ranges and target ranges remain reference-visible at adapter entry.
   - pair-structured functions reject malformed odd trailing-argument shapes with worksheet-visible `#VALUE!`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`
10. `fec_facility_tags`: `cap_reference_resolution`
11. `compile_eval_class`: `runtime_ref_dependent`

## 4. Pre-call Coercion Policy
1. criteria arguments are prepared through the values-only preparer and then parsed into comparison specs.
2. criteria ranges and target ranges are flattened after reference resolution.
3. wildcard criteria are case-insensitive and honor Excel `*`, `?`, and `~` escaping.
4. blank criteria match true blanks and zero-length text according to the existing criteria kernel policy.
5. target aggregation is numeric-only:
   - text, logical, and blank target cells are ignored in `SUMIF`, `SUMIFS`, `AVERAGEIF(S)`, `MAXIFS`, and `MINIFS`.
   - target-side worksheet errors propagate.

## 5. Core Outcome Model
1. `COUNTIF` counts cells in a single range that match the parsed criteria.
2. `COUNTIFS` counts entries where every criteria-range/criteria pair matches at the same flattened position.
3. `SUMIF` sums numeric cells in `sum_range` for positions admitted by the criteria mask and uses the criteria range itself when `sum_range` is omitted.
4. `SUMIFS` sums numeric cells in `sum_range` for positions admitted by the criteria mask.
5. `AVERAGEIF` and `AVERAGEIFS` average numeric target cells admitted by the criteria mask and return `#DIV/0!` when no numeric match survives.
6. `MAXIFS` and `MINIFS` compute extrema over numeric target cells admitted by the criteria mask and return `0` when no numeric match survives in the current baseline.
6. current-baseline mismatched-shape policy:
   - `SUMIF` top-left anchors an explicit mismatched `sum_range` from the referenced top-left cell over the criteria-range shape when the supplied `sum_range` is a parseable A1-style reference.
   - `AVERAGEIF` top-left anchors an explicit mismatched `average_range` from the referenced top-left cell over the criteria-range shape when the supplied `average_range` is a parseable A1-style reference.
   - `COUNTIFS`, `SUMIFS`, `AVERAGEIFS`, `MAXIFS`, and `MINIFS` remain exact-shape and return `#VALUE!` on equivalent mismatched-shape lanes.
   - omitted `sum_range` in `SUMIF` uses the criteria range directly.
   - omitted `average_range` in `AVERAGEIF` uses the criteria range directly.

## 6. Post-call Adaptation Policy
1. successful `COUNTIF` and `COUNTIFS` evaluation returns scalar numeric counts.
2. successful `SUMIF`, `SUMIFS`, `AVERAGEIF(S)`, `MAXIFS`, and `MINIFS` evaluation returns scalar numeric values.
3. arity mismatch, malformed pair structure, and mismatched exact-shape lanes map to worksheet-visible `#VALUE!`.
4. target-side error cells propagate as worksheet-visible errors when reached by the criteria mask.
5. no criteria-family-specific XLL limitation is presently known for the admitted current-baseline slice; ordinary bridge limitations remain governed by `XLL_VERIFICATION_SEAM_LIMITATIONS.md`.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - current workset evidence is pinned on the local default workbook baseline.
   - broader compatibility-template replay remains orthogonal validation work because no current-baseline semantic divergence is known for this family.

## 8. Evidence Posture
1. `spec_anchor`:
   - `FDEF-041` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. empirical anchors:
   - `W16-B51-CRITERIA-20260316`
   - `W22-CRITERIA-SHAPE-20260318`
   - `W52-SUMIF-BL-20260326`
3. policy decision anchors:
   - `docs/function-lane/W16_BATCH51_CRITERIA_AGGREGATES_NOTES.md`
   - `docs/function-lane/W22_EXECUTION_RECORD.md`
   - `docs/function-lane/W52_EXECUTION_RECORD.md`
4. verification seam qualifier:
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`

## 9. Current Coverage
1. native Excel replay pins the current-baseline split between anchored single-criteria target ranges and exact-shape `*IFS` lanes across:
   - `W22_CRITERIA_SHAPE_SCENARIO_MANIFEST_SEED.csv` for `AVERAGEIF` and the `*IFS` members,
   - `W52_SUMIF_SCENARIO_MANIFEST_SEED.csv` for the missing `SUMIF` target-range lanes.
2. Rust runtime coverage includes:
   - wildcard and blank criteria behavior,
   - exact-shape pair checking,
   - `SUMIF` omitted `sum_range`,
   - `SUMIF` anchored mismatched `sum_range`,
   - `SUMIF` numeric-only target aggregation and target-error propagation,
   - `AVERAGEIF` omitted `average_range`,
   - `AVERAGEIF` anchored mismatched `average_range`,
   - exact-shape rejection for `AVERAGEIFS`, `SUMIFS`, `MAXIFS`, and `MINIFS`,
   - `MAXIFS` / `MINIFS` zero-on-no-numeric-match behavior.
3. Lean coverage for the admitted current-phase slice remains the shared metadata/alignment substrate in `formal/lean/OxFunc/Functions/CriteriaFamily.lean`.
4. no known semantic gap remains in the declared current-baseline slice for these eight functions.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/criteria_family.rs`
2. Lean: `formal/lean/OxFunc/Functions/CriteriaFamily.lean`
3. worksets:
   - `docs/worksets/W022_CRITERIA_FAMILY_SHAPE_HARDENING.md`
   - `docs/HISTORY.md`
4. scenario manifests:
   - `docs/function-lane/W22_CRITERIA_SHAPE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W52_SUMIF_SCENARIO_MANIFEST_SEED.csv`
5. runtime requirements:
   - `docs/function-lane/W22_CRITERIA_RUNTIME_REQUIREMENTS.md`
   - `docs/function-lane/W52_SUMIF_RUNTIME_REQUIREMENTS.md`
6. execution records:
   - `docs/function-lane/W22_EXECUTION_RECORD.md`
   - `docs/function-lane/W52_EXECUTION_RECORD.md`
7. runners:
   - `tools/w22-probe/run-w22-criteria-shape-baseline.ps1`
   - `tools/w52-probe/run-w52-sumif-baseline.ps1`
