# Function Slice Contract (Prelim) - SEQUENCE()

## 1. Slice Identity
1. `function_id`: `FUNC.SEQUENCE`
2. `display_name`: `SEQUENCE`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `4`
2. admission policy:
   - admitted for `SEQUENCE(rows[, columns[, start[, step]]])`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Current Coverage
1. row/column dimension coercion and positivity checks are implemented.
2. `start` and `step` are coerced and admitted.
3. current runtime materializes array payload values row-major into `EvalArray`.
4. zero dimensions surface through the worksheet `#CALC!` lane.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/sequence.rs`
2. Lean: `formal/lean/OxFunc/Functions/Sequence.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
