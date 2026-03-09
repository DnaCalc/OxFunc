# Function Adapter Layering Preliminary Spec

Status: `provisional`
Owner lane: `OxFunc`

## 1. Purpose
Define a reusable layered function pipeline that separates:
1. pure function kernels,
2. declarative coercion/array-lift adapters,
3. declarative dereference/argument-preparation adapters.

## 2. Layer Contract
1. Layer K (`kernel`):
   - pure core semantics (for example `num -> num`).
   - no reference-resolution seam access.
2. Layer C (`coercion_lift_adapter`):
   - declarative conversion profile and array-map/error policy.
   - consumes prepared value arguments.
3. Layer P (`arg_preparation_adapter`):
   - dereference/normalization profile.
   - may consume FEC reference capabilities.

Pipeline shape:
1. `surface_args -> P -> prepared_values -> C -> kernel -> result`.

### 2.1 Admission and Failure Accounting
1. Function pipeline execution (`P/C/K`) runs only after FEC/F3E admission for the call context has succeeded.
2. Admission failures (for example token/snapshot/capability rejection at seam level) are boundary outcomes and must not be counted as function semantic failures.
3. Function semantic outcomes and failures are evaluated only on admitted calls.

## 3. Profiles
### 3.1 Argument Preparation Profile
1. `values_only_pre_adapter`:
   - references are resolved before function adapter entry.
   - function adapter sees values only.
2. `refs_visible_in_adapter`:
   - references are visible to function adapter.
   - function controls dereference timing/strategy.

### 3.2 Coercion/Lift Profile
1. `unary_numeric_scalar_only`
2. `unary_numeric_scalar_or_array_elementwise`
3. `aggregate_direct_and_range_dual_policy`
4. `lookup_match_profile`
5. `custom`

### 3.3 Kernel Signature Class
1. `nullary_const`
2. `num_to_num`
3. `nums_to_num`
4. `text_to_text`
5. `lookup_match`
6. `custom`

## 4. ABS Mapping (Reference Example)
1. kernel:
   - `abs_num : Number -> Number`.
2. coercion/lift adapter:
   - `unary_numeric_scalar_or_array_elementwise`.
3. argument preparation:
   - `values_only_pre_adapter`.
4. FEC split:
   - adapter-level `fec_dependency_profile = none`.
   - surface-level `surface_fec_dependency_profile = ref_only`.

## 5. SUM/Family Mapping (Reference-Sensitive Example)
1. kernel:
   - numeric fold.
2. coercion/lift adapter:
   - aggregate dual policy (direct arg vs range-scan behavior).
3. argument preparation:
   - likely `refs_visible_in_adapter` for provenance-sensitive rules.

## 6. Large-Sweep Guidance
For broad non-interesting-function rollout:
1. default to `values_only_pre_adapter` where behavior does not depend on source provenance.
2. maximize reusable declarative coercion/lift profiles instead of per-function bespoke adapters.
3. keep kernels minimal and proof-first.

For interesting/reference-sensitive families:
1. use `refs_visible_in_adapter` only when required by observable behavior.
2. require explicit provenance policy rows (direct arg, range-scan, spilled refs, structured refs).
3. attach focused empirical lanes that distinguish provenance-sensitive outcomes.

## 7. Required Tracking Fields
Function contracts should explicitly state:
1. `arg_preparation_profile`
2. `coercion_lift_profile`
3. `kernel_signature_class`
4. `fec_dependency_profile` (adapter level)
5. `surface_fec_dependency_profile` (pipeline level)

## 8. Rust/Lean Coupling Pattern
For each promoted function slice, keep an explicit two-level model:
1. Adapter/kernel level:
   - Rust: adapter/kernel module (`functions::<f>.rs`).
   - Lean: adapter/kernel module (`Functions.<F>.lean`).
2. Surface execution level:
   - Rust: pre-adapter composition path (either `functions::<f>_surface.rs` or a shared declarative runner path from `functions::adapters`).
   - Lean: surface composition module (`Functions.<F>Surface.lean` or equivalent).

Required cross-level linkage:
1. at least one theorem showing surface-prepared path corresponds to adapter path for prepared inputs.
2. at least one runtime test showing surface execution equals adapter execution on prepared-value cases.
3. correlation ledger row must include both modules and both theorem/test inventories.

### 8.1 Declarative Surface Runner Policy
For non-interesting functions with `values_only_pre_adapter` and no custom surface quirks:
1. use shared surface helpers in `crates/oxfunc_core/src/functions/adapters.rs`:
   - `run_values_only_prepared`
   - `map_values_only_prepared`
2. keep function-specific surface wrappers minimal (or inline in the function module) and avoid bespoke pre-adapter boilerplate.
3. reserve dedicated `*_surface.rs` modules for functions that need custom surface behavior:
   - lazy/selective argument evaluation,
   - source-provenance-sensitive dereference timing,
   - reference-return/caller-context custom paths,
   - other non-standard boundary semantics.
4. decision template:
   - if function behavior fits shared values-only preparation, keep surface in the main function module.
   - if function needs bespoke boundary logic, use a dedicated `*_surface.rs` companion module.

Current adoption baseline (Rust):
1. `ABS` surface wrappers now use shared declarative runner helpers.
2. `ISNUMBER`, `OP_ADD`, `SUM`, `SEQUENCE`, and `INDIRECT` route values-only surface preparation through the shared runner.
3. `XMATCH` remains split (`xmatch.rs` + `xmatch_surface.rs`) because it still needs custom surface behavior.

## 9. Deferred Considerations (Post-W6 XMATCH)
1. Do not promote a new global evaluation-strategy axis yet (`eager_full_scan` vs `selective_probe` remains `to_consider`).
2. Near-term selective behavior should be expressed per function via `refs_visible_in_adapter` and function-owned dereference policy, not as a mandatory cross-function abstraction.
3. If introduced later, selective dereference capability must support reference-subset probing (sub-array/window dereference), not only whole-reference materialization.
4. Non-standard error/coercion behavior should remain function-local until multiple independent functions demonstrate a stable reusable abstraction.
5. Do not assume a generic "lookup function class" for semantic reuse; function-specific quirks remain first-class.
6. Keep counterexample replay loop mandatory when Excel behavior diverges (contract + runtime + formal + evidence + correlation).
7. Keep Q-vs-U side-by-side performance/semantic benching as planned follow-on, lower priority for current closure.
