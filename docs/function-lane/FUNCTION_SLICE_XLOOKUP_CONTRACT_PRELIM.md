# Function Slice Contract (Prelim) - XLOOKUP()

## 1. Slice Identity
1. `function_id`: `FUNC.XLOOKUP`
2. `display_name`: `XLOOKUP`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `3`
   - maximum: `6`
2. admission policy:
   - admitted for `XLOOKUP(lookup_value, lookup_array, return_array[, if_not_found[, match_mode[, search_mode]]])`.
   - `lookup_array` must flatten to a one-dimensional vector.
   - `return_array` may be scalar, vector, matrix, or reference-form so long as its orientation aligns with `lookup_array`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `lookup_match`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. references remain visible to the adapter so lookup vectors, return arrays, and reference-form results can be handled without premature scalarization.
2. omitted `match_mode` and `search_mode` default even when fixed-arity surfaces pass explicit `MissingArg` placeholders.
3. omitted or true-blank `lookup_value` matches true blank cells in `lookup_array`.
4. literal empty string lookup matches formula-empty text cells, not true blank cells.
5. lookup and return array orientation is inferred from one-dimensional array shape or reference geometry before selection.

## 5. Core Outcome Model
1. default behavior is exact, forward search.
2. `match_mode` supports exact (`0`), wildcard (`2`), exact-or-next-larger (`1`), and exact-or-next-smaller (`-1`) through the shared `XMATCH` substrate.
3. `search_mode` supports forward (`1`), reverse (`-1`), binary ascending (`2`), and binary descending (`-2`) ordering lanes.
4. `if_not_found` fallback is returned before worksheet `#N/A` mapping on no-match.
5. reference-form return selections preserve reference identity for scalar cells and aligned row/column slices.
6. true blank return cells materialize as numeric zero.
7. mismatched lookup/return lengths and orientation mismatches return worksheet-visible `#VALUE!`.

## 6. Post-call Adaptation Policy
1. scalar matches return a scalar `EvalValue`.
2. matrix row or column selections return `EvalArray` payloads with preserved orientation.
3. aligned reference-area selections return `EvalValue::Reference` rather than dereferenced values.
4. XLL bridge inability to reproduce reference-resolved lookup arrays is an external seam limitation, not a core function-semantic gap.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-035` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-012`, `XLS-CF-FN-007`, `XLS-CF-TV-007`
2. `empirical_anchor`:
   - `W10-TENMIX-SEED-20260308`
   - `W10-LOOKUP-XLL-20260310`
3. policy decision anchors:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3, 6, and 7)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
4. current status rationale:
   - function-phase-complete and locally verified for the current reference Excel baseline,
   - optional-argument defaulting, blank-vs-empty lookup behavior, reference-return identity, blank return-cell materialization, wildcard escaping, and selected binary invalid-result lanes are replayed and test-backed,
   - remaining locale/version expansion is orthogonal validation-phase work rather than a current-phase function-semantic gap.

## 9. W10 Coverage
1. exact, fallback, wildcard escape, binary unsorted, omitted lookup, blank lookup, formula-empty lookup, blank return cell, reference-return address identity, and range-composition lanes are replayed in `W10_S4_SCENARIO_MANIFEST_SEED.csv`.
2. array-constant built-in vs `ox_XLOOKUP` parity and known XLL reference-range seam divergences are replayed in `LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/xlookup_mod.rs`
2. Lean: `formal/lean/OxFunc/Functions/Xlookup.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3, 6, and 7)
