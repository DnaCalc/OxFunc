# Function Slice Contract (Prelim) - OP_IMPLICIT_INTERSECTION(@)

## 1. Slice Identity
1. `function_id`: `FUNC.OP_IMPLICIT_INTERSECTION`
2. `display_name`: `OP_IMPLICIT_INTERSECTION`
3. `surface_syntax`: unary operator `@<expr>`
4. `owner_lane`: `OxFunc`
5. `status`: `function-phase-complete`
6. `primary_substrate_candidate`: `ContextualScalarization`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `1`
2. admission policy:
   - the current investigation slice covers explicit unary `@` in worksheet formula expressions.
   - legacy alias forms `SINGLE(...)` and `_xlfn.SINGLE(...)` are treated as compatibility/serialization representations, not separate modern semantic operators.
3. current admitted investigation focus:
   - scalar passthrough,
   - range/reference scalarization relative to caller context,
   - array-payload top-left scalarization,
   - spill-anchor and compatibility-roundtrip interactions.
4. prework assumption:
   - `INDEX`, `INDIRECT`, `OFFSET`, and `XLOOKUP` reference-return lanes are treated as supporting reference-family substrate that `@` consumes rather than redefines.
   - `OP_SPILL_REF` exists in the overall architecture, but current seam doctrine assumes OxFml normally resolves `#` to a concrete spill region or error before OxFunc evaluation.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `caller_context`
9. `surface_fec_dependency_profile`: `caller_context`
10. `compat_version_policy`: `version_scoped`
11. `compile_eval_class`: `runtime_context_dependent`

## 4. Proposed Core Outcome Model
Current current-baseline model:
1. evaluate the operand without erasing whether it is:
   - already scalar,
   - a reference/range result,
   - an array payload,
   - a spill-anchor/spill-capable result.
2. if the operand yields a single item, `@` returns that item unchanged.
3. if the operand yields a range/reference result, `@` applies caller-context-dependent scalarization:
   - same-row or same-column selection for the current formula anchor,
   - current admitted 2-D range lane on the native baseline yields `#VALUE!`.
4. if the operand yields an array payload rather than a reference identity, `@` scalarizes to the top-left item.
5. explicit `@` is a modern surface marker for implicit-intersection behavior that older Excel performed silently.
6. removing `@` from a spill-capable expression changes the publication surface from scalarization to spill, subject to host spill rules.
7. `SINGLE` / `_xlfn.SINGLE` aliasing is compatibility metadata only; it does not create a second semantic operator, and the current host baseline normalizes `_xlfn.SINGLE(...)` back onto explicit `@` in `.Formula2`.

## 5. Pre-call Preparation Requirements
OxFunc cannot implement this operator correctly from plain values alone.

Required prepared-operand distinctions:
1. explicit presence of `@` must survive parse/bind into evaluation metadata, even when stored-form normalization later omits the token.
2. caller context must be available:
   - sheet/workbook scope as needed,
   - caller row,
   - caller column.
3. operand provenance must distinguish:
   - scalar value,
   - reference identity,
   - array payload.
4. workbook compatibility/version and feature-gate context must be visible because alias/serialization behavior is version-scoped.
5. current doctrine does not require a spill-provenance bit at the OxFml -> OxFunc seam unless later evidence proves an OxFunc semantic that depends on it.

## 6. Required OxFml Contract Support
OxFml-side requirements implied by this slice:
1. parser/binder must preserve unary `@` as an explicit node or equivalent provenance marker until evaluation semantics are decided.
2. stored-form normalization policy must be reported separately from semantic preservation.
3. prepared results must distinguish:
   - scalar value,
   - reference result,
   - array payload.
4. explicit `@` application must remain observable in traces or evaluation metadata; otherwise OxFunc cannot audit scalarization-vs-spill behavior.
5. structured-reference `[@Col]` syntax and unary `@` must remain distinguishable in grammar/bind layers even though both relate to row-context selection.
6. current working split for `#` is:
   - OxFml resolves spill-anchor syntax into the current spill region or error,
   - OxFunc consumes the resolved artifact class unless a later spill-sensitive function-semantic case proves that provenance must cross the seam.

## 7. Required FEC/F3E Support
FEC/F3E requirements implied by this slice:
1. caller-context capability at admitted execution time.
2. traceable scalarization decision metadata:
   - explicit `@` present or absent,
   - operand class,
   - caller anchor,
   - selection route,
   - resulting scalar/reference/error class.
3. capability/failure accounting must remain separate from function-semantic outcomes:
   - missing caller context or denied spill/reference facilities are seam-level failures, not silent semantic collapse.

## 8. Candidate Call-Mode Seam
The current working view is more precise than "OxFunc needs the whole parse tree".

1. OxFunc does not necessarily need arbitrary parse-tree access if OxFml lowers the relevant node shape into an explicit evaluation request.
2. for a bound function-call operand, a candidate seam shape is:
   - explicit `@` / implicit-intersection mode flag,
   - bound function id/profile visibility,
   - caller context,
   - ordinary input args with scalar/reference/array/error class preserved,
   - explicit result artifact returned for scalarization.
3. this would let OxFunc decide, using function metadata already owned locally:
   - whether scalarization must affect argument admission,
   - whether it instead applies to the result,
   - how to handle reference-vs-array-vs-spill outputs.
4. however, a simple boolean mode is not sufficient for the full language because `@` also applies to non-function operands such as:
   - `@A1:A10`
   - `@SEQUENCE(3)`
   - `@A1#`
5. current narrower doctrinal split is:
   - semantic source of truth: `OP_IMPLICIT_INTERSECTION` remains a unary operator over enriched evaluation artifacts,
   - implementation optimization: OxFml may lower wrapped function-call nodes into an OxFunc call-mode request when that preserves the same semantics and trace fidelity,
   - spill-anchor syntax `#` is resolved on the OxFml side by default rather than forcing spill provenance into OxFunc.
6. any such mode seam must still keep explicit `@` provenance observable in traces and replay artifacts.

## 9. OxFunc Runtime Implementation Sketch
Current recommended implementation path:
1. add `FUNC.OP_IMPLICIT_INTERSECTION` metadata to the OxFunc function catalog and surface dispatcher.
2. introduce a small operand-class carrier for this operator, rather than routing it through existing values-only helpers.
3. implement one surface helper that consumes:
   - caller context,
   - explicit operand provenance,
   - reference/spill identity,
   - workbook compatibility metadata.
4. keep the first Rust kernel deliberately narrow:
   - scalar passthrough,
   - caller-context selection from bounded A1/area/spill inputs,
   - array top-left selection,
   - typed open-lane errors where empirical behavior is not yet pinned.
5. do not bury alias/serialization behavior in the kernel; keep that at parse/storage/interop boundary depth.

## 10. Lean Characterization Path
Recommended Lean shape for the admitted slice:
1. substrate module:
   - `formal/lean/OxFunc/Semantics/ContextualScalarization.lean`
2. function binding module:
   - `formal/lean/OxFunc/Functions/ImplicitIntersection.lean`
3. minimal executable model inputs:
   - caller row/column,
   - operand class (`scalar | ref_range | array_payload | spill_ref`),
   - bounded range/array shape,
   - compatibility/explicitness metadata where it affects semantics.
4. high-value initial theorems/examples:
   - scalar passthrough,
   - same-row reference selection,
   - same-column reference selection,
   - array top-left scalarization,
   - alias metadata does not change modern semantic outcome for equivalent operands.

## 11. Test and Characterization Plan
### 10.1 Parser and Stored-Form Lanes
1. `=@A1:A3`
2. `=@A1#`
3. `=@SEQUENCE(3)`
4. `=A1:A10+@A1:A10`
5. table-form discriminator:
   - `=[@Amount]*2`
   - `=@Tbl1[Amount]` if admitted by Excel in current baseline

Goals:
1. acceptance/rejection behavior,
2. stored formula text,
3. whether explicit `@` survives, normalizes away, or rewrites on save/open.

Current seeded storage result on the native baseline:
1. `.Formula` normalizes away explicit `@` for the seeded `=@A1:A3`, `=@A1:C1`, `=@SEQUENCE(3)`, `=@B1#`, and `=@OFFSET(...)` lanes.
2. `.Formula2` preserves explicit `@` on those same lanes.

### 10.2 OxFunc Semantic Lanes
1. scalar passthrough:
   - `=@A1` where `A1` is scalar
2. same-row reference selection:
   - `D5:=@A1:A10` should select row-aligned element if admitted by baseline
3. same-column reference selection:
   - `E7:=@A1:J1`
4. array top-left:
   - `=@SEQUENCE(2,2)`
5. spill-anchor selection:
   - `=@A1#` with a bounded spill source
6. reference-returning expression inputs:
   - `=@OFFSET(A1,...)`
   - `=@INDEX(B1:C3,0,1)` or other reference-returning bounded cases once admitted precisely
7. wrapper exception lane:
   - old-form formulas under `SUM(...)` / `AVERAGE(...)` should confirm where automatic `@` insertion does not happen.

### 10.2a Legacy CSE Interaction Lanes
1. ordinary formula:
   - `=@A1:A10`
2. legacy CSE array formula over a selected output range:
   - `{=A1:A10}`
3. dynamic-array producer in ordinary mode:
   - `=@SEQUENCE(3)`
4. dynamic-array producer in compatibility / old-Excel roundtrip contexts where available.
5. mixed-form probe:
   - `=A1:A10+@A1:A10`

Goals:
1. distinguish ordinary implicit-intersection scalarization from array-calculation-throughout behavior.
2. confirm whether explicit `@` inside legacy-array or mixed contexts is preserved, rewritten, warned on, or rejected by the host.
3. keep CSE behavior as a separate context axis rather than folding it into ordinary `@` rules.

### 10.3 Compatibility and Roundtrip Lanes
1. modern authoring with explicit `@`, then open in pre-dynamic-array Excel compatibility context.
2. old-form legacy formulas opened in dynamic-array Excel and observed for automatic `@` insertion.
3. mixed-form formulas that can serialize as `_xlfn.SINGLE(...)`.

### 10.4 FEC/F3E Trace Lanes
1. scalarization decision recorded distinctly from spill events.
2. caller-context selection recorded distinctly from plain scalar passthrough.
3. capability-denied caller-context/spill-reference paths recorded as seam failures, not semantic successes.

## 11. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - current local reference baseline starts from the existing OxFml/Foundation dynamic-array evidence pack plus local Excel `16.0.19822.20114` observations.
2. Workbook Compatibility Version scope:
   - `default` current reference baseline.

## 12. Evidence Posture
1. `spec_anchor`:
   - `ECS-004`
   - `ECS-007`
   - `ECS-008`
2. OxFml conformance anchors:
   - `FML-R-003`
   - `FML-R-004`
   - `FML-R-005`
   - `P2-FML-008`
3. OxFunc anchors:
   - `FDEF-018`
   - `docs/PARKED_CURRENT_BASELINE_20260401.md`
4. supporting program notes:
   - `docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md`
   - `docs/function-lane/W14_IMPLICIT_INTERSECTION_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W14_EXECUTION_RECORD.md`
   - `docs/upstream/NOTES_FOR_OXFML.md`
   - `docs/handoffs/HANDOFF_W014_IMPLICIT_INTERSECTION_TO_OXFML.md`
   - `docs/HISTORY.md` (`W014` wave-1 archive entry)

Current status rationale:
1. the admitted current-baseline scalarization kernel is implemented in Rust and aligned in Lean,
2. native Excel replay now pins same-row, same-column, array top-left, spill-source, reference-returning, two-dimensional `#VALUE!`, explicit-`@` storage, and `_xlfn.SINGLE(...)` normalization lanes,
3. OxFml preserves both explicit `@` and legacy-single compatibility semantics without requiring a second OxFunc runtime path,
4. no known function-semantic gap remains in declared current-phase scope.

## 13. Orthogonal Future Validation Lanes
1. broader pre-dynamic-array compatibility-version mapping beyond the current reference baseline
2. structured-reference interaction beyond the already-known `[@Col]` lexical overlap
3. exact OxFml -> OxFunc lowering shape if `@` is exposed as a bound function-call evaluation mode for some operand classes as an optimization rather than the current semantic model
