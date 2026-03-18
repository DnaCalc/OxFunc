# Function Slice Contract (Prelim) - OP_SPILL_REF(#)

## 1. Slice Identity
1. `function_id`: `FUNC.OP_SPILL_REF`
2. `display_name`: `OP_SPILL_REF`
3. `surface_syntax`: postfix operator `<anchor>#`
4. `owner_lane`: `OxFunc`
5. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - exact: `1`
2. admission policy:
   - admitted for spill-anchor formation from a reference-like anchor expression.
   - the current OxFunc runtime slice expects OxFml to admit only anchor-like operands.
3. current admitted runtime slice:
   - A1-style single-cell anchors,
   - already-tagged spill-anchor references,
   - non-A1 symbolic anchor names passed through as spill-anchor text.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `ref_only`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Preparation Policy
1. operand reference identity must remain visible to the adapter.
2. OxFunc does not resolve spill size or spill existence in this operator; it forms spill-anchor reference identity.
3. if the operand is an A1-style reference, single-cell anchor shape is required for the current runtime slice.

## 5. Core Outcome Model
1. successful evaluation returns `EvalValue::Reference` with `ReferenceKind::SpillAnchor`.
2. an ordinary anchor such as `B1` becomes spill-anchor target `B1#`.
3. an already-spill-anchor target remains stable.
4. spill existence and spill-range materialization are downstream resolver concerns, not operator-formation concerns.
5. multi-cell A1 areas are rejected as invalid anchors in the current runtime slice.

## 6. Post-call Adaptation Policy
1. successful evaluation returns spill-anchor reference identity, not dereferenced array payload.
2. invalid anchor shape or missing reference operand surface to worksheet `#REF!`.
3. downstream dereference failures remain resolver-level `#REF!` behavior.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local reference baseline: Excel `16.0 (build 19725)`, dynamic-array-capable baseline through existing coercion and OxFml pass-2 evidence.
2. Workbook Compatibility Version scope:
   - current note relies on the modern dynamic-array baseline; compatibility-lane spill-operator roundtrips remain open.

## 8. Evidence Posture
1. `spec_anchor`:
   - `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md` section 9 operator catalog mentions `OP_SPILL_REF`.
2. empirical anchors:
   - `CO4-015` in `docs/function-lane/COERCION_SCENARIO_MANIFEST_SEED.csv`
   - `P2-FML-008` in OxFml pass-2 formula-language seeds
3. current status rationale:
   - existing program evidence already depends on spill-anchor reference identity as distinct from array payload,
   - this slice closes the missing OxFunc runtime/formal/operator catalog entry needed by W14.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/op_spill_ref.rs`
2. Lean: `formal/lean/OxFunc/Functions/SpillRef.lean`
3. related W14 notes:
   - `docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md`
   - `docs/function-lane/FUNCTION_SLICE_OP_IMPLICIT_INTERSECTION_CONTRACT_PRELIM.md`
