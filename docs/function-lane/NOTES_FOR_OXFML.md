# Notes for OxFml

Status: `active`
Owner lane: `OxFml`
Relationship: interface and design feedback from OxFunc function-semantics work

## 1. Purpose
Capture concrete design constraints that OxFunc has discovered while trying to implement full empirical Excel function semantics.

This is not a speculative architecture note. It is a handoff ledger for cases where OxFunc needs OxFml to preserve distinctions that are currently at risk of being erased too early in parse/bind/evaluation.

## 2. Core Message
OxFml should preserve provenance and expression-class information much longer than a plain “evaluated value” model.

For a number of Excel functions, especially aggregates and reference-returning functions, semantic correctness depends not only on the resulting value, but also on where that value came from.

## 3. Provenance Requirement for Aggregate Families
The immediate pressure point is aggregate semantics.

At minimum, OxFml should preserve a distinction between:
1. direct scalar argument from the formula AST
2. direct array literal / array constant available at parse time
3. reference-derived scalar
4. reference-derived area/array
5. eval-time reference-returning expression result:
   - examples: `OFFSET(...)`, `INDEX(...)`, `XLOOKUP(...)`
6. spilled / dynamic-array expression result that is not the same thing as an array literal
7. omitted argument, blank cell, empty string, and error provenance without collapsing them into one generic scalar bucket

Why this matters:
1. `SUM("2",TRUE)` is not the same source class as `SUM({"2",TRUE})`.
2. `SUM(A1:A2)` is not the same source class as either of the above.
3. `AVERAGE`, `COUNT`, `COUNTA`, and `AND` also depend on these distinctions.
4. If OxFml erases provenance during parse, bind, normalization, or early evaluation, OxFunc cannot recover the correct Excel family policy later.

## 4. Parse-Tree and Evaluation Implication
OxFml should keep provenance attached all the way from parse tree through reference-tree build and evaluation results.

Practical implication:
1. do not normalize array literals and reference-derived arrays into the same opaque “array value” too early
2. do not normalize reference-returning expressions into plain values when reference identity is semantically relevant
3. prepared-call surfaces should eventually consume something closer to:
   - `value + provenance + reference_identity + production_mode`
   rather than only:
   - `value`

## 5. Draft Interface Sketch
This section proposes a minimum vocabulary and handoff shape that OxFunc could consume from OxFml.

The goal is not to freeze final Rust types here. The goal is to make sure the interface preserves the distinctions that Excel semantics require.

### 5.1 Candidate Expression-Source Vocabulary
Suggested expression/argument source classification:
1. `DirectScalar`
   - scalar expression supplied directly in the AST
   - examples: `1`, `"a"`, `TRUE`, `A1+1`
2. `ArrayLiteral`
   - array constant available from the formula AST
   - examples: `{1,2}`, `{"2",TRUE}`
3. `ReferenceNode`
   - syntactic reference expression before dereference
   - examples: `A1`, `A1:B2`, `Sheet2!C3`
4. `ReferenceDerivedScalar`
   - scalar obtained by resolving a reference or implicit intersection
5. `ReferenceDerivedArray`
   - area/array obtained by resolving a reference node
6. `ReferenceReturningExpr`
   - non-reference syntax that evaluates to a reference identity
   - examples: `OFFSET(...)`, `INDEX(...)`, `XLOOKUP(...)`, `INDIRECT(...)`
7. `SpillExprResult`
   - array result produced by expression evaluation rather than by array-literal syntax
8. `Omitted`
   - omitted argument position

These can be represented either as a single enum or as orthogonal fields. The important point is that OxFunc can still distinguish array literal from reference-derived array, and both from eval-time reference-returning results.

### 5.2 Candidate Provenance Carrier
Candidate boundary shape:

```text
PreparedArg {
  value_view: ValueView,
  provenance: ValueProvenance,
  source_class: ExprSourceClass,
  reference_identity: Option<ReferenceIdentity>,
  evaluation_mode: EvaluationMode,
  blankness_class: BlanknessClass,
}
```

Candidate field intent:
1. `value_view`
   - the observed scalar/array/error payload currently visible to the callee
2. `provenance`
   - how the payload was produced
3. `source_class`
   - the originating syntactic/evaluative class
4. `reference_identity`
   - stable identity for reference-bearing results when semantically relevant
5. `evaluation_mode`
   - whether the argument arrived eagerly, lazily, or reference-preserved
6. `blankness_class`
   - omitted vs blank cell vs empty string without premature collapse

Candidate provenance vocabulary:
1. `AstScalar`
2. `AstArrayLiteral`
3. `ReferenceResolvedScalar`
4. `ReferenceResolvedArea`
5. `ReferenceReturnedAtEval`
6. `SpillProduced`
7. `AdapterSynthesized`

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
  provenance: ValueProvenance,
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

If those arrive at OxFunc as the same generic array/scalar bucket, the boundary is already too lossy.

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
1. array literal is not interchangeable with reference-derived area
2. reference-returning expression is not interchangeable with an already-dereferenced payload
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
