# Function Slice Contract (Prelim) - OP_ADD(+)

## 1. Slice Identity
1. `function_id`: `FUNC.OP_ADD`
2. `display_name`: `OP_ADD`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `2`
2. admission policy:
   - binary operator-as-function seed for `+`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `nums_to_num`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. W10 Seed Coverage
1. scalar numeric kernel is implemented and test-covered.
2. numeric text and logical coercion lanes are admitted.
3. non-numeric text returns coercion error lane.
4. Q-binary XLL export path is added (`ox_ADD_Q`).

## 5. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/op_add.rs`
2. Lean: `formal/lean/OxFunc/Functions/OpAdd.lean`
3. XLL bridge: `crates/oxfunc_core/src/xll_export_specs.rs`, `tools/xll-addin/oxfunc_xll/build.rs`, `tools/xll-addin/oxfunc_xll/src/lib.rs`
