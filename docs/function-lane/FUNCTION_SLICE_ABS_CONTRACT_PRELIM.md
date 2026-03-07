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
5. `coercion_policy_class`: `permissive_scalar` with explicit error-bearing array lift
6. `error_policy_class`: `strict_propagate`
7. `compat_version_policy`: `version_scoped` (provisional)
8. `fec_dependency_profile`: `ref_only`
9. `fec_facility_tags`: `cap_reference_resolution`
10. `compile_eval_class`: `const_foldable_when_closed`

## 4. Required Behavior Lanes
1. admission lane:
   - exact unary arity only.
2. coercion lane:
   - uses W4 selected seam (`capability_record_model`) and explicit `CallArgValue -> Number` coercion.
   - text numeric and logical numeric coercion are admitted by coercion primitives.
3. numeric kernel lane:
   - computes absolute value for admitted numeric payloads.
4. floating-point lane:
   - `-0` normalizes to `+0`.
   - `-inf` maps to `+inf`.
   - `NaN` remains `NaN` payload-class (no domain rejection in kernel).
5. array-lift lane:
   - elementwise lift for admitted array-like argument lanes.
   - per-element coercion errors are preserved in lifted outcomes.
6. formula-vs-reference boundary lane:
   - references are normalized/dereferenced before numeric coercion.
   - resolver-capability denial remains an explicit error lane, not silent fallback.
7. entrypoint-mechanism lane:
   - admission/runtime surface tracked across `Range.Formula`, `Application.Evaluate`, `Worksheet.Evaluate`, and `WorksheetFunction.Abs`.

## 5. Core Outcome Model
1. admitted unary call:
   - returns `abs(coerce_to_number(arg))`.
2. coercion failure:
   - returns coercion error lane (`non_numeric_text`, worksheet error propagation, resolver error propagation).
3. arity mismatch:
   - returns admission error lane (`arityMismatch` in Lean/Rust scaffolds).
4. lifted mixed inputs:
   - returns per-element number/error outcomes.

## 6. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - `unbounded_provisional` pending multi-build replay.
2. Workbook Compatibility Version scope:
   - `unbounded_provisional` pending template replay.

## 7. Proof/Implementation Obligations
1. Lean obligations:
   - `absKernel_of_neg`
   - `absKernel_of_nonneg`
   - `evalAbsScalar_rejects_nil`
   - `evalAbsScalar_rejects_two`
   - `evalAbsScalar_admitted_number_neg`
   - `evalAbsScalar_logical_true`
   - `evalAbsScalar_text_bad`
   - `evalAbsLift_length`
   - `evalAbsScalar_deterministic`
   - `evalAbsFromRef_deterministic`
2. Rust obligations:
   - explicit unary-admission check.
   - seam-based scalar coercion path (`coerce_arg_to_number` + resolver).
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
3. current status rationale:
   - contract, Lean, Rust, and empirical baseline are wired for the declared W5 scope.
   - expansion for multi-build/channel/compat template remains required before `validated`.
