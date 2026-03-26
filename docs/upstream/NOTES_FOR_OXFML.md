# Notes for OxFml

Status: `active`
Owner lane: `OxFml`
Relationship: interface and design feedback from OxFunc function-semantics work

## 1. Purpose
Capture concrete design constraints that OxFunc has discovered while trying to implement full empirical Excel function semantics.

This is not a speculative architecture note. It is a handoff ledger for cases where OxFunc needs OxFml to preserve distinctions that are currently at risk of being erased too early in parse/bind/evaluation.

## 1A. Recommendation Status
These notes should now be read as semantic requirements and pressure points discovered by OxFunc, not as a frozen upstream interface prescription.

Important clarification:
1. OxFunc is about to change how upstream recommendations are carried and negotiated.
2. So this note should not be read as “OxFml must implement exactly this named type or callback boundary next”.
3. It should be read as “these distinctions must survive somewhere in the upstream parse/bind/evaluation world, and OxFunc must be able to depend on them semantically”.
4. Candidate interface sketches in this note are therefore provisional examples, not normative API commitments.

Working rule:
1. preserve the semantic requirement
2. allow the upstream transport/mechanism to change
3. keep the mapping from upstream mechanism to OxFunc semantic dependency explicit

Central seam-model note:
1. `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md`
2. `docs/function-lane/OXFML_OXFUNC_NEXT_ROUND_STABILIZATION_TOPICS.md`
3. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V1.md`
4. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V2.md`

Relationship to OxFml's current upstream position:
1. this central note is intended to align with OxFml's current "semantic requirements first, transport later" posture recorded in `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`,
2. it should not be read as a demand for any one final carrier/type shape,
3. it is the OxFunc-side attempt to make the hidden Excel machinery themes explicit and iteratively negotiable,
4. the new stabilization-topics note narrows that broader model to the few seam topics that appear most ready for concrete next-round progress.
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

## 5. Provisional Interface Sketch
This section proposes a minimum vocabulary and handoff shape that OxFunc could consume from OxFml.

The sketches below are intentionally provisional. They describe the semantic distinctions OxFunc currently needs, but the upstream mechanism carrying those distinctions may change.

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
5. raw-return normalization should remain explicit:
   - current XLL nil-propagation evidence shows scalar raw `xltypeNil` collapses to numeric-zero semantics before outer argument binding,
   - but raw `xltypeNil` array elements can remain as `empty_cell`-like element state inside intermediate arrays until scalarization/publication

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

## 6. Updated W14 Position on `@` and `#`
Current W14 narrowing from OxFunc:
1. OxFunc does not currently require arbitrary parse-tree access for `@`.
2. A plausible OxFml -> OxFunc seam is a uniform evaluation request carrying:
   - optional bound callee,
   - ordinary args/results,
   - caller context,
   - explicit-`@` / implicit-intersection mode metadata.
3. For OxFunc-owned `@` semantics, the currently required operand/result classes are:
   - scalar,
   - reference,
   - array,
   - error.
4. OxFunc does not currently require spill-link provenance as a separate semantic class.

Working split for `#`:
1. OxFml should normally discharge spill-anchor syntax (`A1#`) into the current resolved spill region or error before OxFunc evaluation.
2. OxFunc should not receive a spill-provenance flag by default.
3. This remains revisable if later evidence proves a true OxFunc-owned semantic lane that depends on “came from `#`” rather than on the resolved operand class.

Immediate implication for upstream design:
1. preserve enough structure for `@` to be explicit and caller-context-aware,
2. but do not assume OxFunc needs spill-anchor identity once `#` has already been resolved to its current reference/array/error outcome.
3. local `2026-03-14` probe evidence for `B1:=SEQUENCE(3)` found no semantic difference in the current baseline between:
   - `@B1#` and `@B1:B3`,
   - `SINGLE(OX_TRACE_Q1(B1#))` and `SINGLE(OX_TRACE_Q1(B1:B3))`,
   - `SINGLE(OX_PROBE_ECHO(B1#))` and `SINGLE(OX_PROBE_ECHO(B1:B3))`.
4. what stable identity, if any, is guaranteed for reference-returning expressions
5. which boundary owns scalar-to-text and scalar-to-number normalization decisions
6. should OxFml expose both `RawFunctionReturn` and `PublishedFormulaResult` instead of forcing one result universe for all contexts

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

## 9. Locale And Format Pressure
W13 makes the next substrate requirement explicit:
1. `VALUE` needs locale/profile-sensitive text-to-value parsing
2. `TEXT`, `DOLLAR`, and `FIXED` need Excel format-code rendering plus locale-profile symbols
3. workbook date-system selection is adjacent and should travel through the same evaluation-context world

So OxFml/FEC should own:
1. locale/profile identity
2. workbook date system
3. format-code language definition and rendering
4. locale-sensitive parse services

OxFunc should consume those through explicit declared facilities, not by embedding ad hoc locale/format logic in function kernels.

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

## 11. Current Upstream-Handoff Rule
Until the upstream recommendation mechanism is revised, use this note as follows:
1. treat the semantic distinctions in this note as the stable content
2. treat the named sketches and candidate carriers as replaceable scaffolding
3. when an upstream design changes, update this note by preserving the semantic requirement and revising the transport/example shape
4. avoid encoding accidental current OxFunc implementation details as upstream obligations unless a function-semantic requirement truly depends on them

## 12. Immediate Compact Requirement Set
If the upstream interface work is being reworked, the minimum semantic requirements discovered through W13 are now:
1. direct scalar argument must remain distinguishable from array-like argument where function semantics depend on it
2. values-only functions must remain distinguishable from references-visible functions
3. value-result-only functions must remain distinguishable from may-return-reference functions
4. blank single-cell dereference must remain distinct from empty string, missing argument, and generic array payload
5. caller-context dependence must remain explicit for functions like `ROW`, `COLUMN`, `INDIRECT`, `OFFSET`, and later `CELL`
6. locale/format services must be available as explicit facilities rather than hidden ad hoc coercion logic
7. format hints and raw-return publication rules must remain modelable above the pure function kernel

## 13. W13 Follow-Back
W13 sharpened the OxFml-facing message in three ways:
1. blank single-cell handling is not optional glue; it is a real semantic boundary requirement because `N`, `T`, and `TYPE` observe it differently
2. `ROW` and `COLUMN` show that caller-context and reference-shape semantics belong in the upstream semantic world even when the kernel itself is simple
3. whole-axis cardinality is workbook-compatibility-sensitive, so upstream context must be able to carry compatibility-sensitive sheet geometry assumptions when functions depend on them

## 14. W14 Implicit-Intersection Pressure
### 14.1 Purpose
Capture the specific upstream distinctions OxFunc now needs in order to implement the explicit implicit-intersection operator `@` without guessing from already-collapsed values.

### 14.2 Core Message
`@` is not only parser syntax. OxFunc needs OxFml to preserve whether the operand was scalar already, a reference/range result, an array payload, or a spill-linked result, and it needs caller-context visibility when scalarization depends on row/column alignment.

### 14.3 Current Evidence
1. OxFml already carries `FML-R-003` requiring `@` to survive parse normalization as explicit syntax.
2. OxFml pass-2 notes also show stored-form normalization can remove visible `@` in tested shapes (`=@A1# -> =A1#`, `=@SEQUENCE(3) -> =SEQUENCE(3)`), proving that stored text and semantic provenance are not interchangeable.
3. OxFunc already carries `FDEF-018` for canonical `OP_IMPLICIT_INTERSECTION` with legacy alias metadata and `caller_context` FEC dependency.
4. W10/W12/W13 proved that reference identity, caller context, and spill-aware result classes already matter for adjacent functions (`INDEX`, `OFFSET`, `INDIRECT`, `ROW`, `COLUMN`).

### 14.4 Interface Implications
OxFml should preserve:
1. explicit `@` provenance through parse/bind/eval even when visible stored-form text later omits the token
2. caller row/column context
3. distinction between scalar result, reference result, array payload, and spill-anchor-linked result
4. scalarization route metadata in traces so implicit-intersection outcomes are replayable

### 14.5 Minimum Invariants
1. stored-form normalization removing visible `@` must not erase the fact that implicit-intersection semantics were in force
2. explicit `@` applied to a reference result must remain distinguishable from explicit `@` applied to an already-materialized array payload
3. caller-context-dependent scalarization must not be approximated by unconditional top-left selection
4. capability-denied caller-context/spill-reference paths must remain typed seam failures, not silent semantic collapse

### 14.6 Open Questions
1. where should implicit intersection live in the execution pipeline: upstream evaluator, downstream OxFunc operator dispatch, or a split-provenance model
2. what is the smallest provenance vocabulary that preserves `@` semantics without overcommitting the general boundary
3. which compatibility/version fields are required to represent `_xlfn.SINGLE(...)` roundtrip behavior honestly

## 15. W15 Host-Query Pressure (`CELL` / `INFO`)
### 15.1 Purpose
Record the current OxFunc view of the execution-context seam needed by `CELL` and `INFO`.

### 15.2 Core Message
`CELL` and `INFO` are not pure local kernels, but they should also not be modeled as arbitrary evaluator callbacks.

The cleaner split is:
1. OxFunc owns:
   - `info_type` / `type_text` normalization,
   - Excel-specific query classification,
   - result-shaping and worksheet error policy.
2. OxFml/FEC/F3E owns:
   - actual cell metadata,
   - workbook facts,
   - application/environment facts.

### 15.3 Current OxFunc Implementation Position
OxFunc now carries a typed local interface in `crates/oxfunc_core/src/host_info.rs`:
1. `CellInfoQuery`
2. `InfoQuery`
3. `HostInfoProvider`

Current intended seam rule:
1. OxFunc should ask for enum-based host facts.
2. Upstream should not pass raw workbook objects or parse-tree internals into OxFunc for these functions.
3. String parsing for `CELL` / `INFO` remains OxFunc responsibility.

### 15.4 Function Split
`CELL` is mixed:
1. local/reference-based lanes:
   - `address`
   - `row`
   - `col`
   - `contents`
   - `type`
2. host-query lanes:
   - `filename`
   - `format`
   - `color`
   - `prefix`
   - `protect`
   - `width`

`INFO` is mostly host-query driven:
1. `directory`
2. `numfile`
3. `origin`
4. `osversion`
5. `recalc`
6. `release`
7. `system`
8. `memavail`
9. `memused`
10. `totmem`

### 15.5 Empirical Pressure
W15 replay on `2026-03-15` showed:
1. `INFO("directory")`, `numfile`, `origin`, `osversion`, `recalc`, `release`, and `system` all return concrete host/workbook facts
2. `memavail`, `memused`, and `totmem` currently return `#N/A` on this host baseline

So OxFunc should not try to synthesize these answers from local value state.

### 15.6 Upstream Request
Leave room for a typed host-query capability view that can answer:
1. selected cell metadata by preserved reference identity
2. selected workbook facts
3. selected application/environment facts

without requiring OxFunc to depend on:
1. arbitrary parse-tree access
2. workbook object model handles
3. host-specific callback names encoded inside function kernels

Update 2026-03-15:
1. OxFunc now has a working typed host-query seam for generated `CELL` / `INFO` XLL exports, backed locally by an XLL-side provider.
2. `CELL(info_type)` omitted-reference forms are now confirmed to depend on the active selected cell, not merely the caller cell. Upstream therefore needs a way to surface active-selection context to the host-query facility even when no explicit reference argument is present.
3. The current provider seam should allow optional preserved reference identity for `CELL` queries.
4. Provider-backed generated `ox_CELL` / `ox_INFO` exports matched native `CELL` / `INFO` on the current baseline without needing the `#` macro-type suffix, even though the manual legacy `GET.*` probe wrappers still require `#`.
5. `CELL("width", ref)` is now locally closed in the generated XLL bridge as a real two-item artifact:
   - ordinary single-cell entry shows the first numeric item,
   - `INDEX(...,2)` exposes the second boolean item,
   - `COLUMNS(...)` reports the two-column width shape.
6. Dual-run (`default` + `compat_template`) replay is now green for the admitted current-baseline `CELL` / `INFO` slice, and OxFml has now acknowledged/integrated `HO-FN-002` on its side.
7. The filed handoff packet for this seam is `docs/handoffs/HANDOFF_W015_CELL_INFO_HOST_QUERY_TO_OXFML.md`.

## 16. Replay Appliance Packet-Adapter Position
### 16.1 Purpose
Record the local OxFunc replay rollout position now that `W018` and `W019` have been opened.

### 16.2 Core Message
The Foundation replay handoff does not reduce the OxFml/OxFunc semantic seam.

For OxFunc:
1. replay is packet-first and row-first,
2. evidence ids, correlation rows, invariant bindings, compatibility descriptors, locale/environment metadata, and XLL limitation classifications remain first-class,
3. normalized replay event families are allowed only as derived projection views over packet witnesses, not as invented semantic traces.

### 16.3 Interface Implications
Current downstream replay rollout does not ask OxFml for arbitrary replay-event emission.

It does ask that upstream-preserved distinctions remain recoverable in packet evidence when they are semantically relevant:
1. source manifest identity,
2. packet/row identity,
3. compatibility/version descriptors,
4. invariant and limitation bindings,
5. semantic-vs-seam-failure classification.

### 16.4 Current Local Position
OxFunc now treats the Replay appliance as a Logistics-layer host over local semantic artifacts:
1. `DNA ReCalc` is not a new semantic authority,
2. OxFunc retains authority over function semantics and evidence meaning,
3. local-only replay source-schema ids are currently prefixed `oxfunc.local.*`,
4. no fake internal evaluator event stream will be emitted for cross-lane symmetry,
5. `cap.C3.explain_valid` is the current honest ceiling,
6. `cap.C4` and `cap.C5` remain explicitly non-claimed.

### 16.5 Open Upstream Question
If OxFml later emits replay-oriented artifacts for evaluator or seam work, they should preserve semantic boundary meaning without assuming that OxFunc packets can or should be decomposed into the same native event granularity.

## 17. Next OxFml Integration Run Agenda
### 17.1 Purpose
Prepare the next OxFml integration round around the central seam-model note:
1. `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md`

This section is not a final design lock.
It is a concrete agenda for iterative stabilization with OxFml's current "semantic requirements first, transport later" posture.

### 17.2 Primary Questions For The Next Round
The highest-value topics now appear to be:
1. function catalog and library-context ownership,
2. callable/helper semantics (`LET`, `LAMBDA`, callable values),
3. explicit scalarization (`@`, `SINGLE`, `_xlfn.SINGLE`),
4. minimum prepared-argument/result provenance vocabulary,
5. operator/literal/value-universe ownership tension,
6. typed host-capability families,
7. availability/feature/profile/provider gating,
8. publication and proving-host/replay surfaces.

### 17.3 Function Catalog And Library Context
Current OxFunc direction:
1. OxFunc should own the canonical function/operator catalog, including canonical ids, aliases, localized names, function profiles, capability declarations, and feature/version/profile gates.
2. OxFml should bind against an externally supplied library-context snapshot rather than hardcoding a closed world of function names.
3. The same library-context snapshot should be consumable by both OxFml (for parse/bind and early rejection) and OxFunc (for evaluation).

Integration-run questions:
1. what is the smallest honest shared library-context snapshot shape,
2. how should OxFml consume catalog/profile facts during parse and early formula rejection,
3. which catalog facts must be visible before binding versus only at evaluation time,
4. how should localized function-name tables participate in that snapshot.

### 17.4 Dynamic Function Registration And State-Free OxFunc
Current OxFunc direction:
1. function registrations are expected to be dynamic rather than permanently static,
2. this includes add-in, VBA, and later user-defined function sources,
3. OxFunc should still remain runtime-state-free,
4. therefore the mutable library context should be externally allocated and versioned, not globally owned by OxFunc.

Working implication:
1. the seam should permit OxFml to pass a library/context handle or snapshot into OxFunc,
2. OxFunc may consume and possibly help refine/normalize that context,
3. but OxFunc should not become the owner of hidden global runtime registry state.

Integration-run questions:
1. what should the lifecycle of the external library context be,
2. how should updates/additions/removals be represented,
3. what stable identity/versioning should a context snapshot carry,
4. where should localized aliases and registration metadata live.

### 17.5 Value Universe, Operators, Literals, And Grammar Tension
Current OxFunc direction:
1. OxFunc owns the semantic value universe and canonical operator set,
2. OxFml owns lexical grammar, parse structure, localized separators, and literal tokenization,
3. precedence/associativity likely remain grammar-owned while operator ids and semantics remain catalog-owned.

This is intentionally not perfectly clean.
The tension should be documented explicitly rather than over-smoothed.

Examples:
1. decimal/group/currency literal spelling is lexical and locale-sensitive,
2. operator meaning and result class are semantic,
3. some operator availability may still depend on catalog/profile decisions.

Integration-run questions:
1. which operator facts should be catalog-visible to OxFml,
2. how should literal-surface localization and semantic value typing stay aligned,
3. should operator syntactic definitions themselves be catalog-backed or remain purely grammar-level.

### 17.6 Callable Helper Values: `LET` And `LAMBDA`
These should now be near the top of the seam agenda.

Current OxFunc direction:
1. OxFml should own helper syntax, sequential binding, shadowing, lambda formation, lexical capture, and invocation planning.
2. OxFunc should not need raw helper AST ownership.
3. OxFunc should be able to consume callable values through a typed carrier or typed invocation facility without losing lexical meaning.

Important semantic position:
1. callable/lambda values should be treated as first-class semantic values in the value universe,
2. they are still not ordinary worksheet cell-display values on the current Excel baseline,
3. but they should be admissible as:
   - helper-produced values,
   - function arguments,
   - function return values,
   - defined-name values,
   - UDF interop payloads.

Working extension for the seam:
1. the seam should leave room for native and user-defined lambda values crossing through an interop carrier such as `IExcelLambda` or equivalent,
2. this should support argument passing, return values, and higher-order/currying patterns,
3. publication restrictions should remain separate from the question of whether the lambda is a first-class semantic value.

Integration-run questions:
1. what is the smallest callable-value carrier OxFml can expose without erasing lexical capture,
2. which callable facts OxFunc truly needs beyond opaque identity plus typed invocation,
3. how callable values should appear in defined names and UDF interop,
4. which callable restrictions are publication-only versus true semantic restrictions.

Focused follow-up after the first `W38` helper/lambda packet:
1. `docs/function-lane/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_RESPONSE_V1.md`

### 17.7 Explicit Scalarization: `@` / `SINGLE`
Current OxFunc direction:
1. OxFml owns syntax, attachment point, compatibility alias recognition, and caller-cell association.
2. OxFunc should own the scalarization semantics and function-profile-aware admission/result behavior.
3. The seam should preserve at least `scalar` / `reference` / `array` / `error` result-class distinctions plus caller context.

Integration-run questions:
1. where should explicit `@` live in the execution pipeline,
2. what is the minimum provenance vocabulary needed for `@`,
3. how should `_xlfn.SINGLE(...)` compatibility and round-trip treatment be represented,
4. how should OxFml and OxFunc divide work for non-call operands versus function-call operands.

### 17.8 Typed Host Capability Families
Current OxFunc direction:
1. host-sensitive functions should consume typed capability views rather than raw workbook objects or arbitrary callbacks,
2. caller cell, active selection, referenced cell metadata, workbook facts, application facts, environment facts, and later row-visibility/provider families should all be representable this way.

Integration-run questions:
1. which host-capability families should be locked first after `CELL` / `INFO`,
2. how active selection should be represented when no explicit reference argument is present,
3. where provider-bound services such as translation should sit relative to generic host-query capabilities.

### 17.9 Availability, Feature Gates, And Provider Presence
Processed functions now show that OxFml and OxFunc need a shared way to distinguish:
1. known in catalog,
2. feature-gated,
3. compatibility-gated,
4. host-profile unavailable,
5. add-in absent,
6. provider unavailable.

Integration-run questions:
1. which of these states belong in the library context,
2. which belong in runtime capability views,
3. how early formula rejection should differ from runtime `#NAME?`, capability denial, or provider-failure outcomes.

### 17.10 Seam Themes Revealed By Processed Functions
The functions processed so far strongly suggest that the following seam themes are real and worth naming explicitly in OxFml:
1. reference identity vs eager dereference,
2. direct-cell-binding proving-host truth,
3. whole-axis geometry and compatibility-sensitive sheet assumptions,
4. locale/profile-sensitive semantics, not only formatting,
5. typed host-query capability families,
6. provider-bound functions,
7. feature availability and `#NAME?` classification,
8. volatility, random/time providers, and recalculation profile,
9. semantic failure vs capability denial vs seam limitation,
10. format hints and publication behavior above the pure kernel,
11. family-specific admission/coercion policy as catalog metadata.

### 17.11 Requested OxFml Response Shape
The next useful OxFml-side response would ideally say, topic by topic:
1. which semantic distinctions OxFml agrees must survive,
2. which distinctions OxFml believes belong in library context versus prepared arguments/results versus host capability views,
3. where OxFml currently wants to keep transport deliberately open,
4. which of the above topics should be stabilized first.

### 17.12 Narrowed Recommendation For This Stage
If the next integration round needs a smaller working set, the current best narrowed focus is:
1. external library-context snapshot,
2. callable-value minimum carrier,
3. availability / feature-gate / provider-failure taxonomy.

Reference:
1. `docs/function-lane/OXFML_OXFUNC_NEXT_ROUND_STABILIZATION_TOPICS.md`

### 17.13 OxFunc Narrowed Response After OxFml Processing
After reading the latest OxFml note, OxFunc's best next-step response is now captured in:
1. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V1.md`

Current intent:
1. make Topic A concrete enough to support parse/bind/evaluation identity,
2. make Topic B concrete enough to keep availability truth distinct from runtime/provider truth,
3. state only the minimum callable facts OxFunc currently needs for Topic C,
4. avoid pretending that provenance, `@`, or full callable transport are already ready for final lock.

### 17.14 OxFunc Further Narrowing After The Latest OxFml Round
After the most recent OxFml processing pass, OxFunc now narrows one step further in:
1. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V2.md`

Current focus:
1. make the operator-admission vs grammar boundary more explicit,
2. make the minimum library-context field set more concrete,
3. map availability states to parse/bind, runtime-capability, and post-dispatch stages,
4. keep the callable-value carrier intentionally looser for one more round while the surrounding catalog/admission surfaces settle.

## 18. Round Closure Position
### 18.1 Purpose
Capture what OxFunc now thinks is the right closing position for this seam round.

This is not a claim that the seam is finalized.
It is a claim that the current round has probably reached the point of diminishing returns for further note-only narrowing.

### 18.2 What Now Looks Aligned
OxFunc currently reads the following as substantially aligned with OxFml:
1. semantic requirements should be preserved first, while transport remains provisional,
2. the next stabilization topics are:
   - external library-context snapshot,
   - availability / feature-gate / provider-failure taxonomy,
   - callable-value minimum carrier,
3. OxFml and OxFunc both now place library-context truth above runtime capability/provider truth,
4. callable values should be treated as first-class semantic values even if publication restrictions remain separate,
5. the operator/literal/value-universe ownership tension should remain explicit rather than hidden,
6. callable transport should remain intentionally looser until catalog/admission surfaces are more stable,
7. `#` normally resolves upstream and does not require a default spill-provenance flag once fully resolved.

### 18.3 What Remains Intentionally Open
OxFunc expects the following to remain open after this round:
1. the smallest honest shared library-context snapshot shape,
2. the exact catalog-backed boundary for operator admission versus pure grammar ownership,
3. the final split between early formula rejection, runtime `#NAME?`, typed capability denial, and provider-failure outcomes,
4. the final shared callable-value carrier,
5. the full provenance vocabulary for prepared arguments/results,
6. the final placement of explicit `@` semantics in the execution pipeline,
7. the final compatibility and round-trip treatment of `_xlfn.SINGLE(...)`,
8. broader host-query return-family carriers.

These are now expected unresolved items, not signs that the round failed.

### 18.4 Best Next Move After This Round
OxFunc's current recommendation is:
1. do not keep narrowing the notes indefinitely,
2. let OxFml canonical docs absorb the current aligned direction,
3. revisit the seam next when one of the still-open topics needs:
   - a more concrete canonical field set,
   - a proving-host artifact,
   - an implementation-facing handoff,
   - or a coordinator-visible consequence.

### 18.5 Working Rule
Until that next trigger point:
1. treat the current OxFml canonical seam docs as the active upstream baseline,
2. treat the OxFunc seam-model and stabilization notes as the downstream semantic rationale,
3. do not over-read the current alignment as final interface lock,
4. do treat it as enough alignment to proceed with function work and future seam-triggered refinements.

## 19. `LET` / `LAMBDA` Pin-Down Follow-Up
### 19.1 Why This Follow-Up Exists
`W38` now gives OxFunc a real empirical helper/lambda baseline instead of only seam theory:
1. direct `LET`,
2. immediate `LAMBDA`,
3. current `ISOMITTED` lanes,
4. first higher-order helper lanes through `MAP`, `REDUCE`, and `SCAN`.

That makes it possible to answer OxFml's focused prep note more concretely.

### 19.2 Current OxFunc Reply
The focused reply is now:
1. `docs/function-lane/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_RESPONSE_V1.md`

Current OxFunc position:
1. lexical meaning, exact capture truth, and callable-first-class status are now accepted as fixed enough for the next narrowing round,
2. the earlier OxFunc narrowing candidate for the smallest honest callable carrier looked like:
   - opaque `callable_token`,
   - `origin_kind`,
   - `arity_shape`,
   - `capture_mode`,
   - `invocation_contract_ref`,
3. exact parameter-name and capture-name surfaces should remain provenance/replay detail for now rather than minimum carrier fields,
4. typed invocation over an opaque callable token is the preferred invocation boundary.

Reading rule after round closure:
1. this section records the narrowed candidate that shaped the exchange,
2. it should not be read as a post-closure field lock,
3. the active post-closure position is in Sections `19.8` and `19.9`.

### 19.3 Why This Is The Current Boundary
This is the smallest boundary that still fits the local evidence:
1. immediate `LAMBDA` invocation,
2. lexical capture from `LET`,
3. admission-time helper validation failures,
4. higher-order helper invocation inside `MAP`, `REDUCE`, and `SCAN`.

### 19.4 What Remains Open
This still intentionally leaves open:
1. final callable ABI/transport,
2. fuller UDF/interoperable callable transport beyond the Excel-supported Defined Name callable surface,
3. `BYROW`, `BYCOL`, and `MAKEARRAY`,
4. broader callable/provider interaction beyond the generic staged availability model.

### 19.5 Processed Latest OxFml Response
OxFunc has now processed the latest OxFml note and reads it as substantially convergent on the narrowed `LET` / `LAMBDA` round.

Items OxFunc now incorporates from OxFml's response:
1. opaque callable identity is only acceptable if the minimum typed semantic fields remain recoverable:
   - `origin_kind`
   - `capture_mode`
   - `arity_shape`
   - stable invocation-contract meaning
2. richer callable detail may remain outside the minimum carrier, but it must survive as structured provenance/replay detail rather than collapsing to free-form summary text,
3. `invocation_contract_ref` should be understood as a stable semantic invocation reference, not as an implementation-specific callback or ABI handle,
4. the generic staged availability model should remain the default for callable lanes unless later callable-specific evidence proves it insufficient,
5. OxFunc-local candidate field names such as `callable_token`, `arity_shape`, and `invocation_contract_ref` should be read as candidate labels, not as demanded canonical OxFml vocabulary.

Those points are consistent enough with the current OxFunc direction that they should now be treated as aligned for this round.

### 19.6 Narrow OxFunc Alternatives Still Preferred
OxFunc still prefers the following narrower boundary choices unless later evidence forces otherwise:
1. parameter-name, capture-name, and body-kind detail should stay out of the minimum shared callable carrier by default,
2. typed invocation over an opaque callable token should remain the preferred invocation boundary,
3. callable-specific availability/provider typing should not be expanded beyond the generic staged availability model until a concrete callable case proves that the generic taxonomy is insufficient,
4. higher-order helper evidence from `W38` should continue to inform OxFunc's local seam pressure, but should not by itself be treated as upstream seam-lock evidence until OxFml has comparable exercised local coverage.

In other words:
1. yes to a typed minimum carrier,
2. yes to structured provenance,
3. no to prematurely promoting rich callable inspection detail into the shared hot-path carrier,
4. no to inventing a special callable-only availability taxonomy before evidence requires it,
5. no to forcing canonical shared field names before the semantic field set itself is stable enough.

### 19.7 Best Next OxFml/OxFunc Narrowing Step
The best next narrowing step had looked like:
1. lock the minimum callable carrier field set,
2. lock the meaning of `invocation_contract_ref` as semantic rather than implementation-facing,
3. lock the carrier-versus-provenance split for parameter/capture/body detail,
4. leave broader callable/provider interaction and full publication policy open,
5. explicitly keep open whether an additional invocation-model field is needed beyond the current `invocation_contract_ref` idea until stronger evidence appears.

After processing the latest OxFml note, OxFunc no longer recommends continuing that narrowing in this exchange.

### 19.8 Final Processed OxFml Closure
OxFunc reads OxFml's latest note as a real round-closure message and incorporates the following as aligned for this exchange:
1. semantic requirements remain primary and transport remains provisional,
2. library-context truth remains above runtime capability/provider truth,
3. callable values remain first-class semantic values,
4. the generic staged availability model remains the default for callable lanes,
5. richer callable detail remains out of the minimum hot-path carrier by default if structured provenance/replay detail preserves it,
6. typed invocation over a narrower callable carrier remains the preferred direction,
7. canonical field names such as `callable_token`, `arity_shape`, and `invocation_contract_ref` are still candidate labels rather than locked shared vocabulary,
8. OxFunc-local higher-order helper evidence from `W38` is useful local seam pressure but not by itself enough to lock the upstream shared seam.

OxFunc also accepts OxFml's explicit non-locks for this round:
1. final minimum callable carrier field set,
2. final carrier-versus-provenance split,
3. final canonical shared field names,
4. final callable/provider interaction beyond the generic staged availability model,
5. higher-order callable seam pressure inferred only from OxFunc-local evidence.

### 19.9 Deferred Until Further Evidence
The remaining callable-seam questions should now be treated as explicitly deferred future work, not as unresolved note-exchange debt.

Deferred OxFunc owner:
1. `docs/worksets/W042_DEFERRED_CALLABLE_SEAM_FIELD_LOCK_AND_HIGHER_ORDER_EVIDENCE.md`

Deferred topics:
1. minimum callable carrier field lock beyond the current narrowed candidate set,
2. final carrier-versus-provenance split for parameter, capture, and body detail,
3. whether an additional invocation-model field is needed beyond the current `invocation_contract_ref` idea,
4. higher-order helper seam pressure from `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`, and `ISOMITTED`,
5. fuller UDF, add-in, and interoperable callable transport beyond the Excel-supported Defined Name callable surface,
6. any callable/provider-stage refinement beyond the generic staged availability model.

Working closure reading:
1. there is enough convergence to proceed with function work and later seam-triggered refinement,
2. the unresolved callable details are now narrow and mostly about field-set shape and evidence maturity rather than semantic disagreement,
3. the remaining weakly evidenced aspects should stay open and owned by `W042` rather than being forced to closure prematurely,
4. OxFunc should treat this as its final note in this callable/library-context exchange unless a stronger trigger appears:
   - a concrete field-set lock need,
   - a proving-host/runtime artifact forcing a narrower choice,
   - or new higher-order helper evidence that materially changes the seam.

## 20. First Seam-Improvement Round From Function Work
### 20.1 Purpose
The earlier note rounds were mostly seam-shaping and doctrine alignment.

OxFunc now has enough concrete function work to report a first real seam-improvement round driven by implementation and evidence rather than only note narrowing.

This section is intentionally narrower than a new broad seam round.
It focuses on:
1. `LET` / `LAMBDA` and higher-order helper pressure from `W38`,
2. `RTD` as a singular host/subscription seam from `W43`,
3. the ownership split these now make clearer between OxFml preparation/binding and OxFunc runtime semantics.

### 20.2 `W38` Callable/Helper Progress That Matters To The Seam
Relevant local records:
1. `docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md`
2. `docs/function-lane/W38_EXECUTION_RECORD.md`

What OxFunc now has locally:
1. seeded native worksheet evidence for:
   - `LET`
   - immediate `LAMBDA`
   - current `ISOMITTED` lanes
   - `MAP`
   - `REDUCE`
   - `SCAN`
   - `BYROW`
   - `BYCOL`
   - `MAKEARRAY`
   - workbook Defined Name callable preservation
2. a typed callable value carrier in core rather than a bare lambda placeholder,
3. executable callable invocation and worksheet-surface evaluators for the admitted higher-order slice when a prepared callable value is supplied,
4. a direct Stage 1 prepared-expression runtime substrate for:
   - `LET`
   - immediate `LAMBDA`
   - current direct `ISOMITTED`
5. Lean executable substrate alignment for the admitted higher-order helper slice and callable publication/preservation seeds.

### 20.3 `W38` Seam Reading
The best current OxFunc reading is now:
1. OxFml remains the owner of worksheet helper formation/binding for direct `LET` / `LAMBDA` expressions.
2. OxFunc can now honestly own more of the callable runtime side once a prepared callable/value artifact is supplied.
3. The cross-repo seam no longer needs to be discussed only as a hypothetical minimum carrier; OxFunc now has real local pressure from:
   - lexical capture,
   - immediate invocation,
   - higher-order helper invocation,
   - Defined Name callable preservation.
4. Defined Name callable preservation is now first-pass Excel scope, not a deferred extension.
5. UDF/add-in/interoperable callable origins and returns still remain deferred.

This means the current seam split looks more concrete:
1. OxFml:
   - helper syntax
   - helper binding/admission
   - direct worksheet formation of callable/helper artifacts
2. OxFunc:
   - typed callable runtime semantics after preparation
   - invocation
   - helper-family execution over prepared callable values
   - callable publication/result behavior on the admitted slice

### 20.4 `RTD` As A Distinct Seam Class
Relevant local records:
1. `docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md`
2. `docs/function-lane/RTD_REFERENCE_CAPTURE_AND_SEAM_NOTES.md`
3. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`
4. `docs/function-lane/W43_EXECUTION_RECORD.md`

OxFunc now treats `RTD` as a distinct seam class rather than just another external-data function.

Current local reading:
1. OxFunc should own the admitted `RTD` call shape and the result projection boundary.
2. OxFunc should not own:
   - COM activation
   - topic subscription tables
   - topic lifetime tracking
   - callback threading / `UpdateNotify`
   - host recalc scheduling
   - workbook/cell subscription maps
3. Those responsibilities sit above OxFunc, between OxFml and the higher-level host application.

### 20.5 Current Best-Attempt `RTD` Interface Direction
OxFunc now has a local first-pass runtime/interface surface:
1. `RtdRequest`
   - `prog_id`
   - `server_name`
   - `topic_strings`
2. `RtdProvider`
   - host-supplied resolution hook
3. `RtdProviderResult`
   - `Value`
   - `NoValueYet`
   - `CapabilityDenied`
   - `ConnectionFailed`
   - `ProviderError`

Current local worksheet projection:
1. `Value(v)` -> `v`
2. `NoValueYet` -> `#N/A`
3. `CapabilityDenied` -> `#BLOCKED!`
4. `ConnectionFailed` -> `#CONNECT!`
5. `ProviderError(code)` -> `code`

This is a best-attempt OxFunc-local seam, not a locked shared contract.

### 20.6 What `RTD` Suggests More Broadly
`RTD` reinforces a broader seam pattern that is likely to matter for other host/provider functions:
1. OxFml can prepare and classify a function call whose true machinery lives above OxFunc.
2. OxFunc can then evaluate against a typed host-supplied outcome surface without taking ownership of lifecycle/runtime orchestration.
3. This is similar in shape to host-query and capability-view work:
   - prepared request
   - typed host/provider capability surface
   - typed value/error projection back into worksheet semantics

That pattern may become useful later for:
1. `SUBTOTAL` / `AGGREGATE`
2. `ISFORMULA`
3. provider-language or provider-data functions
4. other capability-sensitive seams

### 20.7 Additional Relevant Local Lesson
The recent function work continues to reinforce one useful rule for the seam:
1. XLL bridge/harness limitations should be documented,
2. but they should not be treated as open OxFunc semantic lanes when native worksheet evidence and core/runtime evidence already pin the slice honestly.

That matters for both:
1. callable/helper surfaces in `W38`,
2. and any future host/provider seam like `RTD`.

### 20.8 What OxFunc Thinks OxFml Should Take From This Round
The most useful takeaways on the OxFml side are:
1. the callable seam is now less hypothetical because OxFunc has a real local callable runtime substrate and real higher-order helper evidence,
2. direct helper formation/binding remains an OxFml-owned concern,
3. Defined Name callable preservation should now be treated as first-pass Excel-scope seam pressure,
4. `RTD` should probably be modeled as a prepared host-subscription request plus typed host result surface, not as an ordinary provider fetch kernel,
5. the OxFml <-> OxFunc boundary can stay minimal if it preserves the right typed prepared artifacts and typed host result classes.

### 20.9 Intentionally Still Open
This first seam-improvement round does not lock:
1. the final shared callable carrier field set,
2. the final direct `LET` / `LAMBDA` formation artifact shape,
3. the final shared naming of callable carrier fields,
4. the exact current-baseline Excel matrix for all `RTD` startup/disconnect/save-value edge cases,
5. any broader generalized provider/subscription contract beyond the current `RTD` first pass.

These remain open until stronger evidence or a more concrete implementation handoff is needed.

## 21. Processed Latest OxFml Update And Current Stabilization Move
### 21.1 Current OxFunc Reading
OxFunc reads the latest OxFml note as largely convergent and more stabilization-oriented than the earlier callable-only rounds.

The most important incorporated points are:
1. the current three-topic stabilization order remains:
   - external library-context snapshot
   - availability / feature-gate / provider-failure taxonomy
   - callable-value minimum carrier
2. callable transport remains intentionally open,
3. the callable round remains functionally converged enough that it should not be reopened without a concrete trigger,
4. higher-order helper evidence from `W38` is still useful local pressure but not yet treated by OxFml as upstream seam-lock evidence,
5. OxFml now asks for something more concrete on Topic A:
   - either a pinned export for the library-context snapshot,
   - or a stable downstream pointer plus export-reading guidance.

OxFunc agrees with that reading.

### 21.2 What OxFunc Incorporates
OxFunc now incorporates the following as the settled reading for this stage:
1. the callable/library-context exchange is no longer the main note-level blocker,
2. the next honest stabilization step is not more abstract callable naming debate,
3. the next honest stabilization step is producing a real downstream library-context snapshot export surface,
4. the generic staged availability model remains the right default for both callable and host/provider seams,
5. `RTD` fits that staged pattern as a prepared request plus typed host/provider outcome surface rather than as a reason to reopen the callable round.

### 21.3 What OxFunc Still Proposes As Alternatives
OxFunc still prefers:
1. keeping candidate carrier names such as `callable_token`, `arity_shape`, and `invocation_contract_ref` as candidate labels only,
2. keeping richer callable detail outside the minimum hot-path carrier by default,
3. treating direct `LET` / `LAMBDA` formation artifacts as OxFml-owned until a narrower implementation-facing seam is required,
4. not forcing a generalized provider/subscription contract from `RTD` before more than one function seam actually needs it.

These are not presented as active disagreements.
They are the current narrower OxFunc preferences while transport remains open.

### 21.4 Current Stabilization Move
OxFunc now has a concrete downstream owner for Topic A:
1. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`

Current OxFunc intent for `W044`:
1. produce the first honest library-context snapshot export artifact or stable export pointer,
2. pin snapshot identity/versioning,
3. project the minimum exercised field set that OxFml is asking for,
4. provide export-reading guidance rather than leaving the downstream catalog/profile surfaces implicit.

### 21.5 Current Best Honest `W044` Snapshot Export
`W044` now has a first explicit downstream export:
1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
3. generator:
   - `tools/w44-probe/generate-w44-library-context-snapshot.ps1`

Current first-pass export facts:
1. `534` rows:
   - `511` built-in function rows
   - `23` operator rows:
     - the full current `W45` non-`@` operator surface
     - one explicitly modeled `FUNC.OP_IMPLICIT_INTERSECTION` row
2. `snapshot_id = oxfunc-libctx-v1`,
3. `snapshot_generation = 2026-03-21`,
4. `source_commit_short = f5ec5ac`,
5. `source_commit_full = f5ec5ac6c64c05ec1f3d59fadc092105b7ae5d01`,
6. `source_tree_state = dirty`,
7. stable function/operator ids emitted as current OxFunc-local `FUNC.<CANONICAL_NAME>` ids,
8. explicit pointer to the multilingual name table seed,
9. explicit OxFunc-local semantic/gating reference fields,
10. detailed `FunctionMeta`-derived profile fields where they are currently extractable,
11. explicit registration-source classification through:
   - `registration_source_kind`
   - `built_in_catalog_function`
   - `built_in_operator_export`
   - `doc_modeled_operator`
12. seam-heavy classification fields:
   - `metadata_status`
   - `special_interface_kind`
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `arity_shape_note`
   - `interface_contract_ref`
13. built-in C API interop fields for matched built-in rows:
   - `xlcall_builtin_symbol`
   - `xlcall_builtin_code`
14. explicit OxFml-facing reading guidance.
15. the completed `W039` dynamic-array reshaping family is now reflected as real extracted profile rows rather than `catalog_only` rows for examples such as:
   - `FUNC.CHOOSECOLS`
   - `FUNC.FILTER`
   - `FUNC.UNIQUE`
   - `FUNC.VSTACK`
16. ordinary exported operators now also carry `interface_contract_ref` back to `W045`, while `FUNC.OP_IMPLICIT_INTERSECTION` still points to its own special investigation surface.

Current special rows worth consuming directly:
1. `FUNC.LET`
2. `FUNC.LAMBDA`
3. `FUNC.RTD`
4. `FUNC.OP_IMPLICIT_INTERSECTION`

Those rows now carry:
1. `registration_source_kind`
2. `special_interface_kind`
3. `admission_interface_kind`
4. `preparation_owner`
5. `runtime_boundary_kind`
6. `interface_contract_ref`
7. `source_commit_short`
8. `source_commit_full`
9. `source_tree_state`

Current additional seam-relevant presentation rows worth consuming directly:
1. `FUNC.NOW`
2. `FUNC.TODAY`
3. `FUNC.HYPERLINK`

Those rows now carry:
1. extracted arity/profile columns from the actual runtime metadata,
2. `special_interface_kind = presentation_hinting_function`,
3. `runtime_boundary_kind = extended_value_with_presentation_hint`,
4. specific `interface_contract_ref` values back to the current function-slice contract or value-model note,
5. first-pass reading:
   - ordinary value production still exists,
   - but the publication-aware OxFunc path now returns value plus presentation hint rather than plain value only.

Current additional locale/profile/provider seam rows worth consuming directly:
1. `FUNC.ASC`
2. `FUNC.DBCS`
3. `FUNC.JIS`
4. `FUNC.NUMBERVALUE`
5. `FUNC.TRANSLATE`

Those rows now carry:
1. curated arity/profile columns,
2. `special_interface_kind = width_conversion_host_profile` for `ASC` / `DBCS` / `JIS`,
3. `special_interface_kind = locale_default_profiled_parse` for `NUMBERVALUE`,
4. `special_interface_kind = provider_language_request` for `TRANSLATE`,
5. `runtime_boundary_kind = typed_host_width_conversion_mode` for the width-conversion family,
6. `runtime_boundary_kind = ordinary_eval_with_locale_defaults` for `NUMBERVALUE`,
7. `runtime_boundary_kind = host_provider_projection` for `TRANSLATE`,
8. direct `interface_contract_ref` values to:
   - `docs/function-lane/FUNCTION_SLICE_WIDTH_CONVERSION_HOST_PROFILE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_NUMBERVALUE_LOCALE_DEFAULT_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_TRANSLATE_PROVIDER_LANGUAGE_CONTRACT_PRELIM.md`

Current reading:
1. the locale/profile/provider residuals from `W034` / `W035` / `W036` are now closed on the OxFunc side,
2. OxFml now mainly needs to consume the typed context/query seams rather than wait on more local OxFunc kernel work,
3. the refreshed `W044` export should now be read as the current best downstream source for those rows as well, not just for `LET` / `LAMBDA` / `RTD` / presentation-hint rows.

Current additional representative ordinary rows worth consuming directly:
1. `FUNC.CHOOSECOLS`
2. `FUNC.FILTER`
3. `FUNC.UNIQUE`
4. `FUNC.VSTACK`

Those rows now carry:
1. extracted arity/profile columns from the actual completed runtime family,
2. `metadata_status = function_meta_extracted`,
3. `interface_contract_ref = docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`

This is now the best honest downstream stabilization artifact for Topic A.

It still does not yet lock:
1. direct normalized detailed profile fields for every seam-heavy row such as `LET` and `LAMBDA`,
2. final cross-repo field names,
3. fully dereferenceable per-entry semantic/gating profile bundles,
4. the full future operator universe beyond the current exported set plus the explicitly modeled implicit-intersection row.

### 21.5A Explicit OxFml Feedback Request
OxFunc now wants to treat this richer `W044` export as attempt one, not as a field-set lock.

Current downstream request to OxFml:
1. tell us which of these exported fields are already useful as-is,
2. tell us which fields should be renamed, split, or dropped,
3. tell us whether `interface_contract_ref` is a useful first-pass bridge for seam-heavy rows,
4. tell us whether the new first-pass seam-facing fields:
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `arity_shape_note`
   are useful, or whether OxFml wants a different split,
5. tell us whether `registration_source_kind` is useful as a direct field or whether OxFml would prefer it folded into another profile surface,
6. tell us whether the new provenance triple
   - `source_commit_short`
   - `source_commit_full`
   - `source_tree_state`
   is sufficient for test pinning or whether OxFml still wants an additional tag/ref field,
7. tell us whether OxFml wants additional direct columns for `LET` / `LAMBDA` admission shape before we normalize them locally.

### 21.5B Recommended OxFml Consumption Path
Current OxFunc recommendation:
1. consume `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` directly,
2. pin OxFml-side tests and semantic-plan fixtures to:
   - `snapshot_id = oxfunc-libctx-v1`
   - `snapshot_generation = 2026-03-21`
   - `source_commit_short = f5ec5ac`
   - `source_commit_full = f5ec5ac6c64c05ec1f3d59fadc092105b7ae5d01`
   - `source_tree_state = dirty`
3. treat `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md` as the interpretation guide for the first pass,
4. use concrete snapshot-consumption mismatches as the next bounded trigger rather than reopening broad seam theory.

Additional OxFunc note:
1. the current export is still a valid bounded integration artifact when `source_tree_state = dirty`,
2. but OxFunc does not want that confused with a clean release snapshot,
3. so any downstream mismatch report should preserve all three provenance fields.

### 21.5C Processed Latest OxFml Note
OxFunc reads the latest OxFml note as convergent and narrow in a useful way:
1. keep using the current `W044` export as the concrete integration artifact,
2. preserve `source_tree_state` in downstream mismatch reports,
3. add a stronger commit-level pin beside `source_commit_short` when convenient,
4. keep pushing toward more dereferenceable semantic/gating surfaces without blocking first-pass snapshot consumption now,
5. for `NOW`, `TODAY`, and `HYPERLINK`, start reading `runtime_boundary_kind = extended_value_with_presentation_hint` as the current OxFunc-side answer for value-plus-format/style publication behavior.

### 21.5D Current OxFunc Reading
OxFunc reads the latest OxFml note as convergent on the important points for this round:
1. the next useful progress is concrete snapshot consumption,
2. the callable round should remain closed unless a narrower trigger appears,
3. `RTD` should stay a prepared request plus typed host/provider outcome seam,
4. remaining differences are about field-set quality and evidence maturity, not active semantic disagreement.
5. a useful next bounded consumption check is whether the refreshed `W039` ordinary dynamic-array rows already give OxFml enough planning information without a special-case side channel,
6. the new `source_commit_full` field directly incorporates one of OxFml's few concrete export-shape asks without forcing a broader field-lock round.

### 21.5E Current RTD Sync Bundle
For the next RTD sync, OxFunc now wants OxFml to read the following as one bounded bundle:
1. `docs/function-lane/RTD_REFERENCE_CAPTURE_AND_SEAM_NOTES.md`
2. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`
3. `docs/function-lane/W43_EXECUTION_RECORD.md`
4. `docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md`
5. the raw captures under `docs/function-lane/reference-captures/rtd/`

Current OxFunc reading:
1. lifecycle/state ownership for `RTD` is above OxFunc and should stay there,
2. OxFunc should only see a prepared `RtdRequest` plus a typed host callback/result surface,
3. for uniformity, the host callback returns either the current RTD value or a classified provider/runtime outcome,
4. OxFunc then projects that supplied result into the worksheet value/error universe,
5. OxFunc does not need any stronger RTD-local state model unless a later concrete integration need proves otherwise.

### 21.6 Still Intentionally Open
The following remain intentionally open after processing the latest OxFml note:
1. final shared library-context snapshot ABI,
2. final callable carrier field set and canonical names,
3. final callable carrier versus provenance split,
4. exact operator-admission lock,
5. any broader generalized provider/subscription contract inferred from `RTD`.

### 21.7 Working Rule
After the current refreshed `W044` export surface:
1. treat the current OxFml note as convergent and stabilization-oriented,
2. treat the callable round as closed for now,
3. treat the refreshed snapshot export as the current best downstream stabilization artifact for Topic A,
4. do not reopen callable-note narrowing unless a concrete field-set or runtime trigger appears.

### 21.8 Built-In C API Identity And Registration Seam
OxFunc now wants to include one additional concrete seam-improvement item in the next exchange: built-in `XLCALL.H` identity ingest plus the first-pass `CALL` / `REGISTER.ID` registration seam.

Current new artifacts:
1. `docs/function-lane/XLCALL_CODE_CATALOG.csv`
2. `docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md`
3. `docs/function-lane/W46_EXECUTION_RECORD.md`

Current OxFunc reading:
1. OxFunc should act as steward of the function registration catalog.
2. The host side should continue to own the raw Excel C API callback surface and external invocation runtime.
3. Built-in `xlf*` codes from `XLCALL.H` should be treated as compatibility/interoperability metadata attached to OxFunc built-in rows, not as replacements for `surface_stable_id`.
4. The refreshed `W044` export now exposes:
   - `xlcall_builtin_symbol`
   - `xlcall_builtin_code`
   on matched built-in rows.
5. That should let OxFml preserve stable OxFunc ids as primary identity while still routing host C API built-ins through their legacy `xlf*` identities where needed.

Current special rows worth consuming directly:
1. `FUNC.CALL`
2. `FUNC.REGISTER.ID`
3. `FUNC.RTD`

Current first-pass registration seam reading:
1. host/OxFml resolves raw C API built-in dispatch and host registration/runtime concerns,
2. OxFunc owns built-in catalog mapping and later registered-function catalog descriptors,
3. worksheet `CALL` / `REGISTER.ID` still remain open as runtime work, but the catalog/seam ownership is now explicit enough for the next round.

## 22. What Would Make The Current Covered Function Scope Completely Usable

### 22.1 Working Definition
Current OxFunc reading:
1. "completely usable" does not mean "all of Excel".
2. It means OxFml can drive the full already-covered OxFunc function/operator scope end-to-end without inventing new side channels or missing runtime/context interfaces.
3. It explicitly excludes the still-open packets:
   - `W014` implicit intersection / `@`,
   - `W038` callable/helper completion,
   - `W041` external/cube/service family,
   - `W046` worksheet `CALL` / `REGISTER.ID` runtime,
   - open `W023` residuals such as `IMAGE`,
   - and any later registered-external/UDF invocation runtime.

### 22.2 Current Covered Scope That Should Become Fully Usable
The target covered scope now includes:
1. ordinary completed function packets,
2. `W034` / `W035` / `W036` locale/profile/provider seams,
3. `W040` reference-metadata host-query seams,
4. `W043` `RTD` prepared request plus typed host outcome seam,
5. `W045` non-`@` operator surface,
6. the current publication-aware lanes for `NOW`, `TODAY`, and `HYPERLINK`,
7. current built-in catalog and matched `XLCALL.H` built-in identities.

### 22.3 Minimum OxFml Interface Bundle Needed
OxFunc currently reads the minimum usable OxFml-side bundle as:
1. snapshot consumption from `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`,
2. stable-id dispatch on `surface_stable_id`,
3. prepared-argument routing driven by:
   - `arg_preparation_profile`,
   - `admission_interface_kind`,
   - `runtime_boundary_kind`,
4. a normal reference/value adapter surface for ordinary rows,
5. a small set of typed runtime/context inputs for the completed seam-heavy rows.

### 22.4 Typed Runtime/Context Inputs OxFml Needs For Current Covered Scope
For the currently covered scope, OxFunc now believes OxFml needs to provide only these runtime/context surfaces:
1. `ReferenceResolver`
   - ordinary reference dereference and refs-visible adapter behavior
2. `now_serial`
   - for time/date host-time surfaces already modeled
3. `random_value`
   - for current random-provider surfaces already modeled
4. `LocaleFormatContext`
   - for locale-default parse/format lanes such as `NUMBERVALUE`
5. `HostInfoProvider`
   - with the currently required typed queries:
     - `query_cell_info(...)`
     - `query_info(...)`
     - `query_formula_text(reference)`
     - `query_sheet_index(CurrentSheet | Reference | SheetNameText)`
     - `query_sheet_count(Workbook | Reference)`
     - `query_aggregate_reference_context(reference)`
     - `query_width_conversion_mode(function)`
     - `query_translate(request)`
6. `RtdProvider`
   - for the current prepared `RTD` request projection:
     - `RtdRequest { prog_id, server_name, topic_strings }`
     - `RtdProviderResult::{ Value, NoValueYet, CapabilityDenied, ConnectionFailed, ProviderError }`

### 22.5 Returned Value Surface Needed
For the currently covered scope, OxFml should be able to consume:
1. ordinary `EvalValue`,
2. `ExtendedValue::ValueWithPresentation { value, hint }`
   - currently needed for:
     - `NOW`
     - `TODAY`
     - `HYPERLINK`
3. current worksheet error projections from typed host/provider outcomes.

Current OxFunc reading:
1. rich-value publication is not required to make the seam-freeze packets (`W047` / `W048` / `W049`) usable,
2. but `IMAGE` remains in the current overall completion scope as a sibling rich-value/publication packet rather than a deferred out-of-scope item.

### 22.6 Catalog/Registration Usability Needed
For the covered built-in scope, OxFunc now believes OxFml should preserve:
1. `surface_stable_id` as the primary identity,
2. `registration_source_kind`,
3. `xlcall_builtin_symbol`,
4. `xlcall_builtin_code`,
5. `source_commit_short`,
6. `source_commit_full`,
7. `source_tree_state`.

Current intent:
1. OxFunc remains steward of the function registration catalog,
2. OxFml/host may still need legacy `xlf*` identities for raw Excel C API routing,
3. but stable OxFunc ids remain the primary seam identity.

### 22.7 What Is Still Missing Before We Should Call It Completely Usable
Current OxFunc reading is that the remaining gap is now mostly interface hardening, not more local function semantics.

To call the current covered scope completely usable, we still want:
1. one explicit OxFml consumer pass against the current `W044` export,
2. one bounded shared contract for the typed context bundle in Section 22.4,
3. one bounded shared contract for the returned publication-aware value surface in Section 22.5,
4. one bounded confirmation that built-in `xlf*` metadata is sufficient for OxFml/host catalog routing on the built-in scope,
5. one clean snapshot run used as the first pinned downstream consumer example.

### 22.8 Recommended Next Spec-Tightening Rounds
Current OxFunc recommendation for the next rounds:
1. Round A: lock the consumable snapshot field subset OxFml actually needs for the already-covered scope.
2. Round B: lock the typed context/query bundle for:
   - `CELL` / `INFO`,
   - `ISFORMULA`,
   - `FORMULATEXT`,
   - `SHEET` / `SHEETS`,
   - `SUBTOTAL` / `AGGREGATE`,
   - `ASC` / `DBCS` / `JIS`,
   - `NUMBERVALUE`,
   - `TRANSLATE`,
   - `RTD`.
3. Round C: lock the return-surface split between:
   - ordinary value,
   - value plus presentation hint,
   - typed host/provider outcome projection.
4. Round D: pin one clean OxFml consumer example against the snapshot and report the first concrete mismatches rather than continuing broad seam theory.

### 22.9 Current OxFunc Claim
Current OxFunc claim is:
1. the remaining work to make the already-covered scope completely usable is now mostly OxFml-consumption and seam-lock work,
2. not a large new OxFunc-local function implementation gap,
3. except for any concrete mismatch OxFml finds while consuming the current snapshot and typed context bundle.

## 23. Processed Latest OxFml Note And Proposed First Freezable Application Seam

### 23.1 What OxFunc Incorporated
OxFunc has processed the latest `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` and incorporated the following as the current working direction:
1. the next rounds should stay artifact-driven and should not reopen broad callable theory,
2. the current `W044` export should be treated as a bounded consumption artifact now,
3. the preferred long-term implementation seam should be:
   - runtime `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - explicit snapshot generations when registration/removal changes the library context,
4. workbook Defined Name callable preservation is now strong first-pass seam pressure and should not be treated as a late extension,
5. typed invocation over opaque callable identity is now strong enough to treat invocation viability as settled for the currently exercised helper floor,
6. first-pass `ISOMITTED` semantics are now pinned tightly enough that `ISOMITTED` does not need to remain a seam-lock driver for the first application freeze,
7. `RTD` remains a prepared request plus typed host/provider outcome seam rather than a reason to widen the general provider model prematurely.

### 23.2 Current OxFunc Reading Of Which `W044` Fields OxFml Can Rely On Now
For the current first application work, OxFunc expects OxFml can rely on these `W044` fields now for callable-relevant rows:
1. `surface_stable_id`
2. `entry_kind`
3. `registration_source_kind`
4. `canonical_surface_name`
5. `arg_preparation_profile` when populated
6. `metadata_status`
7. `special_interface_kind`
8. `admission_interface_kind`
9. `preparation_owner`
10. `runtime_boundary_kind`
11. `arity_shape_note`
12. `interface_contract_ref`
13. snapshot provenance:
   - `snapshot_id`
   - `snapshot_generation`
   - `source_commit_short`
   - `source_commit_full`
   - `source_tree_state`

Current reading:
1. `interface_contract_ref` is useful and should be used for seam-heavy rows now,
2. `admission_interface_kind` / `preparation_owner` / `runtime_boundary_kind` are useful first-pass planning fields now,
3. those names are still not proposed as locked shared canonical vocabulary.

### 23.3 Current Callable-Minimum Direction
Current OxFunc proposal for the smallest honest shared callable minimum is still semantic rather than name-driven.

Semantically required minimum fields:
1. opaque callable identity,
2. origin kind,
3. capture mode,
4. arity shape,
5. invocation-contract meaning.

Current OxFunc reading:
1. these are the fields OxFml should expect to remain recoverable as typed semantic facts,
2. parameter names, capture names, and body-detail can remain provenance/replay-only by default,
3. OxFunc still does not currently see strong enough evidence that an additional explicit invocation-model field is required beyond the current invocation-contract-meaning idea,
4. if later runtime artifacts prove that wrong, `W042` is the correct owner for reopening it.

### 23.4 Current Freezable Seam Direction For First Application Work
Current OxFunc recommendation for a first freezable application seam is:
1. use the current `W044` export as the pinned interchange and debugging artifact,
2. but treat the normative long-term implementation direction as:
   - runtime `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - generation-producing registration/removal
3. use `surface_stable_id` as primary function identity,
4. treat `xlf*` identity fields as interoperability metadata only,
5. use the typed context/query bundle from Section 22.4 as the current host/OxFml callback floor for the already-covered scope,
6. use the current returned-value split from Section 22.5:
   - ordinary value
   - `ValueWithPresentation`
   - typed host/provider outcome projection
7. keep richer callable provenance and final transport names out of the freeze for now.

### 23.5 Local Packet Updates Adopted
OxFunc has updated local packet doctrine accordingly:
1. `W044` now records the runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` direction explicitly,
2. `W046` now records that future registration/removal should produce explicit new snapshot generations,
3. `W042` now records the current callable-minimum convergence floor and no longer treats first-pass `ISOMITTED` semantics as a seam-lock driver.

### 23.5A Current `ISOMITTED` Reading
Current OxFunc reading after reprocessing the local `W38` evidence and runtime is:
1. `ISOMITTED` is now narrow rather than mysterious,
2. the important distinction is between:
   - explicit omitted placeholder preservation, for example `LAMBDA(a,b,ISOMITTED(b))(1,)`, and
   - ordinary arity under-application, for example `LAMBDA(a,ISOMITTED(a))()`,
3. the first lane should be preserved and visible to `ISOMITTED`,
4. the second lane should still fail as arity mismatch rather than manufacturing an omission channel,
5. that distinction is now aligned across local runtime, native evidence, and seam doctrine closely enough that `ISOMITTED` no longer needs to stay open as a first-freeze blocker.

### 23.6 Remaining Clarifications OxFunc Would Still Like
The current note exchange is convergent enough to proceed, but OxFunc would still like clarification on a few narrow points before treating the first application seam as frozen:
1. whether OxFml wants the callable-minimum facts represented only through the current contract docs for now, or whether it already wants direct first-pass snapshot fields for:
   - callable origin kind
   - callable capture mode
   - callable arity shape
   - callable invocation-contract meaning
2. whether OxFml wants the first application work to begin from:
   - committed snapshot-export ingestion,
   - or an immediately modeled runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` interface with the CSV only as a pinning artifact,
3. whether the current first-pass `W044` split:
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `interface_contract_ref`
   is already sufficient for OxFml semantic planning on the callable rows, or whether OxFml already sees a concrete insufficiency there.

### 23.7 Current OxFunc Response Reading
Current OxFunc response for the next round is:
1. we agree the exchange is close enough to work toward a first freezable seam for covered application work,
2. we agree the long-term direction should be runtime provider/snapshot rather than permanent build-time CSV coupling,
3. we agree the current callable question is now minimum semantic carrier versus provenance detail, not invocation viability,
4. we do not currently see a need to force an extra invocation-model field,
5. we want the next round to use concrete `W044` consumption or runtime-shape mismatches as triggers, not reopen broad note-only debate.

## 24. Next Round Outbound Position

### 24.1 Direct Answers To The Current OxFml Questions
Current OxFunc answers for the next sync are:
1. yes, OxFml can rely on the current `W044` first-pass fields for callable rows:
   - `surface_stable_id`
   - `entry_kind`
   - `registration_source_kind`
   - `canonical_surface_name`
   - `arg_preparation_profile` when populated
   - `metadata_status`
   - `special_interface_kind`
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `arity_shape_note`
   - `interface_contract_ref`
   - snapshot provenance fields
2. no, OxFunc does not currently think an additional explicit invocation-model field is required beyond the current invocation-contract-meaning idea,
3. yes, OxFunc supports the preferred long-term normative direction of:
   - runtime `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - registration/removal producing explicit new snapshot generations,
4. no, first-pass `ISOMITTED` should no longer be treated as a seam-lock blocker for the initial application freeze.

### 24.2 Recommended First-Freeze Working Rule
Current OxFunc recommendation is:
1. begin first application work from the committed snapshot/export surface because that is the already-pinned shared artifact,
2. in parallel, model the normative runtime seam as `LibraryContextProvider` / immutable `LibraryContextSnapshot`,
3. do not wait for the final runtime provider shape to be fully coded before consuming the committed snapshot in OxFml tests and semantic-plan fixtures,
4. treat any mismatch found during that consumption as the trigger for the next narrower seam change.

Current reading:
1. the CSV is the right immediate pinning surface,
2. the runtime provider/snapshot model is the right long-term implementation target,
3. these are compatible rather than competing paths.

### 24.3 Callable-Minimum Versus Provenance
Current OxFunc view remains:
1. semantically required shared callable minimum:
   - opaque callable identity
   - origin kind
   - capture mode
   - arity shape
   - invocation-contract meaning
2. provenance/replay-only by default:
   - parameter names
   - capture names
   - body detail
   - richer origin-specific transport detail
3. candidate names such as `callable_token`, `arity_shape`, and `invocation_contract_ref` should still be read as candidate labels rather than a demand for OxFml-local canonical names.

### 24.4 Current `ISOMITTED` Closure Reading
Current OxFunc reading is now:
1. `ISOMITTED` is narrow enough to stop treating it as a seam driver,
2. the key pinned distinction is:
   - explicit omitted placeholder is preserved and visible to `ISOMITTED`,
   - ordinary under-application remains an arity failure and does not manufacture an omission channel,
3. the native seeded lane `LAMBDA(a,b,ISOMITTED(b))(1,) -> TRUE` is now part of local replay evidence,
4. this should be treated as first-pass callable semantics already pinned, not as a reason to widen the minimum callable carrier.

### 24.5 Preferred Next Narrowing Trigger
OxFunc would like the next round to be driven by one of these concrete triggers only:
1. OxFml finds a concrete insufficiency in the current `W044` callable-row field split,
2. OxFml needs direct snapshot columns for callable-minimum semantic facts rather than following `interface_contract_ref`,
3. the first runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` model exposes a mismatch with the current CSV reading,
4. built-in `xlf*` metadata proves insufficient for host/OxFml routing on the built-in scope,
5. a proving-host/runtime artifact demonstrates that an extra invocation-model field is actually required.

### 24.6 Current OxFunc Request Back To OxFml
For the next reply, OxFunc would like OxFml to say:
1. whether the current `W044` callable-row split is already sufficient for semantic planning,
2. whether OxFml wants the callable-minimum semantic facts promoted into direct snapshot columns now or is content to keep them in contract docs for one more round,
3. whether OxFml agrees with the recommended first-freeze working rule:
   - consume the committed snapshot now,
   - model runtime provider/snapshot in parallel,
   - use concrete mismatches as the only trigger for further seam narrowing.

## 25. Processed Latest OxFml Note

### 25.1 What OxFunc Incorporated
OxFunc has now incorporated the latest OxFml note as answering the key open questions from Section 24:
1. OxFml accepts the current `W044` callable-row split as sufficient for first-pass semantic planning,
2. OxFml does not need callable-minimum semantic facts promoted into direct snapshot columns in this round,
3. OxFml agrees with the recommended first-freeze working rule:
   - consume the committed snapshot now,
   - model runtime provider/snapshot in parallel,
   - use concrete mismatches as the trigger for further seam narrowing,
4. OxFml accepts built-in `xlf*` metadata as sufficient first-pass compatibility metadata for built-in routing as long as `surface_stable_id` remains primary,
5. OxFml agrees that `ISOMITTED` is no longer a first-freeze blocker.

### 25.2 Local Integration Result
Current local integration result is:
1. `W044` should now be read as good enough for the first bounded consumer round rather than as waiting on another callable-row field split debate,
2. `W042` remains the deferred owner for later callable field-lock only if a new concrete trigger appears,
3. the next agreed seam-hardening owners are now:
   - `W047` typed context and query bundle freeze,
   - `W048` return surface and publication-hint freeze,
   - `W049` runtime library-context provider consumer model,
4. `W046` continues to own registered-external catalog and registration-update direction, but not as a blocker for the first covered built-in application freeze.

### 25.3 Current OxFunc Response Direction
Current OxFunc reading for the next response is:
1. the seam is converged enough to stop spending the next round on callable-row sufficiency questions,
2. the next useful work is now the three agreed successor packets:
   - typed context/query bundle,
   - return-surface split,
   - runtime provider/snapshot consumer model,
3. callable minimum stays semantically narrowed but not field-name frozen,
4. the next note round should therefore focus on any concrete changes or proposed shapes coming out of `W047` / `W048` / `W049`, not revisit the already-accepted first-freeze working rule.

### 25.4 Remaining Clarifications OxFunc Still Wants
The remaining clarifications are now narrower than before:
1. for the typed context/query bundle:
   - whether OxFml wants exactly the current OxFunc query names and result-type partitioning,
   - or wants a merged/split host capability surface before the first shared freeze,
2. for the return surface:
   - whether OxFml wants to preserve the current explicit split between ordinary value, `ValueWithPresentation`, and typed host/provider projection,
   - or wants a different returned-surface factoring before the first shared freeze,
3. for the runtime provider/snapshot model:
   - whether OxFml wants one minimal runtime shape that mirrors the CSV closely,
   - or a cleaner runtime-only shape plus a separate CSV mapping layer.

### 25.5 Current Outbound Ask For The Next Round
Current OxFunc ask for the next bounded round is:
1. review and tighten the first shared typed context/query bundle (`W047`),
2. review and tighten the first shared return-surface split (`W048`),
3. review and tighten the first runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` consumer/model shape (`W049`),
4. use only concrete artifact mismatches from those packets as triggers for any further callable or catalog-field narrowing.

## 26. Final Processed OxFml Update And Final OxFunc Response

### 26.1 What OxFunc Incorporates From The Final OxFml Update
OxFunc reads the final OxFml update in this exchange as confirming the successor-packet seam plan rather than reopening any earlier callable or catalog sufficiency questions.

OxFunc now incorporates the following as agreed for the first application freeze:
1. the typed context/query bundle should stay capability-scoped and typed,
2. the current OxFunc query names and result-type partitioning are acceptable as the first freeze candidate,
3. the current returned-value split is acceptable as the first freeze candidate:
   - ordinary value
   - `ValueWithPresentation`
   - typed host/provider outcome projection
4. the preferred runtime library-context direction is:
   - runtime `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - explicit generation changes when registration/removal changes the library context
5. the runtime object model should prefer a cleaner runtime-only shape plus an explicit CSV/export mapping layer,
6. the current `W044` export remains the pinned interchange/debug artifact for the bounded first application round,
7. no further note-only callable sufficiency debate is needed for the already-covered scope.

### 26.2 Local Integration Result
OxFunc has now integrated that reading locally as:
1. `W047` owns the first shared typed context/query bundle freeze and now treats the current query/result names as the first freeze candidate,
2. `W048` owns the first shared returned-surface freeze and now treats the current three-way return split as the first freeze candidate,
3. `W049` owns the first runtime provider/snapshot consumer model and now treats a runtime-only shape plus CSV mapping layer as the first freeze candidate,
4. `W044` remains the current pinned artifact for bounded consumption and mismatch reporting,
5. `W042` remains deferred and should only reopen if later concrete evidence forces narrower callable-carrier changes,
6. the remaining current-scope hard packets after the seam freeze remain:
   - `W014` for `@` / `SINGLE`,
   - `W046` for worksheet `CALL` / `REGISTER.ID`,
   - residual `W023` publication/rich-value work for `HYPERLINK` / `IMAGE`.

### 26.3 Final OxFunc Response For This Exchange
OxFunc's final response in this exchange is:
1. yes, OxFunc agrees the next useful work is `W047` / `W048` / `W049` rather than more broad seam discussion,
2. yes, OxFunc agrees the current typed context/query names and result partitions should be treated as the first shared freeze candidate,
3. yes, OxFunc agrees the current returned-surface split should be treated as the first shared freeze candidate,
4. yes, OxFunc agrees the runtime library-context model should prefer a cleaner runtime-only shape plus explicit CSV/export mapping,
5. yes, OxFunc agrees the committed `W044` snapshot should remain the immediate shared artifact while the runtime model is developed in parallel,
6. no additional clarification is currently required before treating the first application seam as provisionally freezable for the already-covered scope,
7. that freezable seam does not by itself remove `W014`, `W046`, or residual `W023` `IMAGE` / `HYPERLINK` work from the current completion target.

### 26.4 Narrow Remaining Clarifications
The only remaining clarifications OxFunc still expects to matter are now implementation-facing rather than note-facing:
1. a concrete OxFml consumer mismatch against the current typed context/query bundle,
2. a concrete mismatch in how `ValueWithPresentation` or typed host/provider projections need to be consumed,
3. a concrete mismatch between the preferred runtime-only snapshot model and the current CSV/export artifact,
4. a later callable or provider artifact that proves one of the currently deferred seams must be narrowed after all.

### 26.5 Working Rule After This Exchange
After this final exchange, OxFunc's working rule is:
1. stop using note rounds to revisit already-converged callable sufficiency questions,
2. treat the first application seam as provisionally freezable for the already-covered scope,
3. drive any further seam change only from concrete artifacts, consumer mismatches, or implementation pressure coming out of `W047`, `W048`, or `W049`,
4. continue in parallel or immediately afterward with the remaining current-scope packets:
   - `W014`
   - `W046`
   - residual `W023` `IMAGE` / `HYPERLINK`,
5. keep only genuinely deferred topics in their deferred worksets unless and until concrete triggers appear.

## 27. W47 Typed Context And Query Bundle Freeze - First Packet Output

### 27.1 Current Frozen Bundle Artifact
OxFunc has now turned `W047` into a concrete artifact set:
1. `docs/function-lane/FUNCTION_SLICE_TYPED_CONTEXT_AND_QUERY_BUNDLE_CONTRACT_PRELIM.md`
2. `docs/function-lane/W47_TYPED_CONTEXT_QUERY_DEPENDENCY_MAP.csv`
3. `docs/function-lane/W47_EXECUTION_RECORD.md`

### 27.2 Current First-Freeze Candidate
The current first-freeze candidate remains:
1. `ReferenceResolver`
2. `NowProvider`
3. `TodayProvider`
4. `RandomProvider`
5. `LocaleFormatContext`
6. `HostInfoProvider`
7. `RtdProvider`
8. `RegisteredExternalProvider`

Current OxFunc reading:
1. the current query names and result partitions remain good enough as the first shared freeze candidate,
2. `RtdProvider` remains separate from `HostInfoProvider`,
3. `RegisteredExternalProvider` should also remain separate from `HostInfoProvider`,
4. the bundle remains capability-scoped and typed rather than collapsing into raw workbook/host objects.

### 27.3 Concrete Ask Back To OxFml
For `W047`, OxFunc now only wants OxFml to report concrete mismatches against:
1. the current query/result names,
2. the current dependency map,
3. the current separation between:
   - `ReferenceResolver`
   - time/random providers
   - `LocaleFormatContext`
   - `HostInfoProvider`
   - `RtdProvider`
   - `RegisteredExternalProvider`

### 27.4 Reconciliation Artifact
OxFunc has now added explicit reconciliation artifacts for `W047`:
1. `docs/function-lane/W47_OXFML_CONSUMER_RECONCILIATION.md`
2. `docs/function-lane/W47_CONSUMER_MISMATCH_LEDGER.csv`

Current reading:
1. the final OxFml note introduced no concrete mismatch against the current `W047` bundle,
2. the packet is now locally pinned at `scope_complete` / `target_complete`,
3. integration remains partial until OxFml consumes the frozen bundle in the bounded first application round.

## 28. W48 Return Surface And Publication Hint Freeze - First Packet Output

### 28.1 Current Frozen Return Artifact
OxFunc has now turned `W048` into a concrete artifact set:
1. `docs/function-lane/FUNCTION_SLICE_RETURN_SURFACE_AND_PUBLICATION_HINT_CONTRACT_PRELIM.md`
2. `docs/function-lane/W48_RETURN_SURFACE_CLASS_MAP.csv`
3. `docs/function-lane/W48_EXECUTION_RECORD.md`

### 28.2 Current First-Freeze Candidate
The current first-freeze candidate remains:
1. ordinary value
2. `ValueWithPresentation`
3. typed host/provider outcome projection

Current OxFunc reading:
1. `ValueWithPresentation` is now the shared publication-aware class for:
   - `NOW`
   - `TODAY`
   - `HYPERLINK`
2. typed provider-outcome projection remains explicit for:
   - `TRANSLATE`
   - `RTD`
3. `IMAGE` remains a sibling rich-value/publication packet pressure, not a reason to widen the `W048` freeze prematurely.

### 28.3 Concrete Ask Back To OxFml
For `W048`, OxFunc now only wants OxFml to report concrete mismatches against:
1. the current three-way return split,
2. the current classification map,
3. the current reading that typed provider outcomes remain explicit at the callback boundary even when the final worksheet-visible result lands in ordinary value/error space.

### 28.4 Reconciliation Artifact
OxFunc has now added explicit reconciliation artifacts for `W048`:
1. `docs/function-lane/W48_OXFML_CONSUMER_RECONCILIATION.md`
2. `docs/function-lane/W48_CONSUMER_MISMATCH_LEDGER.csv`

Current reading:
1. the final OxFml note introduced no concrete mismatch against the current `W048` return split,
2. the packet is now locally pinned at `scope_complete` / `target_complete`,
3. integration remains partial until OxFml consumes the frozen split in the bounded first application round.

## 29. W49 Runtime Library Context Provider Consumer Model - First Packet Output

### 29.1 Current Runtime Model Artifact
OxFunc has now turned `W049` into a concrete artifact set:
1. `docs/function-lane/FUNCTION_SLICE_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL_PRELIM.md`
2. `docs/function-lane/W49_RUNTIME_LIBRARY_CONTEXT_CSV_TO_RUNTIME_MAPPING.csv`
3. `docs/function-lane/W49_RUNTIME_LIBRARY_CONTEXT_CONSUMER_WALKTHROUGH.md`
4. `docs/function-lane/W49_EXECUTION_RECORD.md`

### 29.2 Current First-Freeze Candidate
The current first-freeze runtime model remains:
1. `LibraryContextProvider`
2. immutable `LibraryContextSnapshot`
3. grouped runtime entry model
4. explicit CSV/export mapping layer
5. explicit generation changes on registration/removal

Current OxFunc reading:
1. the runtime model should not mirror the CSV column-for-column,
2. the CSV remains the pinned interchange/debug artifact,
3. runtime consumers should group fields by identity, naming, planner-visible semantics, seam guidance, and provenance,
4. future registered-external change pressure from `W046` should produce fresh immutable snapshot generations rather than mutating current snapshot meaning silently.

### 29.3 Concrete Ask Back To OxFml
For `W049`, OxFunc now only wants OxFml to report concrete mismatches against:
1. the current runtime provider/snapshot split,
2. the current CSV-to-runtime mapping,
3. the current consumer walkthrough,
4. the current reading that runtime implementation should use a cleaner object model plus mapping layer rather than direct CSV mirroring.

### 29.4 Reconciliation Artifact
OxFunc has now added explicit reconciliation artifacts for `W049`:
1. `docs/function-lane/W49_OXFML_CONSUMER_RECONCILIATION.md`
2. `docs/function-lane/W49_CONSUMER_MISMATCH_LEDGER.csv`

Current reading:
1. the final OxFml note introduced no concrete mismatch against the current `W049` runtime model,
2. the packet is now locally pinned at `scope_complete` / `target_complete`,
3. integration remains partial until OxFml consumes the frozen runtime model in the bounded first application round.

## 31. Seam Requirements Consolidation And Evaluation Adapter Request (2026-03-24)

### 31.1 What Changed

OxFunc has completed a W051 execution round bringing 6 more functions to `function-phase-complete` (ROWS, COLUMNS, RANDBETWEEN, VALUETOTEXT, RANDARRAY, TRIMRANGE), scaffolding 2 more as `scaffold-partial` waiting on W038 callable infrastructure (GROUPBY, PIVOTBY), and closing 3 trim-reference operators.

The W051 inventory dropped from 25 to 16 rows. The remaining rows are either W038-blocked (callable family), W014-blocked (`@`), W023-blocked (HYPERLINK/IMAGE publication), or W046-blocked (CALL/REGISTER.ID).

Two of those blockers — W038 and W014 — depend on OxFml seam progress. OxFunc has now consolidated all scattered seam requirements into one document and defined a concrete integration test artifact.

### 31.2 New Document: Consolidated Seam Requirements

**File:** `docs/upstream/OXFUNC_OXFML_SEAM_REQUIREMENTS_CONSOLIDATED.md`

This replaces no existing note, but it is now the single reference for what OxFunc needs from OxFml and why. It consolidates requirements from 10 source documents into 7 numbered requirement groups:

1. **SR-ARG-01 through SR-ARG-04**: Prepared argument seam — direct scalar vs array-like, omitted/empty/error distinction, reference identity preservation, caller context.
2. **SR-CALL-01 through SR-CALL-05**: Callable value seam — carrier minimum fields, typed invocation interface, lexical capture, arity checking, parse-time rejection of duplicate names/params.
3. **SR-AT-01 through SR-AT-04**: Implicit intersection (`@`) seam — explicit `@` survival, operand provenance, caller context, trace distinguishability.
4. **SR-RET-01 through SR-RET-03**: Return surface — all value types, presentation hints, lambda publication restriction.
5. **SR-LIB-01 through SR-LIB-03**: Library context / catalog — per-entry fields, snapshot identity, current freeze artifact.
6. **SR-AVAIL-01 through SR-AVAIL-03**: Availability / gating taxonomy — parse/bind, runtime, post-dispatch stages.

Each requirement has: what OxFunc needs, why (which functions break without it), and the evidence source.

### 31.3 New Requirement: Evaluation Adapter Artifact

Section 10 of the consolidated document specifies a concrete artifact OxFml must provide:

**"OxFml Evaluation Adapter for OxFunc Seam Validation"**

Purpose: OxFunc has 1042 unit tests, all using mock resolvers. None exercise OxFml's real preparation pipeline. Both sides can pass their own tests while the seam is broken. The adapter closes this gap.

Shape:
1. Accepts a formula string, a caller cell address, and a cell-value fixture.
2. Uses OxFml's real parser, binder, and argument preparation logic.
3. Calls into OxFunc's real surface dispatch.
4. Returns the result value plus trace metadata showing which `ArgPreparationProfile` was used and what provenance each argument received.

The document defines 38 validation scenarios across 6 categories:
- **A (10):** Argument preparation — scalar/array coercion, omitted args, reference preservation, caller context
- **B (7):** `@` implicit intersection — all scalarization modes, provenance preservation
- **C (14):** Callable/LAMBDA/LET — binding, invocation, lexical capture, higher-order helpers, error cases
- **D (6):** Return surface — all value types
- **E (3):** Volatile/provider functions — random seed, time injection
- **F (5):** Cross-seam stress tests — combining multiple requirements

The critical diagnostic scenarios are:
- **A02 vs A03**: `SUM("2",TRUE)` must return 3 while `SUM({"2",TRUE})` must return 0. If both return the same value, SR-ARG-01 is violated.
- **C05**: `=LET(x,1,f,LAMBDA(y,x+y),LET(x,99,f(10)))` must return 11, not 109. If it returns 109, SR-CALL-03 (lexical capture) is violated.
- **B01**: `=@A1:A3` at caller B2 with A1=10,A2=20,A3=30 must return 20 (not 10). If it returns 10, SR-AT-02 (operand provenance) is violated.

### 31.4 Open Questions For This Round

The consolidated document (§8) lists 4 decisions OxFunc is waiting on:

1. **Q1:** Who evaluates `@`? OxFunc-side operator, or OxFml evaluates upstream?
2. **Q2:** What prepared-operand vocabulary will OxFml implement?
3. **Q3:** When will OxFml produce `LambdaValue` carriers for LET/LAMBDA?
4. **Q4:** Will OxFml provide a real `CallableInvoker` implementation?

OxFunc does not need all four answered at once. Any one of them unblocks meaningful progress. Q4 + Q3 together unblock the entire callable family (11 functions). Q1 + Q2 together unblock `@`.

### 31.5 What OxFunc Has Ready For Integration

| Component | Status |
|-----------|--------|
| `LambdaValue` type | Implemented in `value.rs` |
| `CallableInvoker` trait | Implemented in `callable_helpers.rs` |
| Stage1 expression interpreter (LET/LAMBDA/ISOMITTED) | Implemented in `callable_stage1_prepared.rs` |
| MAP/REDUCE/SCAN/BYROW/BYCOL/MAKEARRAY | Fully implemented + tested |
| `@` operator | Fully implemented + 8 empirical scenarios passing |
| Library context snapshot | First freeze published (`OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`) |
| 40+ functions at `function-phase-complete` | Full artifact stack (Rust + Lean + contract + tests) |
| Surface dispatch with 1042 passing tests | Ready to accept real prepared arguments |

### 31.6 Suggested Reading Order For OxFml

1. Start: `docs/upstream/OXFUNC_OXFML_SEAM_REQUIREMENTS_CONSOLIDATED.md` — the whole document, especially §2 (prepared args), §3 (callables), §4 (`@`), §10 (evaluation adapter).
2. If callable-focused: `crates/oxfunc_core/src/functions/callable_helpers.rs` (the `CallableInvoker` trait and higher-order helper implementations).
3. If `@`-focused: `docs/handoffs/HANDOFF_W014_IMPLICIT_INTERSECTION_TO_OXFML.md` and `crates/oxfunc_core/src/functions/op_implicit_intersection.rs`.
4. For catalog context: `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` and its README.

OxFunc is ready to iterate. Respond to any subset of Q1–Q4 or propose alternatives.

## 32. Processed OxFml §25 Response — Adapter Agreement And First-Wave Narrowing (2026-03-24)

### 32.1 What OxFunc Reads From OxFml §25

OxFml agrees:
1. The concrete adapter artifact is the right next closure step.
2. It is a bounded test/integration packet, not the normative production host API.
3. The committed W044 snapshot/export is the first pinning artifact.
4. CALL/REGISTER.ID and richer publication families are explicitly deferred from the first wave.

OxFml proposes richer adapter input than OxFunc's initial spec:
- adds `formula_channel`, `active_selection_anchor`, defined-name bindings, table packet inputs, typed context/query bundle, pinned `library_context_snapshot_ref`
- wants three output artifact families: preparation, evaluation, mismatch report
- wants fixture families organized by seam pressure rather than only by function name

OxFml assigns W049 (adapter harness) and W050 (fixture families) as owners.

### 32.2 OxFunc Accepts

1. **Bounded test artifact, not production API.** Agreed. OxFunc will not treat the adapter as normative production interface.
2. **W044 as first pinning artifact.** Agreed. OxFunc's current `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` is the committed snapshot.
3. **CALL/REGISTER.ID deferred from first wave.** Agreed. W046 remains a separate seam lane.
4. **Three output artifact families.** Agreed. The preparation artifact is the most critical for OxFunc — it shows whether argument provenance survived OxFml's pipeline.
5. **Fixture families by seam pressure.** Agreed and preferred. OxFunc's 38 scenarios are already organized by seam category (A=args, B=`@`, C=callable, D=returns, E=volatile, F=cross-seam). This maps directly to OxFml's proposed pressure families.

### 32.3 OxFunc Accepts With Clarification

1. **`formula_channel`**: OxFunc does not currently model channel-dependent function behavior (locale/version sweeps are IP-02, still planned). For the first wave, a single default channel is sufficient. OxFunc will not reject a channel field — it just won't exercise it yet.

2. **`active_selection_anchor`**: No current OxFunc function depends on selection anchor (as distinct from caller anchor). OxFunc accepts the field as optional and will not populate it in the first 38 scenarios. If OxFml has evidence that a function depends on it, that would be a new finding.

3. **Defined-name bindings**: Required for the callable scenarios (C01–C14 involve LET/LAMBDA, which are formula-level, not defined-name). For the first wave, defined-name bindings can be empty unless OxFml wants to exercise the defined-name callable lane. OxFunc accepts the field.

4. **Table packet inputs**: No current OxFunc scenario uses structured references. OxFunc accepts the field as optional. Structured-reference interaction is explicitly out of W014 scope.

5. **Typed context/query bundle**: OxFunc already has `HostInfoProvider`, `LocaleFormatContext`, `RtdProvider`, `RegisteredExternalProvider` as typed query surfaces. For the first wave, only `LocaleFormatContext` matters (for TEXT, DOLLAR, VALUE). The others can be stub/absent.

### 32.4 What OxFunc Needs In Return

The OxFml §25 response is convergent on the adapter shape but does not answer Q1–Q4 from the consolidated requirements. OxFunc reads this as: OxFml will surface answers through the adapter implementation itself, not through another note round.

OxFunc accepts this posture. The adapter implementation will concretely answer:
- **Q2 (prepared-operand vocabulary):** The preparation artifact output will show what provenance OxFml actually assigns. If it doesn't distinguish direct-scalar from array-like (SR-ARG-01), scenario A02 vs A03 will fail.
- **Q1 (who evaluates `@`):** If OxFml's adapter can pass scenarios B01–B07, the answer is demonstrated regardless of which side technically evaluates.
- **Q3 and Q4 (callable carriers and invoker):** If scenarios C01–C14 pass, callable formation and invocation work. If they fail, the mismatch report will show exactly where.

### 32.5 OxFunc Concrete Next Steps

1. OxFunc will keep the 38 scenarios stable as the canonical first-wave fixture set. They are in `docs/upstream/OXFUNC_OXFML_SEAM_REQUIREMENTS_CONSOLIDATED.md` §10.4.
2. OxFunc will add new scenarios only if concrete mismatches from the adapter surface new distinctions.
3. OxFunc will not reopen callable carrier or `@` seam debates unless the adapter exposes a real contradiction.
4. OxFunc considers this round convergent on shape. The next productive artifact is the adapter itself.

### 32.6 Reply To OxFml §24 (Editor Help / Signature Packet)

OxFunc notes OxFml's §24 proposal for editor help and signature metadata.

Quick answers to OxFml's asks:
1. **Help/signature retrieval:** A sibling help provider keyed by the same snapshot identity is fine. No need to overload the hot-path semantic snapshot.
2. **Response shape:** The proposed fields (`stable_function_id`, `display_name`, `signature_forms`, `short_description`, `availability_summary`, `deferred_or_profile_limited`, optional `documentation_ref`) are sufficient for first editor/signature-help use.
3. **Semantic truth vs presentation prose:** `stable_function_id`, `signature_forms` (parameter labels, arity bounds), and `availability_summary` are semantic truth. `short_description` and `documentation_ref` are presentation prose. `display_name` is presentation-first but should be consistent with the catalog canonical name.
4. **Runtime-registered extension functions:** These should appear under snapshot identity with `entry_kind = external_registered_function` and whatever metadata the registration provides. If registration metadata is incomplete, the help provider should return a minimal response (id + arity) rather than refusing.

This lane should remain a bounded editor/help packet. OxFunc considers it low-priority relative to the adapter artifact but non-blocking.

## 33. Processed OxFml §26–§27 — Adapter Convergence And First-Wave Readback (2026-03-25)

### 33.1 What OxFunc Reads From OxFml §26

OxFml accepts:
1. The narrower first-wave reading for the adapter.
2. The OxFunc W047/W048/W049 packet artifacts as pinned downstream freeze candidates.
3. No note-level mismatch against those three packets.
4. The next honest reply should be implementation-facing and mismatch-driven.

OxFunc accepts this. Round convergent on adapter shape — confirmed.

### 33.2 What OxFunc Reads From OxFml §27

This is the critical update. OxFml has now:
1. Built a real W049 adapter against OxFunc's real surface dispatch.
2. Established a first pinned W050 fixture corpus in machine-readable form.
3. Passed 43 of 45 scenarios.
4. Deferred only C12 and C14.
5. Made concrete local widenings to reach this floor:
   - parser correction for inline-array element separation,
   - trailing omitted-argument trimming at the call boundary,
   - wider local semantic metadata (ROWS, COLUMNS, PI, CONCAT, RANDBETWEEN, VALUETOTEXT, VLOOKUP),
   - explicit `@` precedence and scalarization alignment through the OxFunc implicit-intersection operator surface,
   - lambda over-application surfacing as worksheet `#VALUE!` rather than adapter failure.

### 33.3 Scenario Count Clarification

OxFml is correct. The authoritative first-wave table in `docs/upstream/OXFUNC_OXFML_SEAM_REQUIREMENTS_CONSOLIDATED.md` §10.4 enumerates 45 scenario ids (A01–A10, B01–B07, C01–C14, D01–D06, E01–E03, F01–F05). The "38" in earlier OxFunc prose was a pre-publication planning estimate that was superseded when the full table was written. OxFunc corrects its own earlier wording: the authoritative first-wave count is **45**, not 38. OxFml should continue treating the published 45-id table as the pinned first-wave fixture catalog.

### 33.4 C12 and C14 Residuals

**C12 (`=LAMBDA(x, x+1)` → expected `#CALC!`):**

OxFunc reads OxFml's deferral as: OxFml currently preserves the callable value at the publication boundary rather than mapping an uninvoked LAMBDA to `#CALC!`.

OxFunc position:
1. The empirical Excel fact is pinned: bare uninvoked `LAMBDA(...)` publishes as `#CALC!` on the current baseline (W38 evidence, LET_LAMBDA_PIN_DOWN_RESPONSE §3 item 5).
2. This is a publication-boundary rule, not a semantic-value rule. The callable value exists internally — it just cannot be displayed in a cell.
3. OxFunc does not require OxFml to implement this mapping immediately. It is acceptable for the first adapter wave to defer C12 as long as:
   - the callable value is correctly formed internally (SR-CALL-01),
   - the deferral is explicitly tracked,
   - a future wave maps uninvoked callable → `#CALC!` at the publication boundary.
4. OxFunc does not treat this deferral as a seam violation. It is a publication-policy residual.

**C14 (`=LET(x, 1, x, 2, x)` → expected parse error):**

OxFunc reads OxFml's deferral as: duplicate LET-name rejection needs a dedicated bind/reject artifact rather than silent evaluation fallback.

OxFunc position:
1. The empirical Excel fact is pinned: duplicate LET names are rejected at formula admission (W38 evidence, LET_LAMBDA_PIN_DOWN_RESPONSE §3 item 1).
2. This is a parse/bind responsibility (SR-CALL-05). OxFunc expects OxFml to reject this formula before it reaches evaluation.
3. OxFunc does not require this in the first adapter wave. It is acceptable to defer as long as:
   - the deferral is explicitly tracked,
   - a future wave rejects the formula at bind time rather than silently evaluating.
4. OxFunc does not treat this deferral as a seam violation. It is a bind-time validation residual.

### 33.5 Assessment

**The seam is now functionally validated for the admitted first-wave slice.**

43/45 scenarios passing through OxFml's real parser/binder/preparation pipeline into OxFunc's real surface dispatch is the strongest integration evidence to date. The two deferred residuals (C12, C14) are both narrow publication/validation policy questions, not structural seam gaps.

Concrete implications:
1. **SR-ARG-01 (scalar vs array):** Validated. A02 and A03 return different results through the real pipeline.
2. **SR-ARG-02 (omitted args):** Validated. A05 exercises omitted-argument preservation.
3. **SR-ARG-03 (reference identity):** Validated. A07, A08 exercise reference-visible functions.
4. **SR-ARG-04 (caller context):** Validated. A09, A10, B01–B02 exercise caller-relative behavior.
5. **SR-AT-01 through SR-AT-04 (`@`):** Validated. B01–B07 pass through the real `@` scalarization path.
6. **SR-CALL-01 through SR-CALL-04 (callable):** Validated. C01–C11, C13 exercise LET, LAMBDA, lexical capture, higher-order helpers, and arity rejection.
7. **SR-CALL-05 (duplicate name rejection):** Deferred (C14). Tracked, not blocking.
8. **SR-RET-01 through SR-RET-03 (returns):** SR-RET-01 and SR-RET-02 validated (D01–D06). SR-RET-03 deferred (C12). Tracked, not blocking.

### 33.6 What This Unblocks

With the seam functionally validated, OxFunc can now:
1. **Claim integration-level evidence** for functions exercised through the adapter (not just unit-test-level).
2. **Move W014 (`@`) toward closure** — the adapter proves the `@` operand provenance seam works end-to-end.
3. **Move W038 callable family toward closure** — the adapter proves callable formation and invocation work end-to-end for the admitted direct-invocation, LET, LAMBDA, MAP, REDUCE, SCAN, BYROW, BYCOL, MAKEARRAY slice.
4. **Unblock GROUPBY and PIVOTBY** — if the adapter's `CallableInvoker` works for C06–C11, it can work for GROUPBY/PIVOTBY with built-in aggregation functions.

### 33.7 OxFunc Concrete Next Steps

1. OxFunc considers this round convergent. No note-level objections.
2. OxFunc will track C12 and C14 as residuals for a future adapter wave, not as blockers.
3. OxFunc will use the 43-passing adapter results as integration evidence in completion claims for W014, W038, and function-phase-complete promotions.
4. The next useful OxFunc work is to exercise the adapter's `CallableInvoker` for GROUPBY and PIVOTBY, and to update the W051 inventory to reflect the new integration floor.
5. OxFunc does not request another note round. The seam is working. Future mismatches should surface through the adapter fixture corpus, not through notes.

## 30. W46 CALL / REGISTER.ID Runtime Narrowing

### 30.1 Current Admitted Runtime Artifact
OxFunc has now moved `W046` beyond catalog-only planning:
1. `docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md`
2. `docs/function-lane/W46_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W46_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W46_EXECUTION_RECORD.md`

### 30.2 Current OxFunc Runtime Reading
Current typed seam is:
1. `RegisterIdRequest`
2. `RegisteredExternalDescriptor`
3. `RegisteredExternalCallRequest`
4. `RegisteredExternalProvider`

Current reading:
1. `REGISTER.ID` normalizes a typed registration request and receives a typed descriptor from host/OxFml,
2. `CALL` normalizes either:
   - numeric register-id target, or
   - direct `{ library, procedure, optional type_text }` target,
3. host/OxFml still owns registration-handle allocation and actual external invocation,
4. OxFunc only projects the worksheet-visible result.

### 30.3 Seeded Native Baseline
Pinned native Excel4 replay now shows:
1. `REGISTER.ID("Kernel32","GetTickCount","J!")` returns a numeric register id,
2. `CALL(register_id)` succeeds on the seeded zero-argument lane,
3. `CALL("Kernel32","GetTickCount","J!")` succeeds directly,
4. `CALL("Kernel32","MulDiv","JJJJ",6,7,3)` returns `14`,
5. `CALL(register_id,6,7,3)` returns `14`,
6. the seeded zero-argument `GetTickCount` lane also succeeds when `type_text` is omitted.

### 30.4 Concrete Ask Back To OxFml
For the next round, OxFunc only wants clarification on:
1. whether OxFml agrees `RegisteredExternalProvider` should stay separate from `HostInfoProvider`,
2. whether the first bounded consumer model should carry `RegisterIdRequest` / `RegisteredExternalDescriptor` directly or only through the `W049` runtime snapshot/provider layer,
3. whether OxFml sees any concrete mismatch with the current reading that `CALL` runtime stays above OxFunc except for request normalization and result projection.

## 34. Post-W52 Residual Seam Corrections For OxFml (2026-03-26)

After the standalone `SUMIF` completion packet and a cleanup pass over the broader OxFunc-side adapter corpus, OxFunc now narrows the next OxFml-facing note to two seam corrections only.

This is not a request to change OxFunc function definitions or metadata profiles. The remaining OxFml-facing ask is:
1. unary negative literal handling through the OxFml adapter path,
2. blank single-cell stand-in resolution for ordinary worksheet references.

### 34.1 Unary Negative Literal Evidence

Direct Excel readback on `2026-03-26` confirms:
1. `=SIGN(-5)` -> `-1`
2. `=PV(0.05,10,-100)` -> `772.1734929184813`
3. `=FV(0.05,10,-100)` -> `1257.789253554883`

Current OxFunc-side seam run still fails these rows before worksheet-value comparison:
1. `FN-SIGN-01` -> `OxFunc surface evaluation failed for SIGN: Value`
2. `FN-PV-01` -> `OxFunc surface evaluation failed for PV: Value`
3. `FN-FV-01` -> `OxFunc surface evaluation failed for FV: Value`

Current OxFunc reading:
1. the shared pressure is the negative literal, not the function family,
2. this now looks like an OxFml-side parse/bind/evaluation seam issue rather than an OxFunc function-kernel issue,
3. OxFunc is therefore filing this as a seam-correction ask rather than touching `SIGN`, `PV`, or `FV` locally.

### 34.2 Blank Single-Cell Stand-In Evidence

Direct Excel readback on `2026-03-26` confirms:
1. `=ISBLANK(A1)` on a blank `A1` -> `TRUE`

Current OxFunc-side seam run still fails this row with:
1. `FN-ISBLANK-01` -> `reference resolution failed: UnresolvedReference { target: "A1" }`

Current OxFunc reading:
1. OxFml's stand-in local resolver is currently treating absent single-cell worksheet references as unresolved,
2. the same stand-in path already treats absent cells inside area-reference expansion as blank cells,
3. the single-cell path should be aligned to the same blank-cell semantics for this packet.

### 34.3 What OxFunc Already Corrected Locally

OxFunc has already corrected its local broad-corpus fixture expectations where the fixtures themselves were wrong. That local cleanup should not be read as an OxFml action item by itself.

Rows corrected locally on the OxFunc side:
1. `ASIN`
2. `DATE`
3. `DAY`
4. `EDATE`
5. `IFNA`
6. `PV`
7. `FV`
8. `LARGE`
9. `SMALL`
10. `COUNTIF`
11. `AVERAGEIF`

These rows were fixture/value or fixture/profile corrections in OxFunc's local broader corpus, not new OxFml semantic obligations unless OxFml mirrors the same broader corpus.

### 34.4 What OxFunc Is Not Asking OxFml To Own

The remaining reviewed residuals:
1. `ASINH` low-order publication drift
2. `PMT` low-order publication drift

are not being handed to OxFml as seam defects. OxFunc currently treats those as local residuals unless new evidence says otherwise.

### 34.5 Filed Handoff Packet

This note is paired with:
1. `docs/handoffs/HANDOFF_W052_UNARY_NEGATIVE_AND_BLANK_SINGLE_CELL_TO_OXFML.md`
2. `docs/handoffs/HANDOFF_REGISTER.csv` row `HO-FN-003`

## 35. Processed OxFml §29 Response - Residual Seam Fixes Confirmed (2026-03-26)

OxFunc has now processed OxFml Section 29 from `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` and verified the claimed seam corrections from the OxFunc side.

What OxFunc reads from OxFml Section 29:
1. OxFml corrected unary signed-literal parsing/binding locally.
2. OxFml corrected absent single-cell stand-in resolution so blank worksheet references no longer fail as unresolved references.
3. OxFml explicitly preserved blank-cell identity rather than collapsing blank to empty string.
4. OxFml does not reopen broader callable/catalog note lanes as part of these corrections.

OxFunc verification:
1. reran `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --test oxfml_seam_integration -- --nocapture`
2. prior seam failures for `SIGN`, `PV`, `FV`, and `ISBLANK` no longer fail as seam defects
3. the broad residual set is now reduced to worksheet-value mismatches only:
   - `ASINH`
   - `PV`
   - `FV`
   - `PMT`

Current OxFunc reading:
1. `HO-FN-003` is acknowledged on the OxFml side and verified from the OxFunc side.
2. `BLK-FN-012` and `BLK-FN-013` are resolved.
3. the remaining work now sits entirely in OxFunc-local publication/parity investigation under `W053`.
