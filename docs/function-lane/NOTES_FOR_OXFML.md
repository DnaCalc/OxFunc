# Notes for OxFml

Status: `active`
Owner lane: `OxFml`
Relationship: interface and design feedback from OxFunc function-semantics work

## 1. Purpose
Capture concrete design constraints that OxFunc has discovered while trying to implement full empirical Excel function semantics.

This is not a speculative architecture note. It is a handoff ledger for cases where OxFunc needs OxFml to preserve distinctions that are currently at risk of being erased too early in parse/bind/evaluation.

## 2. Core Message
OxFml should preserve the distinctions that Excel semantics actually depend on for the current function set.

The most important current split is not “all provenance everywhere”, but:
1. direct scalar argument versus array-like argument
2. value-only function versus reference-observable function
3. value-only result versus may-return-reference result

## 3. Current Aggregate Requirement
The immediate pressure point is aggregate semantics.

For the currently closed `SUM` slice, OxFunc needs to preserve:
1. direct scalar argument from the call structure
2. array-like argument as a single argument
3. omitted argument, blank cell, empty string, and error classes without collapsing them too early

Why this matters:
1. `SUM("2",TRUE)` is not the same as `SUM({"2",TRUE})`.
2. current evidence does not require `SUM` to distinguish reference-derived arrays from array literals once both have become array-like inputs.
3. `AVERAGE`, `COUNT`, `COUNTA`, and `AND` may later require richer distinctions, but those should be demanded only when we have actual examples for them.

Current OxFunc status:
1. OxFunc now carries explicit aggregate input-structure classes for `SUM` (`direct_scalar`, `direct_array_literal`, `reference_derived`, `opaque_array_value`).
2. The current `SUM` semantics use those classes to preserve direct-scalar versus array-like behavior; they do not currently require a worksheet-semantic distinction between array literal and reference-derived array.
3. When upstream source structure is missing, OxFunc uses an explicit `opaque_array_value` fallback rather than silently pretending the source class is known.

## 4. Parse-Tree and Evaluation Implication
OxFml should keep provenance attached all the way from parse tree through reference-tree build and evaluation results.

Practical implication:
1. do not collapse direct scalar arguments and array-like arguments into the same generic prepared shape for aggregate-style functions
2. do not normalize reference-returning expressions into plain values when reference identity is semantically relevant
3. prepared-call surfaces should eventually consume something closer to:
   - `value + structure_class + reference_identity + production_mode`
   rather than only:
   - `value`

## 5. Draft Interface Sketch
This section proposes a minimum vocabulary and handoff shape that OxFunc could consume from OxFml.

The goal is not to freeze final Rust types here. The goal is to make sure the interface preserves the distinctions that Excel semantics require.

### 5.1 Candidate Expression-Source Vocabulary
Suggested expression/argument source classification:
1. `DirectScalar`
   - scalar expression supplied directly in the call structure
   - examples: `1`, `"a"`, `TRUE`, `A1+1`
2. `ArrayLikeValue`
   - array-like input presented to the callee as one argument
   - examples: `{1,2}`, a dereferenced range, a spill result
3. `ReferenceNode`
   - syntactic reference expression before dereference
   - examples: `A1`, `A1:B2`, `Sheet2!C3`
4. `ReferenceReturningExpr`
   - non-reference syntax that evaluates to a reference identity
   - examples: `OFFSET(...)`, `INDEX(...)`, `XLOOKUP(...)`, `INDIRECT(...)`
5. `Omitted`
   - omitted argument position

These can be represented either as a single enum or as orthogonal fields. The important point is that OxFunc can still distinguish direct scalar input, array-like input, and reference-observable input/result cases.

### 5.2 Candidate Provenance Carrier
Candidate boundary shape:

```text
PreparedArg {
  value_view: ValueView,
  structure_class: StructureClass,
  source_class: ExprSourceClass,
  reference_identity: Option<ReferenceIdentity>,
  evaluation_mode: EvaluationMode,
  blankness_class: BlanknessClass,
}
```

Candidate field intent:
1. `value_view`
   - the observed scalar/array/error payload currently visible to the callee
2. `structure_class`
   - whether the argument is a direct scalar, array-like input, omitted, or another semantically distinct prepared shape
3. `source_class`
   - the originating syntactic/evaluative class
4. `reference_identity`
   - stable identity for reference-bearing results when semantically relevant
5. `evaluation_mode`
   - whether the argument arrived eagerly, lazily, or reference-preserved
6. `blankness_class`
   - omitted vs blank cell vs empty string without premature collapse

Candidate minimal structure vocabulary:
1. `DirectScalar`
2. `ArrayLike`
3. `Omitted`
4. `AdapterSynthesized`

Optional richer source vocabulary can sit beside this when later functions prove that it matters.

### 5.3 Candidate Reference Identity Carrier
OxFunc likely needs more than a formatted address string.

Candidate sketch:

```text
ReferenceIdentity {
  workbook_scope: WorkbookScopeId,
  sheet_scope: SheetScopeId,
  area_shape: AreaShape,
  anchor: ReferenceAnchor,
  address_mode: AddressMode,
  area_kind: AreaKind,
}
```

Important properties:
1. reference identity should survive through functions that return references
2. identity should not depend only on formatted A1 text
3. the carrier should distinguish scalar cell, rectangular area, row, column, and later richer shapes if needed
4. caller-relative semantics should have a place to live for `OFFSET`, `INDIRECT`, and later info/macro functions

### 5.4 Candidate Evaluation-Mode Contract
OxFml should make evaluation strategy explicit rather than implicit.

Candidate sketch:

```text
enum EvaluationMode {
  EagerValue,
  BranchLazy,
  FallbackLazy,
  ReferencePreserved,
  Selective,
}
```

Intent:
1. `IF` needs branch laziness
2. `IFERROR` needs fallback laziness
3. `INDEX`, `OFFSET`, `INDIRECT`, and `XLOOKUP` may need reference-preserved handling
4. later functions may need selective evaluation rather than universal eager normalization

### 5.5 Candidate Function-Boundary Contract
Suggested OxFml to OxFunc boundary idea:
1. parse/bind stage emits AST with stable source classification and reference nodes intact
2. reference-build stage emits bound reference structures without erasing literal-vs-reference distinctions
3. evaluation stage emits prepared arguments/results with:
   - payload
   - provenance
   - blankness class
   - reference identity when present
   - evaluation mode
4. function adapter layer decides family policy from that richer shape

Candidate result shape:

```text
PreparedResult {
  payload: ValueView,
  result_class: ResultClass,
  structure_class: StructureClass,
  reference_identity: Option<ReferenceIdentity>,
  format_hint: Option<FormatHint>,
}
```

Candidate `ResultClass` vocabulary:
1. `ScalarValue`
2. `ArrayPayload`
3. `ReferenceResult`
4. `ErrorResult`
5. `OmittedLike`

Candidate `FormatHint` intent:
1. carry post-evaluation worksheet-surface hints without pretending the pure function kernel mutates the grid directly
2. canonical current examples are `NOW()` and `TODAY()` entered into a caller cell previously formatted as `General`
3. FEC/F3E or the surrounding engine surface can decide whether and how to apply the hint
4. XLL verification may legitimately omit this application step while still preserving the semantic characterization in OxFunc

### 5.6 Aggregate Design Test
Aggregate families are the main current design test for this boundary.

The boundary must allow OxFunc to distinguish at least:
1. `SUM("2",TRUE)`
2. `SUM({"2",TRUE})`
3. `SUM(A1:A2)`
4. `SUM(OFFSET(A1,0,0,2,1))`
5. `SUM(XLOOKUP(...))` when the lookup returns a reference-capable result

Current lesson:
1. `SUM("2",TRUE)` must stay distinct from the single-argument array-like cases.
2. the current `SUM` evidence does not require array literal versus reference-derived array to stay distinct once both are array-like inputs.
3. if later aggregate examples prove otherwise, the boundary should be able to grow richer source classes without breaking the simpler current model.

### 5.7 OFFSET Design Test
`OFFSET` should be treated as a primary interface test, not just another function.

The boundary should support:
1. base reference identity
2. caller/context anchor where required
3. row/column displacement without forced dereference
4. height/width shaping as reference formation rather than array materialization
5. downstream consumption by functions that care about reference identity, not just dereferenced values

This is likely the best immediate test for whether OxFml has a coherent reference-preserving evaluation interface.

### 5.8 Text and Formatting Design Test
Text functions are a separate pressure test on the same interface.

OxFml should avoid baking in host-language stringification. Instead it should leave room for:
1. Excel-grade scalar-to-text coercion
2. distinct treatment of blank cell vs empty string
3. array/range flattening policies that remain family-specific
4. formatting-sensitive or locale/version-sensitive textification later
5. post-evaluation format hints that are attached to function results rather than collapsed into plain scalar payloads too early

Related pressure:
1. `NOW` and `TODAY` are not just value producers; they can also emit caller-cell format hints in the observed Excel baseline
2. that hint should survive the OxFml boundary as result metadata rather than as an ad hoc side effect

### 5.9 Minimum Invariants
The following invariants should hold across the OxFml boundary:
1. direct scalar input is not interchangeable with array-like input
2. reference-returning expression is not interchangeable with an already-dereferenced payload when reference identity matters
3. omitted argument, blank cell, empty string, and error are not collapsed into one generic empty/scalar bucket
4. reference identity survives until the consuming function family has explicitly decided to dereference or flatten it
5. evaluation strategy is visible where Excel semantics depend on non-eager behavior

### 5.10 Near-Term Interface Questions
Questions OxFml should answer explicitly:
1. what is the smallest provenance vocabulary that still keeps aggregate and reference semantics correct
2. where does implicit intersection live in the pipeline, and how is its provenance recorded
3. how are spilled results represented distinctly from array literals
4. what stable identity, if any, is guaranteed for reference-returning expressions
5. which boundary owns scalar-to-text and scalar-to-number normalization decisions

## 6. Reference-Semantics Pressure
This is not only an aggregate issue.

The same substrate is required for:
1. `OFFSET`
2. `INDEX`
3. `INDIRECT`
4. `XLOOKUP`
5. spill-aware/dynamic-array functions
6. info/macro-style functions such as `CELL` and later `GET.*`

For these functions, reference identity and the route by which a reference was produced can matter semantically, not just operationally.

## 7. Evaluation Strategy Pressure
OxFml should also leave room for argument-evaluation strategy to remain explicit.

Examples:
1. `IF` requires branch laziness.
2. `IFERROR` requires fallback laziness.
3. reference-returning expressions should not be forced through eager dereference when the function contract requires reference identity instead.

So OxFml should avoid an architecture that assumes:
1. every argument is eagerly value-normalized before the function layer sees it
2. every reference-bearing expression must collapse immediately to a dereferenced value

## 8. Text-Coercion Pressure
Text-producing and text-consuming functions also expose a shared OxFml-facing need:
1. preserve enough information for Excel-grade textification and coercion
2. do not assume a generic host-language `toString` model is acceptable

This matters for at least:
1. `TEXTJOIN`
2. `EXACT`
3. `CLEAN`
4. later formatting-sensitive functions

## 9. Current OxFunc View
Current OxFunc evidence suggests that the following distinctions should be first-class in OxFml interfaces:
1. source class:
   - direct scalar
   - direct array literal
   - reference-derived
   - reference-returned
   - spilled-expression result
2. return class:
   - value-only
   - reference-possible
   - reference-required
   - array-payload
3. evaluation strategy:
   - eager
   - branch-lazy
   - fallback-lazy
   - selective reference-preserving

## 10. Immediate Follow-up Candidates
1. define a small OxFml provenance vocabulary that OxFunc can rely on
2. define a reference-identity carrier that survives parse/bind/eval
3. define the minimum prepared-call contract between OxFml and OxFunc for aggregate families
4. review `OFFSET` as the main design test for eval-time dereference versus reference preservation

## 11. Initial Seed Statement
Seed statement for future interface work:

`OxFml must preserve argument and result provenance sufficiently for OxFunc to distinguish array literals, reference-derived arrays, and eval-time reference-returning expressions, because Excel function semantics depend on those distinctions and they cannot be reconstructed after erasure.`
