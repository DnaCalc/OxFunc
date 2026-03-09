# Function Slice Contract (Prelim) - SUM()

## 1. Slice Identity
1. `function_id`: `FUNC.SUM`
2. `display_name`: `SUM`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `255`
2. admission policy:
   - admitted in this slice as variadic numeric fold over prepared value arguments.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `aggregate_direct_and_range_dual_policy`
7. `kernel_signature_class`: `nums_to_num`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Current Coverage
1. direct scalar arguments coerce numeric text and logicals through the worksheet aggregate direct-argument policy.
2. direct arrays and reference-derived values use range-like policy: numeric cells contribute, text/logical/blank cells are ignored, and worksheet errors propagate.
3. missing arguments and empty cells in direct-scalar position contribute zero.
4. the adapter preserves provenance explicitly enough to distinguish direct-scalar and range-like treatment.

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/sum.rs`
2. Lean: `formal/lean/OxFunc/Functions/Sum.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 1)
