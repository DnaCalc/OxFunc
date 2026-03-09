# Function Slice Contract (Prelim) - MATCH()

## 1. Slice Identity
1. `function_id`: `FUNC.MATCH`
2. `display_name`: `MATCH`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `3`
2. admission policy:
   - admitted for `MATCH(lookup_value, lookup_array[, match_type])`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `lookup_match_profile`
7. `kernel_signature_class`: `lookup_match`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Current Coverage
1. default `match_type` follows Excel’s approximate-next-smaller lane (`1` semantics).
2. explicit `match_type = 0` supports exact matching and wildcard matching when the text lookup value contains unescaped wildcard characters.
3. explicit `match_type = -1` maps to exact-or-next-larger over descending-ordered data.
4. lookup vectors are flattened from one-dimensional arrays and references; two-dimensional lookup arrays are rejected with `#VALUE!`.
5. text comparison is currently case-insensitive in the runtime.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/match_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/MatchFn.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
