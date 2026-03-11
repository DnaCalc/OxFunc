# Function Slice Contract (Prelim) - INDEX()

## 1. Slice Identity
1. `function_id`: `FUNC.INDEX`
2. `display_name`: `INDEX`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `4`
2. admission policy:
   - admitted for `INDEX(array_or_ref, row_num[, col_num[, area_num]])`.
   - source may be array-form or reference-form.
   - multi-area reference-form is admitted when the referenced areas are same-sheet A1-style areas.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. source reference identity remains visible to the adapter.
2. `row_num` is syntactically required, but explicit blank/missing `row_num` is treated as `0`.
3. absent `col_num` defaults to `1`; explicit blank/missing `col_num` is treated as `0`.
4. absent or explicit blank/missing `area_num` defaults to `1`.
5. numeric coercion for index arguments follows worksheet number coercion and rejects non-integral, negative, and non-finite values.

## 5. Core Outcome Model
1. `area_num` selection occurs before row/column indexing.
2. in reference form:
   - nonzero `row_num` and nonzero `col_num` select a single referenced cell,
   - `row_num = 0` selects a whole column within the selected area,
   - `col_num = 0` selects a whole row within the selected area,
   - `row_num = 0` and `col_num = 0` return the whole selected area.
3. in array form:
   - nonzero `row_num` and nonzero `col_num` return the payload value at that position,
   - `row_num = 0` returns a one-column array slice,
   - `col_num = 0` returns a one-row array slice,
   - `row_num = 0` and `col_num = 0` return the whole array payload.
4. whole-column and whole-row A1 references are admitted through the shared A1 reference parser.
5. same-sheet multi-area selection is supported; mixed-sheet multi-area references are rejected as an unsupported source in the current OxFunc seam and surface to worksheet `#VALUE!`.

## 6. Post-call Adaptation Policy
1. successful evaluation may return:
   - scalar `EvalValue`,
   - sliced `EvalArray`,
   - `EvalValue::Reference`.
2. out-of-bounds `row_num`, `col_num`, or `area_num` surface to worksheet `#REF!`.
3. source-class violations and mixed-sheet multi-area violations surface to worksheet `#VALUE!`.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-035` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`
2. `empirical_anchor`:
   - `W10-TENMIX-SEED-20260308`
3. policy decision anchors:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 7)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
4. current status rationale:
   - omitted `row_num`/`col_num`, same-sheet multi-area `area_num`, and reference-return identity are replayed and test-backed in the current baseline,
   - Rust and Lean now align on selection/defaulting rules for array-form and reference-form `INDEX`,
   - no current-phase function-semantic gap remains for the declared reference-baseline slice.

## 9. W10 Coverage
1. basic reference projection.
2. array payload selection.
3. row/column zero slice semantics.
4. explicit missing `row_num` and `col_num` defaults.
5. same-sheet multi-area `area_num` selection and out-of-range `area_num`.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/index.rs`
2. Lean: `formal/lean/OxFunc/Functions/Index.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 7)
