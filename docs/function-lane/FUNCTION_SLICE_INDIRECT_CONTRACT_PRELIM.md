# Function Slice Contract (Prelim) - INDIRECT()

## 1. Slice Identity
1. `function_id`: `FUNC.INDIRECT`
2. `display_name`: `INDIRECT`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `2`
2. admission policy:
   - admitted for `INDIRECT(ref_text[, a1_style])`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `volatile_contextual`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `caller_context`
9. `surface_fec_dependency_profile`: `caller_context`

## 4. Current Coverage
1. A1-style text to reference-like conversion is implemented.
2. R1C1 (`a1_style = FALSE`) supports absolute and relative forms, with relative references requiring caller context.
3. non-text ref expression input is rejected as invalid reference text.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/indirect.rs`
2. Lean: `formal/lean/OxFunc/Functions/Indirect.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 6)
