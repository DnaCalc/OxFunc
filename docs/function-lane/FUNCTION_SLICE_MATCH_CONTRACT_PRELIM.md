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
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `lookup_match_profile`
7. `kernel_signature_class`: `lookup_match`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. W10 Seed Coverage
1. exact match lane (`match_type = 0` or omitted in this seed) is implemented.
2. lookup candidate errors are skipped as non-candidates in exact lane.
3. non-exact match modes are explicit seed rejections (`unsupported_match_type_for_seed`).

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/match_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/MatchFn.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 6)
