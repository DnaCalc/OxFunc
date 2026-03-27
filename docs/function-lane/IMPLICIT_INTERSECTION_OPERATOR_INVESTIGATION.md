# Implicit Intersection Operator Investigation

Status: `active`
Owner lane: `OxFunc`
Workset: `W14`

## 1. Current Status
Execution state:
1. `in_progress`

Completeness axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`

Open lanes:
1. compatibility-version and legacy serialization behavior are only partially characterized.
2. structured-reference/table-context interaction remains outside the admitted slice.
3. the reference-family prework inventory is now explicit:
   - `INDEX`, `INDIRECT`, `OFFSET`, and `XLOOKUP` reference-return are already closed in OxFunc,
   - `OP_SPILL_REF` is now explicit in Rust/Lean/docs,
   - current seam doctrine does not require spill-link provenance to cross into OxFunc,
   - `OP_IMPLICIT_INTERSECTION` still remains open.

## 2. Current Evidence Baseline
Existing repo anchors already establish the operator as a real OxFunc concern:
1. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md` section 9 assigns canonical identity `OP_IMPLICIT_INTERSECTION`.
2. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv` row `FDEF-018` already treats `@` as a canonical operator-function with `SINGLE` alias metadata.
3. `../OxFml/docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md` rule `FML-R-003` requires parse retention of explicit `@`.
4. OxFml pass-2 notes already show normalization-sensitive cases:
   - `=@A1#` stored as `=A1#`,
   - `=@SEQUENCE(3)` stored as `=SEQUENCE(3)`.

Current official-support summary captured in the Foundation evidence corpus:
1. if the operand already yields a single item, `@` is observationally a no-op.
2. if the operand yields a range, implicit intersection selects the cell on the same row or column as the formula.
3. if the operand yields an array payload, implicit intersection selects the top-left item.
4. removing `@` from a range/array-returning expression can change worksheet behavior from scalarization to spill.
5. formulas authored with explicit `@` in modern dynamic-array Excel may appear as `_xlfn.SINGLE(...)` in pre-dynamic-array Excel.

Reference-seam prework inventory relevant to `@`:
1. `INDEX` already preserves reference identity and row/column slice semantics.
2. `OFFSET` already constructs shifted/resized references without dereferencing them.
3. `INDIRECT` already interprets text into reference identity with caller-context support for relative R1C1.
4. `XLOOKUP` already preserves reference-return identity for aligned slices.
5. `OP_SPILL_REF` is now explicit in the architecture, but current working doctrine is that OxFml should normally resolve `A1#` into the current spill region or error before handing control to OxFunc.
6. no current evidence shows that OxFunc-owned semantics depend on preserving "came from #"
   separately from the resolved operand class (`reference | array | scalar | error`).

Quick local confirmation on `2026-03-14`:
1. with `B1:=SEQUENCE(3)`, the following pairs were observationally identical in the current Excel baseline:
   - `@B1#` versus `@B1:B3`
   - `SINGLE(OX_TRACE_Q1(B1#))` versus `SINGLE(OX_TRACE_Q1(B1:B3))`
   - `SINGLE(OX_PROBE_ECHO(B1#))` versus `SINGLE(OX_PROBE_ECHO(B1:B3))`
2. observed rows aligned identically (`1`, `2`, `3`) and non-aligned rows failed identically with `#VALUE!`.
3. this does not prove equivalence for all future cases, but it is current positive evidence against carrying spill-link provenance into OxFunc by default.

Native replay confirmation on `2026-03-22` from `.tmp/w14-implicit-intersection-results.csv`:
1. `=@A1:B2` at `C3` currently yields `#VALUE!`.
2. `=@B1#` with `B1:=SEQUENCE(3)` currently yields `1`.
3. `.Formula` strips explicit `@` on the seeded lanes:
   - `=@A1:A3` stores as `=A1:A3`
   - `=@SEQUENCE(3)` stores as `=SEQUENCE(3)`
4. `.Formula2` preserves explicit `@` on the same lanes.

## 3. OxFunc Semantic Fit
Current substrate vocabulary does not fit `@` cleanly.

Nearest existing homes and why they are insufficient:
1. `ReferenceSelectionReturn`
   - too reference-return-oriented; `@` produces a scalarized value rather than preserving reference identity as its final result.
2. `ReferenceConstruction`
   - too focused on building references (`OFFSET`-style) rather than selecting a scalar from an existing reference or array-like source.
3. `ArrayShapeConstruction`
   - backwards for `@`; the operator collapses potential spill shape instead of constructing it.

Working recommendation:
1. add a new primary substrate class for this family, for example `ImplicitIntersectionScalarization`.
2. until the taxonomy is extended, treat `@` as a reference-sensitive/caller-context operator whose output semantics depend on preserving the distinction between reference input and array payload input.

## 4. Candidate OxFunc Contract Shape
Suggested starting profile for `OP_IMPLICIT_INTERSECTION`:
1. `function_id`: `FUNC.OP_IMPLICIT_INTERSECTION`
2. `display_name`: `OP_IMPLICIT_INTERSECTION`
3. arity:
   - minimum: `1`
   - maximum: `1`
4. `determinism_class`: `deterministic`
5. `volatility_class`: `nonvolatile`
6. `host_interaction_class`: `workbook_state`
7. `thread_safety_class`: `host_serialized`
8. `arg_preparation_profile`: `refs_visible_in_adapter`
9. `coercion_lift_profile`: `custom`
10. `kernel_signature_class`: `custom`

FEC dependency posture:
1. current OxFunc enum vocabulary is awkward for `@`.
2. at minimum the surface needs:
   - caller anchor/context,
   - reference-resolution capability,
   - spill/reference provenance,
   - dynamic-array/compatibility feature awareness.
3. current `FecDependencyProfile` likely needs either:
   - `composite`, or
   - a richer facility-tag story carried beside the enum.

## 5. Candidate Runtime Shape In OxFunc
`@` should not be implemented as a generic "take first element" helper.

Recommended runtime shape:
1. add a dedicated surface module:
   - `crates/oxfunc_core/src/functions/op_implicit_intersection.rs`
2. keep `arg_preparation_profile = RefsVisibleInAdapter`.
3. introduce an explicit prepared source classification, for example:

```text
ImplicitIntersectionSource =
  | Scalar(EvalValue)
  | Reference(ReferenceLike)
  | Array(EvalArray)
  | SpillReference(ReferenceLike)
  | StructuredReference(ReferenceLike)
```

4. use caller anchor data when the source is reference-like.
5. use top-left selection only when the source is already an array payload.

Why this split matters:
1. the current value universe already allows both `Array` and `ReferenceLike` in `CallArg` and raw return domains.
2. if OxFml or shared pre-adapter preparation dereferences a reference into an `EvalArray` too early, OxFunc loses the official range-vs-array distinction and cannot model `@` honestly.

Current OxFunc seams that are reusable:
1. `crates/oxfunc_core/src/value.rs`
   - already distinguishes `EvalValue::Array` from `EvalValue::Reference`.
2. `crates/oxfunc_core/src/resolver.rs`
   - already carries `CallerContext { prefix, row, col }` and resolver capabilities.
3. `crates/oxfunc_core/src/functions/offset.rs`
   - shows the `refs_visible_in_adapter` pattern for a reference-sensitive operator/function.
4. `crates/oxfunc_core/src/functions/row_fn.rs`
   - shows direct caller-context consumption.
5. `crates/oxfunc_core/src/functions/adapters.rs`
   - shows why `values_only_pre_adapter` is the wrong seam for `@`.

Likely runtime extension points:
1. a helper that intersects a parsed reference with the caller anchor.
2. a provenance-preserving prepared-arg carrier for scalarization-sensitive operators.
3. optional extension of `CallerContext` if row/col/prefix is not enough for structured or workbook-sensitive cases.
4. dispatch registration in `surface_dispatch.rs`.

## 6. What OxFml And FEC Need To Preserve
`@` makes one upstream requirement unavoidable: OxFunc must be able to tell whether it received a range/reference-like source or an already materialized array payload.

Minimum upstream requirements:
1. parse/bind must retain explicit `@` as an evaluable node, not just as formatting/normalization trivia.
2. the prepared-call surface must preserve whether the operand reached `@` as:
   - a reference-like expression,
   - a spill-anchor/spilled-range reference,
   - an array payload.
3. caller anchor must be available at the point where scalarization happens.
4. traces must record whether scalarization happened because of explicit `@` versus ordinary dereference/publication behavior.
5. compatibility/dynamic-array mode must be visible somewhere in the seam or fixed earlier with explicit provenance.

Practical implication for OxFml/FEC:
1. if OxFml chooses to evaluate `@` upstream, it still needs to emit a trace/provenance record that preserves the distinction.
2. if OxFml chooses to leave `@` as an operator-function for OxFunc, the prepared operand must remain rich enough for OxFunc to apply the documented rules.

Current working recommendation:
1. preserve `@` as an explicit bound/evaluable node in OxFml.
2. keep scalarization decision semantics near the OxFunc operator-function unless and until OxFml/FEC can expose the same provenance and trace fidelity.

## 7. Candidate Lean Characterization
Recommended Lean shape follows the current executable-semantic-model strategy:
1. substrate module:
   - `formal/lean/OxFunc/Scalarization.lean` or similar
2. function binding module:
   - `formal/lean/OxFunc/Functions/ImplicitIntersection.lean`

Candidate model vocabulary:
1. `CallerAnchor`
   - `row`, `col`, optional sheet/workbook qualifier.
2. `IntersectionSource`
   - `scalar`
   - `rangeRef`
   - `arrayPayload`
   - `spillRef`
3. `SelectionResult`
   - selected scalar
   - explicit worksheet error

Candidate executable function:

```text
evalImplicitIntersection :
  CallerAnchor -> IntersectionSource -> Except WorksheetErrorCode CoercionInput
```

High-value Lean obligations:
1. determinism:
   - same caller anchor and same source produce the same result.
2. scalar idempotence:
   - scalar source passes through unchanged.
3. array top-left rule:
   - array payload selection equals the top-left element when present.
4. caller-relative reference rule:
   - one-dimensional row/column selections follow the caller anchor.
5. preparation-profile theorem:
   - the `@` binding depends on a refs-visible surface and is not equivalent to values-only pre-adapter preparation.

## 8. Test And Replay Plan
Testing has to span three distinct layers.

### 8.1 OxFunc Runtime Unit Tests
Seed unit-test lanes:
1. scalar passthrough.
2. single-column reference selected by caller row.
3. single-row reference selected by caller column.
4. array payload selects top-left.
5. spill-reference source remains distinct from array payload source.
6. caller-context-missing failure path.

### 8.2 OxFml Parse And Normalization
Seed upstream lanes:
1. `=@A1:A3` parse retention.
2. `=@A1#` parse retention and normalization capture.
3. `=@SEQUENCE(3)` parse retention and normalization capture.
4. explicit `@` mixed with reference-returning expressions such as `=@OFFSET(...)`.

### 8.3 Excel Empirical Replay
Seed replay matrix lives in:
1. `docs/function-lane/W14_IMPLICIT_INTERSECTION_SCENARIO_MANIFEST_SEED.csv`

Primary empirical lanes:
1. caller-row selection from a single-column range.
2. caller-column selection from a single-row range.
3. top-left selection from array literal/dynamic-array payload.
4. reference-returning expression consumed by `@`.
5. spill-anchor source consumed by `@`.
6. unresolved 2-D range lane with explicit open expectation.
7. compatibility/normalization lane for explicit `@` storage.

## 9. Current Recommendation
Current recommendation for the implementation spike:
1. do not route `@` through `values_only_pre_adapter`.
2. do not model `@` as generic top-left scalarization.
3. do not let OxFml erase reference-vs-array provenance before `@` semantics are decided.
4. treat `@` as a boundary-pressure operator that connects formula language, FEC preparation, function semantics, and spill publication.
5. current evidence now supports a sharper reading:
   - the admitted OxFunc-side runtime and formal slice is already in place,
   - the OxFml adapter has exercised that slice end-to-end on the seeded corpus,
   - so the live residual is compatibility/serialization characterization rather than a missing OxFunc kernel.

## 10. Next OxFunc Actions
1. keep compatibility-version and `_xlfn.SINGLE(...)` roundtrip characterization open as the next replay pressure.
2. treat the current OxFml adapter evidence for `@` as integration-level proof for the admitted modern/current-baseline slice.
3. drive remaining seam changes only from concrete OxFml/FEC consumer mismatches or compatibility-version evidence.


