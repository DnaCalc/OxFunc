# Function Slice Contract (Prelim) - ABS()

## 1. Slice Identity
1. `function_id`: `FUNC.ABS`
2. `display_name`: `ABS`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - formula/evaluator contract admits unary shape only (`ABS(x)`).
   - zero-arg and multi-arg shapes are non-admitted in this slice contract.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure` (no host mutation/shared-state dependency in slice scope)
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `unary_numeric_scalar_or_array_elementwise`
7. `kernel_signature_class`: `num_to_num`
8. `error_policy_class`: `strict_propagate`
9. `compat_version_policy`: `version_scoped` (provisional)
10. `function_adapter_fec_dependency_profile`: `none`
11. `surface_fec_dependency_profile`: `ref_only` (through pre-adapter preparation seam)
12. `fec_facility_tags`: `cap_reference_resolution` (surface preparation seam)
13. `compile_eval_class`: `const_foldable_when_closed`

## 4. Required Behavior Lanes
1. admission lane:
   - exact unary arity only.
2. coercion lane:
   - function adapter consumes prepared values (no reference-bearing args).
   - text numeric and logical numeric coercion are admitted by declarative unary numeric coercion/lift adapter.
3. pre-adapter dereference lane:
   - uses W4 selected seam (`capability_record_model`) before entering the ABS adapter.
   - references are resolved into prepared value args by evaluator-side preparation.
4. numeric kernel lane:
   - computes absolute value for admitted numeric payloads.
5. floating-point lane:
   - `-0` normalizes to `+0`.
   - `-inf` maps to `+inf`.
   - `NaN` remains `NaN` payload-class (no domain rejection in kernel).
   - worksheet-formula paths observed so far do not preserve tiny negative inputs through `ABS` strongly enough to survive reciprocal follow-back; the tested tiny-value ABS lanes collapse to worksheet-visible zero and yield `#DIV/0!` on reciprocal reuse.
6. array-lift lane:
   - elementwise lift for admitted array-like argument lanes.
   - per-element coercion errors are preserved in lifted outcomes.
7. formula-vs-reference boundary lane:
   - function adapter is values-only and never requires direct dereference control.
   - resolver-capability denial remains an explicit pre-adapter error lane, not silent fallback.
8. entrypoint-mechanism lane:
   - admission/runtime surface tracked across `Range.Formula`, `Application.Evaluate`, `Worksheet.Evaluate`, and `WorksheetFunction.Abs`.

## 5. Core Outcome Model
1. admitted unary call:
   - pipeline returns `abs_kernel(num)` after preparer + coercion/lift adapter.
2. coercion failure:
   - returns coercion error lane (`non_numeric_text`, worksheet error propagation).
3. preparation failure:
   - returns pre-adapter seam error lane (resolver capability and resolution failures).
4. arity mismatch:
   - returns admission error lane (`arityMismatch` in Lean/Rust scaffolds).
5. lifted mixed inputs:
   - returns per-element number/error outcomes.

## 6. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - `unbounded_provisional` pending multi-build replay.
2. Workbook Compatibility Version scope:
   - `unbounded_provisional` pending template replay.

## 7. Proof/Implementation Obligations
1. Lean obligations:
   - adapter/kernel module: `formal/lean/OxFunc/Functions/Abs.lean`
   - surface execution module: `formal/lean/OxFunc/Functions/AbsSurface.lean`
   - `absKernel_of_neg`
   - `absKernel_of_nonneg`
   - `absMeta_values_only_preparation`
   - `absMeta_adapter_fec_none`
   - `absMeta_surface_fec_ref_only`
   - `absMeta_kernel_signature_num_to_num`
   - `absMeta_coercion_profile_unary_numeric_scalar_or_array`
   - `evalAbsScalar_rejects_nil`
   - `evalAbsScalar_rejects_two`
   - `evalAbsScalar_admitted_number_neg`
   - `evalAbsScalar_logical_true`
   - `evalAbsScalar_text_bad`
   - `evalAbsLift_length`
   - `evalAbsScalar_deterministic`
   - `prepareAbsSurfaceArgValuesOnly_prepared_passthrough`
   - `evalAbsSurfaceScalar_rejects_nil`
   - `evalAbsSurfaceScalar_prepared_number_neg`
   - `evalAbsSurfaceScalar_prepared_logical_true`
   - `evalAbsSurfaceScalar_prepared_text_bad`
   - `evalAbsSurfaceLift_length`
   - `evalAbsSurfacePrepared_matches_adapter_arg`
   - `evalAbsSurfaceScalar_deterministic`
   - `evalAbsSurfaceFromRef_deterministic`
2. Rust obligations:
   - layered runtime split:
     - adapter/kernel module: `crates/oxfunc_core/src/functions/abs.rs`
     - surface/pre-adapter composition path: `crates/oxfunc_core/src/functions/abs.rs` (wrappers) + `crates/oxfunc_core/src/functions/adapters.rs` (shared runner)
   - explicit unary-admission check.
   - declarative pre-adapter preparation profile (`values_only_pre_adapter`).
   - declarative coercion/lift profile (`unary_numeric_scalar_or_array_elementwise`).
   - floating-point edge checks (`-0`, `±inf`, `NaN`).
   - array-lift mixed outcome checks.

## 8. Evidence Posture
1. `spec_anchor`:
   - `TBD-SPEC-ABS-001`
2. empirical anchors:
   - `W2-RUN-20260305` (floating-point feed used by ABS edge lane)
   - `W4-COERCE-BL-20260307` (coercion + resolver seam baseline)
   - `W5-ABS-BL-20260308` (ABS slice baseline run)
   - `W5-ABS-ENTRY-20260308` (entrypoint mechanism baseline)
   - `W5-ABS-FP-20260310` (ABS floating-point reciprocal follow-back)
3. current status rationale:
   - contract, Lean, Rust, and empirical baseline are wired for the declared W5 scope.
   - expansion for multi-build/channel/compat template remains required before `validated`.

