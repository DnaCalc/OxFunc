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

## 4. W10 Seed Coverage
1. exact match mode only (`match_mode = 0` or omitted) is implemented.
2. search modes supported in seed: forward (`1` default) and reverse (`-1`).
3. lookup and return array length mismatch is explicit error.
4. return lane preserves reference-form results when return element is reference-like.
5. reference-return behavior is empirically exercised for:
   - `CELL("address", XLOOKUP(...))` address identity,
   - `SUM(XLOOKUP(...):XLOOKUP(...))` range composition.
6. unsupported match/search lanes are explicit seed rejections.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/xlookup.rs`
2. Lean: `formal/lean/OxFunc/Functions/Xlookup.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3, 6, and 7)
