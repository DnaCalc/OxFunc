# Function Slice Contract (Prelim) - NOW()

## 1. Slice Identity
1. `function_id`: `FUNC.NOW`
2. `display_name`: `NOW`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `0`
   - maximum: `0`
2. admission policy:
   - nullary only.

## 3. Semantic Class Axes
1. `determinism_class`: `time_dependent`
2. `volatility_class`: `volatile_full`
3. `host_interaction_class`: `application_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `none`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `time_provider`
9. `surface_fec_dependency_profile`: `time_provider`

## 4. W10 Seed Coverage
1. explicit provider seam (`NowProvider`) is implemented.
2. runtime adapter rejects non-finite provider payloads.
3. no argument-admission ambiguity in current seed.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/now_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Now.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 4 and 8)
