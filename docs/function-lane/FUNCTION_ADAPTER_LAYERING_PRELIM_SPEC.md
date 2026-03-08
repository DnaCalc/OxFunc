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
   - Rust: pre-adapter composition module (`functions::<f>_surface.rs` or equivalent).
   - Lean: surface composition module (`Functions.<F>Surface.lean` or equivalent).

Required cross-level linkage:
1. at least one theorem showing surface-prepared path corresponds to adapter path for prepared inputs.
2. at least one runtime test showing surface execution equals adapter execution on prepared-value cases.
3. correlation ledger row must include both modules and both theorem/test inventories.
