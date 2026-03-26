# Function Slice Contract (Prelim) - VALUETOTEXT()

## 1. Slice Identity
1. `function_id`: `FUNC.VALUETOTEXT`
2. `display_name`: `VALUETOTEXT`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `2`
2. admission policy:
   - admitted for `VALUETOTEXT(value[, format])`.
   - `format` defaults to `0` (concise) when omitted.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `none`
10. `fec_facility_tags`: none
11. `compile_eval_class`: `const_foldable_when_closed`

## 4. Core Outcome Model
1. `format = 0` (concise): returns the value rendered as concise text (numbers as decimal strings, text passed through, booleans as `TRUE`/`FALSE`, errors as error text).
2. `format = 1` (strict): returns the value in a format suitable for formula embedding; text values are quoted with escaped inner quotes.
3. non-`{0, 1}` format values surface `#VALUE!`.
4. error values in the first argument propagate as errors in both format modes.
5. the result is always a scalar text `EvalValue`.

## 5. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - Microsoft 365 current channel (reference baseline).
2. Workbook Compatibility Version scope:
   - `default`.

## 6. Proof/Implementation Obligations
1. Lean obligations:
   - admitted result theorem for one-argument and two-argument cases.
   - determinism theorem.
   - format-flag validation theorem.
2. Rust obligations:
   - correct concise and strict rendering for all value types.
   - format-flag validation and `#VALUE!` rejection for invalid flags.
   - error propagation from first argument.
   - unit tests mapped in correlation ledger.

## 7. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/valuetotext_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Valuetotext.lean`
3. XLL: not yet exported.

## 8. Evidence Posture
1. `spec_anchor`: to be attached from public references.
2. `empirical_anchor`: required for validated status.
3. current status rationale:
   - `VALUETOTEXT()` is `function-phase-complete` for the current reference baseline,
   - concise and strict format rendering are aligned across Rust and Lean,
   - cross-build/channel and compatibility-variant replay remain validation-phase follow-up.
