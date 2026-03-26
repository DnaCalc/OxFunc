# OxFunc → OxFml Seam Requirements — Consolidated

Status: `provisional`
Owner: OxFunc
Last updated: 2026-03-24

## 1. Purpose

OxFunc has filed seam requirements to OxFml across multiple documents since W014. This document consolidates them into one reference. It replaces no source document — it cross-references them — but it is the single place to read what OxFunc currently needs from OxFml and why.

### Source Documents

| # | Document | Primary Content |
|---|----------|-----------------|
| 1 | `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md` | Ownership split across 5 coupled seam surfaces |
| 2 | `docs/function-lane/OXFML_OXFUNC_NEXT_ROUND_STABILIZATION_TOPICS.md` | Library context, callable carrier, availability taxonomy |
| 3 | `docs/function-lane/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_RESPONSE_V1.md` | Callable carrier minimum fields, invocation boundary |
| 4 | `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V2.md` | Operator admission, library context fields, availability stages |
| 5 | `docs/upstream/NOTES_FOR_OXFML.md` | Prepared arg/result vocabulary, evaluation mode, `@` position |
| 6 | `docs/handoffs/HANDOFF_W014_IMPLICIT_INTERSECTION_TO_OXFML.md` | `@` provenance, caller context, reference-vs-array |
| 7 | `docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md` | What OxFml/FEC must preserve for `@` |
| 8 | `docs/worksets/W047_TYPED_CONTEXT_AND_QUERY_BUNDLE_FREEZE.md` | Context bundle freeze scope |
| 9 | `docs/worksets/W048_RETURN_SURFACE_AND_PUBLICATION_HINT_FREEZE.md` | Return surface freeze scope |
| 10 | `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md` | Runtime provider/snapshot model |

---

## 2. Prepared Argument Seam

This is the most fundamental seam. When OxFml prepares arguments for OxFunc evaluation, it must preserve the following distinctions.

### SR-ARG-01: Direct Scalar vs Array-Like Argument

OxFml must preserve whether each argument was a direct scalar expression or an array-like input.

**Why:** `SUM("2", TRUE)` ≠ `SUM({"2", TRUE})`. In the first case, `"2"` and `TRUE` are direct scalar arguments — SUM coerces text-to-number for direct scalars. In the second case, `{"2", TRUE}` is an array literal — SUM ignores non-numeric cells within range-scanned arrays. If OxFml collapses both into the same array representation, OxFunc cannot model the correct aggregate coercion policy.

**Functions affected:** All aggregate functions (SUM, AVERAGE, COUNT, COUNTA, MAX, MIN, SUMPRODUCT, criteria family), plus any function whose behavior differs on direct-scalar vs range-scan coercion.

**Source:** NOTES_FOR_OXFML.md §3, HIDDEN_MACHINERY_SEAM §2 (prepared argument/result seam).

### SR-ARG-02: Omitted / Empty Cell / Empty String / Error — Not Collapsed

OxFml must preserve the distinction between an omitted argument, an empty cell, an empty string, and an error value. These are four distinct semantic states.

**Why:** `VLOOKUP(x, range, col, )` (omitted match_type defaults to TRUE). `VLOOKUP(x, range, col, "")` (empty string coerces to 0 = exact match). `VLOOKUP(x, range, col, FALSE)` (explicit exact match). If omitted collapses into empty cell, default-argument semantics break.

**Functions affected:** Every function with optional arguments (majority of catalog). ISOMITTED depends on this distinction directly.

**Source:** NOTES_FOR_OXFML.md §3, LET_LAMBDA_PIN_DOWN_RESPONSE §3.

### SR-ARG-03: Reference Identity Preserved for Reference-Visible Functions

For functions declared `ArgPreparationProfile::RefsVisibleInAdapter`, OxFml must not dereference references before passing them to OxFunc. The reference identity (target string, kind) must arrive intact.

**Why:** ROW(A1:A10) returns an array of row numbers 1–10 by parsing the reference structure. If OxFml dereferences A1:A10 into an array of cell values, ROW has no way to know which rows were referenced — it would see values, not addresses.

**Functions affected:** ROW, COLUMN, ROWS, COLUMNS, OFFSET, INDEX, INDIRECT, AREAS, FORMULATEXT, SHEET, SHEETS, ADDRESS, and critically `@` (OP_IMPLICIT_INTERSECTION).

**Source:** NOTES_FOR_OXFML.md §5, IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION §5–6.

### SR-ARG-04: Caller Context Available at Evaluation Time

OxFml must make the caller context (at minimum: row, column, optional sheet prefix) available when invoking OxFunc evaluation.

**Why:** `@` selects a value from a single-column reference by matching the caller's row. ROW() with no arguments returns the caller's row number. CELL("row") returns information about the calling cell.

**Functions affected:** ROW, COLUMN (0-arg form), `@`, CELL, INFO, and any future caller-context-dependent function.

**Source:** HANDOFF_W014 §2–3, NOTES_FOR_OXFML.md §6.

---

## 3. Callable Value Seam

OxFunc has a fully implemented callable invocation infrastructure (trait, carriers, higher-order helpers). For it to work end-to-end, OxFml must produce callable value carriers and provide an invocation implementation.

### SR-CALL-01: Callable Carrier Minimum Fields

OxFml must produce a callable carrier with at least these fields when forming LAMBDA, LET, or Defined Name callable values:

```
callable_token          — opaque stable handle within active evaluation context
origin_kind             — helper_lambda | defined_name_callable | built_in_callable | external_registered_callable
arity_shape             — { min, max } for admission and invocation checks
capture_mode            — no_capture | lexical_capture
invocation_contract_ref — stable reference to callable invocation semantics/profile
```

**Why:** OxFunc dispatches callable invocation through a typed interface. Without the carrier, MAP/REDUCE/SCAN/BYROW/BYCOL/MAKEARRAY/GROUPBY/PIVOTBY cannot invoke their callable arguments.

**What does NOT need to be in the minimum carrier:** parameter-name surface, exact capture-name surface, helper source span, body-kind detail, explanatory text.

**OxFunc-side implementation:** `LambdaValue` in `crates/oxfunc_core/src/value.rs` (lines 298–353) already models this carrier.

**Source:** LET_LAMBDA_PIN_DOWN_RESPONSE §4, NOTES_FOR_OXFML.md §5.

### SR-CALL-02: Typed Invocation Interface

OxFml must provide a `CallableInvoker` implementation — a concrete object that can, given a callable token and prepared arguments, invoke the callable and return a prepared result.

```
invoke(callable_token, prepared_args, caller_context) → prepared_result | error
```

**Why:** OxFunc's higher-order helpers (MAP, REDUCE, SCAN, etc.) call `invoker.invoke(callable, args)` for each element/row/column. Without a real invoker, these functions can only reject callable tokens.

**OxFunc-side implementation:** `CallableInvoker` trait in `crates/oxfunc_core/src/functions/callable_helpers.rs` (lines 93–99). Currently only `RejectingCallableInvoker` exists as default.

**Source:** LET_LAMBDA_PIN_DOWN_RESPONSE §5, HIDDEN_MACHINERY_SEAM §3 (callable/helper value seam).

### SR-CALL-03: Lexical Capture Preserved

Created helper lambdas must preserve lexical meaning. Later helper-name shadowing must not rebind an already-created lambda.

**Why:** `=LET(x, 1, f, LAMBDA(y, x+y), LET(x, 99, f(10)))` must return 11, not 109. The inner `LET(x, 99, ...)` must not rebind the `x` captured by `f`.

**Evidence:** W38 empirical facts (LET_LAMBDA_PIN_DOWN_RESPONSE §3). OxFunc Stage1 expression interpreter verifies this.

**Source:** LET_LAMBDA_PIN_DOWN_RESPONSE §2.1.

### SR-CALL-04: Arity Checked at Invocation Boundary

Callable invocation must check arity before dispatch. Under-application and over-application yield `#VALUE!`.

**Evidence:** W38 empirical fact: direct under/over-application of LAMBDA yields `#VALUE!`.

**Source:** LET_LAMBDA_PIN_DOWN_RESPONSE §3.

### SR-CALL-05: Duplicate Names / Params Rejected at Parse/Bind

OxFml is responsible for rejecting duplicate LET names and duplicate LAMBDA parameter names at formula admission time, before evaluation reaches OxFunc.

**Evidence:** W38 empirical fact: duplicate LET names are rejected; duplicate LAMBDA params are rejected.

**Source:** LET_LAMBDA_PIN_DOWN_RESPONSE §3.

---

## 4. Implicit Intersection (`@`) Seam

The `@` operator cannot be implemented from plain dereferenced values. OxFml must preserve operand provenance.

### SR-AT-01: Explicit `@` Survives Parse/Bind

Explicit `@` must survive parse/bind as an evaluable node or as an equivalent explicitly traced scalarization step. It must not be treated as formatting trivia that can be silently discarded.

**Why:** Removing `@` changes publication from scalarization to spill. `=@SEQUENCE(3)` returns `1` (scalarized). `=SEQUENCE(3)` spills `{1; 2; 3}`.

**Source:** HANDOFF_W014 §3, IMPLICIT_INTERSECTION_INVESTIGATION §6.

### SR-AT-02: Operand Reaching `@` Must Distinguish Provenance

The operand that `@` receives must preserve whether it is:
1. already a scalar value (passthrough)
2. a reference/range (caller-context-dependent row/column selection)
3. an array payload (top-left selection)
4. a spill-anchor/spilled-range reference

**Why:** The scalarization rule differs by provenance:
- Single-column reference: selects caller's row
- Single-row reference: selects caller's column
- Array payload: selects top-left
- 2-D reference: `#VALUE!` on current baseline

If OxFml collapses reference and array into the same representation, OxFunc cannot distinguish these cases.

**Source:** HANDOFF_W014 §2 ("@ is not safely reconstructible after provenance erasure"), IMPLICIT_INTERSECTION_INVESTIGATION §2–5.

### SR-AT-03: Caller Context at Scalarization Point

See SR-ARG-04. Caller row and column must be available when `@` evaluates.

### SR-AT-04: Scalarization Trace-Distinguishable

Scalarization caused by `@` must be trace-distinguishable from ordinary dereference and ordinary spill publication.

**Why:** Replay and differential testing need to know whether a scalar result came from `@` scalarization or from a naturally scalar expression.

**Source:** HANDOFF_W014 §3.

---

## 5. Return Surface Seam

### SR-RET-01: Ordinary Value Returns

OxFunc returns `EvalValue` — one of: Number, Text, Logical, Error, Array, Reference, Lambda.

OxFml must handle all of these. Most are straightforward for worksheet publication.

### SR-RET-02: Value + Presentation Hint

Some functions return a value with a format/presentation hint (e.g., DOLLAR returns a number with currency format hint). OxFml/publication layer must accept an optional presentation hint alongside the value.

**Source:** W048 scope.

### SR-RET-03: Lambda Values Are First-Class But Not Cell-Publishable

Lambda values exist as semantic values (can be function arguments, function results, LET bindings, Defined Name values). But they do NOT display in worksheet cells — a bare `=LAMBDA(x, x+1)` publishes as `#CALC!`.

**Why:** If OxFml special-cases lambda at the return boundary, it may lose lambda values in contexts where they should flow as intermediate values (e.g., LET binding, MAP argument).

**Source:** LET_LAMBDA_PIN_DOWN_RESPONSE §2.3.

---

## 6. Library Context / Catalog Seam

### SR-LIB-01: Per-Entry Catalog Fields

OxFml needs the following per function/operator at parse/bind time:

```
stable_id       — canonical function or operator id (e.g., "FUNC.SUM", "FUNC.OP_IMPLICIT_INTERSECTION")
entry_kind      — built_in_function | built_in_operator | external_registered_function
surface_names   — canonical name, aliases, compatibility names, localized names
arity_shape     — { min, max } for early rejection
profile_refs    — references to OxFunc-owned semantic/admission profiles
static_gates    — feature gates, version gates, compatibility gates, add-in presence
```

**Source:** MINIMUM_STABILIZATION_RESPONSE_V2 Topic B.

### SR-LIB-02: Snapshot Identity

Each catalog snapshot must carry a snapshot id and generation marker so OxFml can detect staleness.

**Source:** MINIMUM_STABILIZATION_RESPONSE_V2 Topic B, W049.

### SR-LIB-03: Current Artifact

The current first-freeze artifact is `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`. OxFml has accepted this as a working rule for one round.

**Source:** W044, W049, IN_PROGRESS_FEATURE_WORKLIST IP-10.

---

## 7. Availability / Gating Seam

### SR-AVAIL-01: Parse/Bind Stage Classification

At parse/bind, OxFml should classify function availability as:
- `unknown_name` — not in catalog at all
- `catalog_present` — known, admitted
- `static_feature_gate` — known but feature-gated (e.g., COPILOT without Copilot)
- `static_compat_gate` — known but compatibility-gated (e.g., dynamic-array function in compat mode)

### SR-AVAIL-02: Runtime Stage Classification

At runtime, additional failure modes:
- `runtime_capability_denied` — function known but host capability unavailable
- `provider_unavailable` — external provider not reachable

### SR-AVAIL-03: Post-Dispatch Classification

After successful dispatch:
- `provider_failure` — provider was reachable but returned an error

**Why:** Replay and explain artifacts need to distinguish "not known here" from "known but gated" from "known, dispatched, but provider failed." Collapsing these into generic `#NAME?` or `#VALUE!` loses diagnostic information.

**Source:** MINIMUM_STABILIZATION_RESPONSE_V2 Topic C, NEXT_ROUND_STABILIZATION Topic C.

---

## 8. Open Questions for OxFml

These are concrete decisions OxFunc is waiting on. Each blocks downstream work.

### Q1: Who evaluates `@`?

Will `@` remain an OxFunc-evaluated operator (OxFml passes the operand with provenance, OxFunc applies scalarization rules), or does OxFml want to evaluate `@` upstream and pass the scalarized result?

**If OxFml evaluates upstream:** OxFml must still produce trace/provenance showing the scalarization event, operand class, and caller anchor used. OxFunc needs this for replay/explain.

**If OxFunc evaluates:** OxFml must preserve operand provenance per SR-AT-01 through SR-AT-04.

**Source:** HANDOFF_W014 §3, NOTES_FOR_OXFML.md §6.

### Q2: What prepared-operand vocabulary will OxFml implement?

OxFunc has proposed a minimum vocabulary (SR-ARG-01 through SR-ARG-04). What shape will OxFml actually provide? Candidates:
- Enum-based source classification (DirectScalar, ArrayLikeValue, ReferenceNode, Omitted)
- Orthogonal fields (value_view + structure_class + source_class + reference_identity)
- Something else

**Source:** NOTES_FOR_OXFML.md §5.

### Q3: When will OxFml produce `LambdaValue` carriers?

OxFunc has defined the carrier shape (SR-CALL-01). OxFml must implement helper formation/binding that creates these carriers when evaluating LET/LAMBDA/Defined Name callables.

**Blocked downstream:** All 9 W038 functions (LAMBDA, LET, MAP, REDUCE, SCAN, BYROW, BYCOL, MAKEARRAY, ISOMITTED) plus GROUPBY, PIVOTBY from W051.

### Q4: Will OxFml provide a `CallableInvoker` implementation?

OxFunc's `CallableInvoker` trait (SR-CALL-02) is the typed invocation boundary. OxFml must provide a concrete implementation that can:
1. Resolve a `callable_token` to its semantic meaning
2. Evaluate the callable body with the provided arguments
3. Return a prepared result

**Alternative:** If OxFml prefers a different invocation pattern (e.g., callback-based, or OxFml handles invocation internally and only passes results), that needs to be agreed.

---

## 9. Current OxFunc Implementation Ready for Integration

For reference, OxFunc already has:

| Component | Location | Status |
|-----------|----------|--------|
| `LambdaValue` type | `crates/oxfunc_core/src/value.rs` | Implemented |
| `CallableInvoker` trait | `crates/oxfunc_core/src/functions/callable_helpers.rs` | Implemented |
| `RejectingCallableInvoker` | `crates/oxfunc_core/src/functions/surface_dispatch.rs` | Default fallback |
| Stage1 expression interpreter | `crates/oxfunc_core/src/functions/callable_stage1_prepared.rs` | LET, LAMBDA, ISOMITTED |
| MAP/REDUCE/SCAN/BYROW/BYCOL/MAKEARRAY | `crates/oxfunc_core/src/functions/callable_helpers.rs` | Fully implemented + tested |
| `@` operator | `crates/oxfunc_core/src/functions/op_implicit_intersection.rs` | Fully implemented + tested |
| `@` Lean model | `formal/lean/OxFunc/Functions/ImplicitIntersection.lean` | 6 theorems proven |
| `@` empirical replay | `.tmp/w14-implicit-intersection-results.csv` | 8/8 scenarios passing |
| Library context snapshot | `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` | First freeze published |
| `CallerContext` / `ReferenceResolver` | `crates/oxfunc_core/src/resolver.rs` | Available in evaluation path |

All of this is ready for OxFml to connect to. The gap is on the OxFml side of the seam.

---

## 10. Required Artifact: OxFml Evaluation Adapter for OxFunc Seam Validation

### 10.1 Why This Artifact Exists

OxFunc has 1042 unit tests, but every one uses mock resolvers, fixed random providers, and test-only reference fixtures. None of them exercise OxFml's real preparation pipeline. This means:

- OxFunc cannot verify that OxFml's argument preparation preserves the distinctions SR-ARG-01 through SR-ARG-04 require.
- OxFunc cannot verify that callable values (SR-CALL-01) are formed correctly by OxFml's binder.
- OxFunc cannot verify that `@` receives operands with intact provenance (SR-AT-02).
- Both sides can pass their own tests while the seam is broken.

The evaluation adapter is the artifact that closes this gap. OxFml builds it using its real parser, binder, and preparation logic. OxFunc drives it with structured test scenarios that exercise every seam requirement. If all scenarios pass, the seam works. If any fail, the failure points directly at which requirement is violated.

**Owner:** OxFml
**Consumer:** OxFunc (for validation), both sides (for integration regression)

### 10.2 Interface Shape

The adapter must accept a structured evaluation request and return a structured result. OxFunc does not prescribe the implementation mechanism (Rust crate, CLI binary, JSON protocol, or in-process API), but the semantic contract is:

**Input: Evaluation Request**

```
EvaluationRequest {
  // The formula to evaluate, as a string that OxFml's real parser consumes.
  formula: String,

  // The cell address where this formula lives (provides caller context).
  caller_cell: String,              // e.g. "B5", "Sheet1!C3"

  // A fixture of cell values that the formula may reference.
  // OxFml must resolve references against this fixture using its real
  // reference resolution pipeline.
  cell_fixture: Map<String, FixtureValue>,

  // Optional: explicit random seed for volatile functions (RAND, RANDBETWEEN, RANDARRAY).
  random_seed: Option<f64>,

  // Optional: explicit "now" serial for time functions (NOW, TODAY).
  now_serial: Option<f64>,
}

FixtureValue = Number(f64)
             | Text(String)
             | Logical(bool)
             | Error(String)        // e.g. "#N/A", "#VALUE!"
             | Empty
             | Formula(String)      // A cell that itself contains a formula (for spill, FORMULATEXT, etc.)
```

**Output: Evaluation Result**

```
EvaluationResult {
  // The computed value, in OxFunc's EvalValue vocabulary.
  value: ResultValue,

  // The worksheet error code if evaluation failed.
  error: Option<String>,           // e.g. "#VALUE!", "#NUM!", "#CALC!"

  // Trace metadata (optional but recommended).
  trace: Option<EvaluationTrace>,
}

ResultValue = Number(f64)
            | Text(String)
            | Logical(bool)
            | Error(String)
            | Array { rows: usize, cols: usize, cells: Vec<ResultValue> }
            | Lambda(String)       // callable_token for inspection

EvaluationTrace {
  // Which OxFunc function_id was dispatched.
  function_id: String,

  // What ArgPreparationProfile was used.
  arg_preparation_profile: String,  // "ValuesOnlyPreAdapter" or "RefsVisibleInAdapter"

  // Per-argument: the provenance class OxFml assigned.
  arg_provenance: Vec<ArgProvenance>,

  // Whether caller context was provided.
  caller_context_provided: bool,

  // Whether a CallableInvoker was provided (not the rejecting default).
  callable_invoker_provided: bool,
}

ArgProvenance {
  source_class: String,  // "DirectScalar", "ArrayLikeValue", "ReferenceNode", "Omitted", etc.
  reference_identity: Option<String>,  // e.g. "A1:A10" if reference was preserved
}
```

### 10.3 What OxFml Must Exercise Internally

The adapter is not a passthrough. It must use OxFml's real pipeline:

1. **Real parser:** The formula string must go through OxFml's actual parser (the same one that would parse worksheet formulas).
2. **Real binder:** Name resolution, function lookup against the OxFunc library-context snapshot, arity checking.
3. **Real argument preparation:** Classify each argument according to the function's declared `ArgPreparationProfile`. For `ValuesOnlyPreAdapter` functions, dereference references. For `RefsVisibleInAdapter` functions, preserve reference identity.
4. **Real callable formation:** For formulas containing LAMBDA/LET, produce actual `LambdaValue` carriers with lexical capture.
5. **Real `CallableInvoker`:** For higher-order functions (MAP, REDUCE, etc.), provide an invoker that can evaluate callable bodies.
6. **Real reference resolution:** Resolve A1-style references against the cell fixture, including multi-cell ranges and whole-row/column references.
7. **Real caller context:** Derive CallerContext from the `caller_cell` field.

If any of these steps is stubbed or mocked, the adapter does not fulfill its purpose. The entire point is that OxFml's real logic feeds into OxFunc's real dispatch.

### 10.4 Seam Validation Scenarios

Each scenario maps to one or more SR-* requirements. OxFunc will drive all of these through the adapter. Every scenario must pass for the seam to be considered validated.

#### Category A: Argument Preparation (SR-ARG-01 through SR-ARG-04)

| ID | Formula | Caller | Fixture | Expected | Validates |
|----|---------|--------|---------|----------|-----------|
| A01 | `=SUM(1, 2, 3)` | A1 | — | 6 | Basic scalar dispatch |
| A02 | `=SUM("2", TRUE)` | A1 | — | 3 | SR-ARG-01: direct scalar coercion (text "2" coerces to 2, TRUE coerces to 1) |
| A03 | `=SUM({"2", TRUE})` | A1 | — | 0 | SR-ARG-01: array-scan coercion (text and logical ignored in range scan) |
| A04 | `=SUM(A1:A3)` | B1 | A1=10, A2=20, A3=30 | 60 | Reference resolution through fixture |
| A05 | `=VLOOKUP(2, A1:B3, 2, )` | C1 | A1=1,B1=10,A2=2,B2=20,A3=3,B3=30 | 20 | SR-ARG-02: omitted 4th arg defaults to TRUE (approximate match) |
| A06 | `=VLOOKUP(2, A1:B3, 2, FALSE)` | C1 | (same) | 20 | SR-ARG-02: explicit FALSE = exact match |
| A07 | `=ROW(A3:A7)` | B1 | — | {3;4;5;6;7} | SR-ARG-03: reference identity preserved |
| A08 | `=ROWS(A1:A5)` | B1 | — | 5 | SR-ARG-03: reference identity preserved (count) |
| A09 | `=ROW()` | B5 | — | 5 | SR-ARG-04: caller context used |
| A10 | `=COLUMN()` | C3 | — | 3 | SR-ARG-04: caller context used |

#### Category B: Implicit Intersection / `@` (SR-AT-01 through SR-AT-04)

| ID | Formula | Caller | Fixture | Expected | Validates |
|----|---------|--------|---------|----------|-----------|
| B01 | `=@A1:A3` | B2 | A1=10, A2=20, A3=30 | 20 | SR-AT-01 + SR-AT-02: explicit @ preserved, single-column ref selects caller row |
| B02 | `=@A1:C1` | B2 | A1=10, B1=20, C1=30 | 20 | SR-AT-02: single-row ref selects caller column |
| B03 | `=@{10,20;30,40}` | A1 | — | 10 | SR-AT-02: array payload selects top-left |
| B04 | `=@SEQUENCE(3)` | A1 | — | 1 | SR-AT-01: explicit @ scalarizes dynamic-array producer |
| B05 | `=SEQUENCE(3)` | A1 | — | {1;2;3} | Contrast: without @, SEQUENCE spills |
| B06 | `=@A1:B2` | C3 | A1=10,B1=20,A2=30,B2=40 | #VALUE! | SR-AT-02: 2-D reference yields #VALUE! on current baseline |
| B07 | `=@A1` | B1 | A1=42 | 42 | SR-AT-02: scalar operand passthrough |

#### Category C: Callable Values / LAMBDA / LET (SR-CALL-01 through SR-CALL-05)

| ID | Formula | Caller | Fixture | Expected | Validates |
|----|---------|--------|---------|----------|-----------|
| C01 | `=LET(x, 10, x+1)` | A1 | — | 11 | SR-CALL-01: basic LET binding |
| C02 | `=LET(x, 10, y, x*2, y+1)` | A1 | — | 21 | SR-CALL-01: sequential LET bindings |
| C03 | `=LAMBDA(x, x+1)(10)` | A1 | — | 11 | SR-CALL-01 + SR-CALL-02: immediate LAMBDA invocation |
| C04 | `=LET(f, LAMBDA(x, x*2), f(5))` | A1 | — | 10 | SR-CALL-01 + SR-CALL-03: LET-bound LAMBDA with lexical scope |
| C05 | `=LET(x, 1, f, LAMBDA(y, x+y), LET(x, 99, f(10)))` | A1 | — | 11 | SR-CALL-03: lexical capture survives shadowing (NOT 109) |
| C06 | `=MAP({1,2,3}, LAMBDA(x, x*10))` | A1 | — | {10,20,30} | SR-CALL-02: callable invocation through higher-order function |
| C07 | `=REDUCE(0, {1,2,3}, LAMBDA(a,b, a+b))` | A1 | — | 6 | SR-CALL-02: accumulator-style callable invocation |
| C08 | `=SCAN(0, {1,2,3}, LAMBDA(a,b, a+b))` | A1 | — | {1,3,6} | SR-CALL-02: intermediate-accumulator callable invocation |
| C09 | `=BYROW({1,2;3,4}, LAMBDA(r, SUM(r)))` | A1 | — | {3;7} | SR-CALL-02: row-wise callable invocation |
| C10 | `=BYCOL({1,2;3,4}, LAMBDA(c, SUM(c)))` | A1 | — | {4,6} | SR-CALL-02: column-wise callable invocation |
| C11 | `=MAKEARRAY(2, 3, LAMBDA(r,c, r*10+c))` | A1 | — | {11,12,13;21,22,23} | SR-CALL-02: generated-array callable invocation |
| C12 | `=LAMBDA(x, x+1)` | A1 | — | #CALC! | SR-RET-03: uninvoked LAMBDA publishes as #CALC! |
| C13 | `=LAMBDA(x, x+1)(1,2)` | A1 | — | #VALUE! | SR-CALL-04: over-application yields #VALUE! |
| C14 | `=LET(x, 1, x, 2, x)` | A1 | — | (parse error) | SR-CALL-05: duplicate LET names rejected |

#### Category D: Return Surface (SR-RET-01 through SR-RET-03)

| ID | Formula | Caller | Fixture | Expected | Validates |
|----|---------|--------|---------|----------|-----------|
| D01 | `=PI()` | A1 | — | 3.14159... | SR-RET-01: number return |
| D02 | `=CONCAT("a","b")` | A1 | — | "ab" | SR-RET-01: text return |
| D03 | `=TRUE` | A1 | — | TRUE | SR-RET-01: logical return |
| D04 | `=1/0` | A1 | — | #DIV/0! | SR-RET-01: error return |
| D05 | `=SEQUENCE(2,2)` | A1 | — | {1,2;3,4} | SR-RET-01: array return |
| D06 | `=VALUETOTEXT(42)` | A1 | — | "42" | SR-RET-01: function producing text from number |

#### Category E: Volatile / Provider Functions

| ID | Formula | Caller | Fixture | Random Seed | Expected | Validates |
|----|---------|--------|---------|-------------|----------|-----------|
| E01 | `=RAND()` | A1 | — | 0.5 | 0.5 | Random provider injection |
| E02 | `=RANDBETWEEN(1, 10)` | A1 | — | 0.5 | 6 | Random provider + coercion |
| E03 | `=NOW()` | A1 | — | — (now_serial=45000.5) | 45000.5 | Time provider injection |

#### Category F: Cross-Seam Stress Tests

| ID | Formula | Caller | Fixture | Expected | Validates |
|----|---------|--------|---------|----------|-----------|
| F01 | `=SUM(A1:A3) + ROW()` | B5 | A1=10,A2=20,A3=30 | 65 | Combined: reference resolution + caller context |
| F02 | `=LET(data, A1:A3, SUM(data))` | B1 | A1=10,A2=20,A3=30 | 60 | LET binding with reference argument |
| F03 | `=MAP(A1:A3, LAMBDA(x, x*2))` | B1 | A1=10,A2=20,A3=30 | {20,40,60} | Callable invocation with fixture references |
| F04 | `=@OFFSET(A1,1,0,3,1)` | B2 | A1=10,A2=20,A3=30,A4=40 | 20 | @ over reference-returning function + caller context |
| F05 | `=ROWS(A1:A5) + COLUMNS(A1:C1)` | D1 | — | 8 | Multiple ref-visible functions in one formula |

### 10.5 Acceptance Criteria

The adapter is considered complete when:

1. All scenarios in categories A through F pass.
2. The adapter uses OxFml's real parser, binder, and preparation pipeline — not OxFunc mocks.
3. The `EvaluationTrace` output confirms correct `arg_preparation_profile` and `arg_provenance` for each scenario.
4. Scenario C05 (lexical capture survives shadowing) returns 11, not 109.
5. Scenarios A02 vs A03 (direct scalar vs array coercion) return different results (3 vs 0).
6. Scenarios B01–B06 (implicit intersection) return correct results with correct operand provenance in the trace.
7. Category C scenarios exercise a real `CallableInvoker`, not the `RejectingCallableInvoker`.

### 10.6 How OxFunc Will Use This Artifact

1. **Integration regression suite:** OxFunc will maintain the scenario table as a versioned test fixture. On each OxFml or OxFunc change, all scenarios are re-run through the adapter. Any regression is a seam violation.
2. **Seam requirement validation:** Each scenario maps to SR-* requirements. If a scenario fails, the trace output identifies which requirement is violated.
3. **New function onboarding:** When OxFunc adds a new function, new scenarios are added to the table and run through the adapter to validate seam compliance before claiming integration.
4. **Evidence for completion claims:** The adapter's pass/fail results become part of the evidence trail for function-phase-complete and workset closure claims.
