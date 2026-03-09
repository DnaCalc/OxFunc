# Function Slice Contract (Prelim) - ISNUMBER()

## 1. Slice Identity
1. `function_id`: `FUNC.ISNUMBER`
2. `display_name`: `ISNUMBER`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - unary only.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `none`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. W10 Seed Coverage
1. returns true only for numeric prepared value payload.
2. text/logical/error lanes return false.
3. reference arguments are supported through pre-adapter preparation.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/isnumber.rs`
2. Lean: `formal/lean/OxFunc/Functions/IsNumber.lean`
