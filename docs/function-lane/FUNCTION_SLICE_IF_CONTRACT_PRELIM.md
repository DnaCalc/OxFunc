# Function Slice Contract (Prelim) - IF()

## 1. Slice Identity
1. `function_id`: `FUNC.IF`
2. `display_name`: `IF`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `3`
2. admission policy:
   - admitted for `IF(condition, then_value[, else_value])`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. W10 Seed Coverage
1. condition coercion is explicit and deterministic.
2. branch selection is lazy in the runtime adapter (non-selected branch is not prepared/evaluated).
3. missing `else` defaults to logical false in this seed scope.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/if_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/IfFn.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 2)
