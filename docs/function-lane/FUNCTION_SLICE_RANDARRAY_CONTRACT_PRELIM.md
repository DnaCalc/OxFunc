# Function Slice Contract (Prelim) - RANDARRAY()

## 1. Slice Identity
1. `function_id`: `FUNC.RANDARRAY`
2. `display_name`: `RANDARRAY`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `0`
   - maximum: `5`
2. admission policy:
   - admitted for `RANDARRAY([rows[, columns[, min[, max[, whole_number]]]]])`.
   - all arguments default when omitted: `rows=1`, `columns=1`, `min=0`, `max=1`, `whole_number=FALSE`.

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
1. `RANDARRAY()` with all defaults returns a single random decimal in `[0, 1)`.
2. dimension arguments (`rows`, `columns`) are coerced to positive integers; zero or negative dimensions surface `#CALC!`.
3. `min` and `max` are coerced to numbers; if `min` > `max`, the function returns `#VALUE!`.
4. when `whole_number` is `TRUE`, each cell contains a random integer in the closed interval `[ceil(min), floor(max)]`; if that interval is empty, the function returns `#VALUE!`.
5. when `whole_number` is `FALSE` (default), each cell contains a random decimal in `[min, max)`.
6. the result is a dynamic array `EvalValue::Array` with the specified shape, populated from the random-provider seam.

## 5. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - Microsoft 365 current channel (reference baseline).
2. Workbook Compatibility Version scope:
   - `default`.

## 6. Proof/Implementation Obligations
1. Lean obligations:
   - admitted result theorem for zero-through-five-argument cases.
   - dimension validation theorem (`#CALC!` on zero/negative).
   - min/max range validation theorem.
   - whole_number integer-interval validation theorem.
   - pseudo-random provider dependency declaration.
2. Rust obligations:
   - correct default substitution for omitted arguments.
   - dimension validation and `#CALC!` generation.
   - min/max validation and `#VALUE!` generation.
   - whole_number mode integer generation with ceiling/floor truncation.
   - random-provider seam integration for array population.
   - unit tests mapped in correlation ledger.

## 7. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/misc_conversion_family.rs`
2. Lean: `formal/lean/OxFunc/Functions/Randarray.lean`
3. XLL: not yet exported.

## 8. Evidence Posture
1. `spec_anchor`: to be attached from public references.
2. `empirical_anchor`: required for validated status.
3. current status rationale:
   - `RANDARRAY()` is `function-phase-complete` for the current reference baseline,
   - dimension validation, min/max range checking, whole_number mode, and dynamic array output are aligned across Rust and Lean,
   - cross-build/channel and compatibility-variant replay remain validation-phase follow-up.
