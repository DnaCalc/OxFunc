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

## 4. Current Coverage
1. `match_mode` supports exact (`0` default), wildcard (`2`), exact-or-next-larger (`1`), and exact-or-next-smaller (`-1`) through the shared `XMATCH` substrate.
2. `search_mode` supports forward (`1` default), reverse (`-1`), binary ascending (`2`), and binary descending (`-2`) ordering lanes.
3. lookup and return array length mismatch is explicit error, and mismatched non-scalar orientations return `#VALUE!`.
4. `if_not_found` fallback is returned on no-match before worksheet `#N/A` mapping.
5. return lane preserves reference-form results when return element is reference-like.
6. reference-return behavior is empirically exercised for:
   - `CELL("address", XLOOKUP(...))` address identity,
   - `SUM(XLOOKUP(...):XLOOKUP(...))` range composition.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/xlookup_mod.rs`
2. Lean: `formal/lean/OxFunc/Functions/Xlookup.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3, 6, and 7)
