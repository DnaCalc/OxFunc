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

## 4. W10 Seed Coverage
1. reference source projection is implemented as reference-preserving shape projection.
2. area number other than `1` is rejected in seed scope.
3. array payload extraction is explicitly out of scope (`array_payload_unavailable`) until array payload modeling is available.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/index.rs`
2. Lean: `formal/lean/OxFunc/Functions/Index.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 7)
