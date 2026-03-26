# Function Slice Contract (Prelim) - RANDBETWEEN()

## 1. Slice Identity
1. `function_id`: `FUNC.RANDBETWEEN`
2. `display_name`: `RANDBETWEEN`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `2`
2. admission policy:
   - admitted for `RANDBETWEEN(bottom, top)`.
   - both arguments are coerced to numeric values; non-numeric coercion failures surface `#VALUE!`.

## 3. Semantic Class Axes
1. `determinism_class`: `pseudo_random`
2. `volatility_class`: `volatile_full`
3. `host_interaction_class`: `application_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `random_provider`
9. `surface_fec_dependency_profile`: `random_provider`
10. `fec_facility_tags`: `cap_random_provider`
11. `compile_eval_class`: `not_const_foldable`

## 4. Core Outcome Model
1. `bottom` is coerced to a number and then ceiling-truncated to an integer.
2. `top` is coerced to a number and then floor-truncated to an integer.
3. if truncated `bottom` > truncated `top`, the function returns `#NUM!`.
4. otherwise, the function returns a random integer in the closed interval `[ceil(bottom), floor(top)]`.
5. the random value is drawn from the declared random-provider seam supplied by the host surface.
6. the result is always a scalar numeric `EvalValue`.

## 5. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - Microsoft 365 current channel (reference baseline).
2. Workbook Compatibility Version scope:
   - `default`.

## 6. Proof/Implementation Obligations
1. Lean obligations:
   - admitted result theorem for two-argument case.
   - `#NUM!` rejection theorem when bottom > top after truncation.
   - pseudo-random provider dependency declaration.
2. Rust obligations:
   - correct ceiling/floor truncation of bottom/top arguments.
   - integer range validation and `#NUM!` generation.
   - random-provider seam integration.
   - unit tests mapped in correlation ledger.

## 7. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/randbetween_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Randbetween.lean`
3. XLL: not yet exported.

## 8. Evidence Posture
1. `spec_anchor`: to be attached from public references.
2. `empirical_anchor`: required for validated status.
3. current status rationale:
   - `RANDBETWEEN()` is `function-phase-complete` for the current reference baseline,
   - ceiling/floor truncation semantics and random-provider integration are aligned across Rust and Lean,
   - cross-build/channel and compatibility-variant replay remain validation-phase follow-up.
