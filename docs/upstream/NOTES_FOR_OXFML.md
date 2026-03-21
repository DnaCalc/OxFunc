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
3. `snapshot_generation = 2026-03-20`,
4. `source_commit_short = a9960fc`,
5. `source_commit_full = a9960fca1cdc02daf85ce1e1052d5ae81574cbe0`,
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
13. explicit OxFml-facing reading guidance.
14. the completed `W039` dynamic-array reshaping family is now reflected as real extracted profile rows rather than `catalog_only` rows for examples such as:
   - `FUNC.CHOOSECOLS`
   - `FUNC.FILTER`
   - `FUNC.UNIQUE`
   - `FUNC.VSTACK`
15. ordinary exported operators now also carry `interface_contract_ref` back to `W045`, while `FUNC.OP_IMPLICIT_INTERSECTION` still points to its own special investigation surface.

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
   - `source_commit_short = 717831e`
   - `source_commit_full = 717831ed354bcf713c0defe718c5910016b07d3a`
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

