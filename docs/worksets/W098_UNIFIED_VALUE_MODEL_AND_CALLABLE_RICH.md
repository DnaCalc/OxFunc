# W098 Unified Value Model — CalcValue (core + optional rich) And Callable As A Rich Value

Status: `planned`

> Names (`CoreValue` / `RichValue` / `CalcValue` / `CallableValue` / `OpaqueCallable`) are
> design-of-record names unless implementation evidence forces a later correction.
>
> Note on numbering: `W2`–`W5` below are the **value-model workstream** labels (continuing
> from the already-landed `W1` OxFml compiled-body cache). They are distinct from the OxFunc
> workset id `W098`. The workstreams span four repos (OxFunc/OxFml/OxCalc/DnaTreeCalc); this
> OxFunc packet is the authoritative design-of-record because OxFunc owns the value type.

## 1. Purpose

Own the design-of-record for replacing OxFunc's evaluation value type `EvalValue` with a
single uniform value, **`CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }`** —
"core + optional rich" — and for representing a **callable** as one of the `RichValue` types
carried by an opaque, refcounted handle. This is the foundation that lets a TreeCalc node
hold a `=LAMBDA(...)` and be invoked by name from other nodes (node-as-function), while
keeping the callable opaque to OxFunc and owned by OxFml.

This packet captures the full design and the four-repo workstream plan. It does **not**
execute the refactor; `.beads/` owns live execution truth (see §12).

## 2. Why This Packet Exists

The original goal — node-as-function — exposed two structural facts:

1. **OxCalc node values are stringly-typed.** The engine hands OxCalc a stringly
   `seam::ValuePayload` (`../OxFml/crates/oxfml_core/src/seam/mod.rs:26`:
   `Number(String)|Text(String)|Logical(bool)|ErrorCode(String)|Blank`), which OxCalc stores
   as `String` and re-parses via `string_to_eval_value`. There is no callable case on this
   seam, and the round-trip loses type.
2. **A callable cannot be stored as `EvalValue::Lambda`.** Today's `EvalValue::Lambda(LambdaValue)`
   (`crates/oxfunc_value_types/src/lib.rs:552`) is a **session-local token** into a per-frame
   `CallableRegistry`. It cannot survive as a node's durable value, and OxFunc cannot hold
   OxFml's `BoundExpr`-bearing binding directly (the dependency is one-way: OxFml → OxFunc).

A long design pass (validated against Excel's rich-cell value model and the OxFunc/OxFml
boundary) produced a cleaner foundation: one uniform `CalcValue` with the callable as an
opaque rich value. See §4.

The durable value-taxonomy rationale lives in
`docs/function-lane/VALUE_UNIVERSE_PRELIM_SPEC.md` §2A. W098 applies that two-tier model to
the concrete refactor and callable-carrier work.

## 3. Provenance

1. Already landed — **W1**: OxFml callable-subsystem consolidation + structural compiled-body
   cache (`fml-oh8.1`), one `resolve_callable` path, zero-based compiled bodies, recompile-free
   repeated invocation. OxFml workset `../OxFml/docs/worksets/W075_compiled_formula_plan_and_hot_loop_optimization.md`.
2. Kept bead workstreams (this packet links them, does not fold or reparent them):
   `oxf-ahi7` (W2), `fml-oh8.2` (W3), `calc-4vs8.73` (W4), `dtc-z0i.8` (W5).
   W098 is the authoritative design record and supersedes any stale `CellValue`, `Arc`,
   `origin_kind`, or `StructuredRichValue` wording still present in related beads until their
   companion worksets are refreshed.
3. Callable lineage (OxFml): `../OxFml/docs/worksets/W020_semantic_catalog_and_callable_value_breadth.md`,
   `W027_callable_value_and_helper_transport_narrowing.md`,
   `W040_higher_order_callable_evidence_and_seam_reopen.md`,
   `W063_callable_capability_review_and_excel_example_matrix.md`,
   `W064_returned_lambda_invocation_and_lambda_valued_binding_followthrough.md`,
   `W065_recursive_callable_safety_and_workbook_visible_behavior.md`.
4. Higher-order callable seam (OxFunc): `W095_REDUCE_LAMBDA_HELPER_HOTLOOP_PERF.md`,
   `docs/handoffs/HO-FN-015_callable_batching_invocation_seam.md`,
   `crates/oxfunc_core/src/functions/callable_helpers.rs` (`CallableInvoker`).
5. Display/formatting reference (DnaOneCalc): `../DnaOneCalc/src/dnaonecalc-host/src/adapters/oxfml/types.rs`
   (`worksheet_error_literal`, `FormulaValuePresentation`),
   `../DnaOneCalc/src/dnaonecalc-host/src/adapters/oxfml/live_bridge.rs` (`format_eval_value_for_display`).
6. Design dialogue: session `64923573-2a4e-4346-b8cc-d3f88d011f45` (the `core + optional rich`
   shape; callable as one of the RichValue types; Rc-handle lifetime; no persistence).

## 4. The Value Model (design of record)

### 4.1 The shape — core + optional rich (a struct, not a union)

```rust
// Core calculus value — XLOPER12-like; the `.core` every CalcValue carries.
enum CoreValue { Number(f64), Text(ExcelText), Logical(bool), Error(WorksheetErrorCode),
                 Empty, Missing, Array(EvalArray), Reference(ReferenceLike) }

// Extensible rich layer — Callable is ONE of the RichValue types, a peer of the existing
// structured/linked-data rich. The current `RichValue` struct
// { value_type: RichValueType, fallback: RichValueData, kvps } — string-keyed,
// runtime-extensible — remains the structured/object payload inside the broader `RichValue`
// enum. Image/extended-error/entity/formatted-number/dynamic-array can split out as future
// typed variants when evidence or implementation pressure justifies it.
enum RichValue {
    Object(RichObjectValue),           // existing kvp/linked-data rich (image, entity, ...)
    Callable(CallableValue),           // the new callable type — one of the rich types
    // future: Image(..), ExtendedError(..), DynamicArrayAnchor(..), …
}

// THE uniform value: returned from and passed to OxFunc as args; stored by OxCalc on a node.
struct CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }

struct CallableValue { arity: CallableArityShape, summary: String, handle: Rc<dyn OpaqueCallable> }

// OxFunc-owned trait; references no OxFml types. OxFml implements it for its binding.
// No Send + Sync: the whole eval stack is single-threaded (Rc/RefCell). Keep it minimal.
trait OpaqueCallable: std::fmt::Debug + 'static {
    fn as_any(&self) -> &dyn std::any::Any;   // OxFml downcasts in the invoker
}
```

`CalcValue` is **the one uniform value type**, returned from and passed to OxFunc as args,
and stored by OxCalc on a node. Kernels read `.core`. `CoreValue` is the inner calculus enum,
not a separate "working type" — the thing passed around is always `CalcValue`. A scalar is
`CalcValue { core: Number(3.0), rich: None }`.

The model is intentionally two-tier, not one flat extended enum. `CoreValue` captures the
traditional Excel-compatible value gamut shared by C API/XLOPER12-style interop, COM/VBA
automation and UDF exchange, and ordinary worksheet formula values. `RichValue` carries the
modern and DNA Calc-specific semantic payloads that exceed that core projection. Every
rich value still has a coherent `.core` projection for compatibility, coercion, publication
fallback, display fallback, and degradation.

### 4.2 The callable as one of the RichValue types

A callable is `RichValue::Callable(CallableValue{ arity, summary, handle })` with
`core: Error(#CALC!)`. It is "just another rich value type" — **no separate invocable field
per node**. Its core fallback being `#CALC!` (the error enum, not a string) is Excel-faithful:
arithmetic on a callable naturally errors via the core, while higher-order kernels and
node-as-function dereference use the rich part. From the OxFunc side it must "look like
something that can be invoked / called (so OxFunc can implement MAP)"; the invoking call is
made from OxFunc but **runs in OxFml**.

### 4.3 The opaque, Rc-managed carrier (lifetime and ownership)

The handle is `Rc<dyn OpaqueCallable>`. The **`Rc` IS the lifetime**: a node's stored
`CalcValue` holds it; the structural compiled-body cache sharing identical callables is
`Rc::clone`; clearing/changing a node (dependency tracking → re-evaluate → replace the value)
drops the `Rc` and frees the binding. **No token map to garbage-collect.** The carrier is
opaque to OxFunc — OxFml `downcast`s its concrete binding only inside the invoker — which is
required because OxFml depends on OxFunc one-way, so the value type cannot hold OxFml's
`BoundExpr`-bearing binding. OxFml may rebind/JIT the binding behind the `Rc` (interior
mutability) without OxFunc/OxCalc noticing.

The carrier owns or points to OxFml-side execution state, not OxFunc state. In the expected W3
shape, the concrete `OxFmlCallableBinding` behind `OpaqueCallable` contains the arity/parameter
shape, a reference to the W1 compiled-body cache entry or compiled body handle, the captured
lexical/value environment needed for returned/curried lambdas, and any host/dependency
provenance OxFml needs to invoke and report captured-reference facts. OxFunc sees only
`CallableValue { arity, summary, handle }`; it never sees body nodes, parameter symbols, closure
slots, or host reference internals.

**No persistence.** The carrier is runtime-scoped; on load, ordinary re-evaluation
re-materializes callables from the formulas. There is no "durable definition" stored for
per-frame rehydration — the Rc-held binding *is* the live implementation, and the W1 cache
dedups the compiled body so a hot `MAP(arr, B)` does not recompile per call.

### 4.4 Existing rich types fold IN, not aside

Today's structured rich lives only in `ExtendedValue`
(`Core(EvalValue)` / `RichValue` / `ValueWithPresentation` / `ErrorWithMetadata`,
`crates/oxfunc_value_types/src/lib.rs:633`) and is produced by
`image_fn`/`hyperlink_fn`/`now_fn`/`today_fn`/`surface_dispatch` (5 files, ~31 sites).
Under the unified model these become `CalcValue` with `rich = Some(RichValue::Object(..))`;
`ExtendedValue`'s role (core + rich + presentation superset) is subsumed by `CalcValue`
(presentation reconciled as a future rich type / side-channel). The string-keyed
`RichValueType`/`RichValueData` extensibility is preserved inside `RichValue::Object` —
runtime-unknown rich types still ride there, which is the extensibility the design requires.

### 4.5 Derive mechanics

`CalcValue` keeps `Debug/Clone/PartialEq` (no `Eq` — `f64`) if its parts do.
`Rc<RichValue>` is `Clone` (cheap refcount bump). `CallableValue` needs a manual `PartialEq`
(compare `arity` + identity via `Rc::ptr_eq` on the handle — callables are reference-like) and
`Debug` (delegate to the carrier). `Empty` replaces ad-hoc blank/nil value states; `Missing`
is reserved for omitted call-argument slots and is not a published/node literal value.

### 4.6 Boundary facts (verified)

OxFml depends on OxFunc one-way (`oxfunc` has `oxfml` only as a dev-dependency). `LAMBDA`/`LET`
are **OxFml special forms** (`SPECIAL.LAMBDA`, `../OxFml/crates/oxfml_core/src/eval/mod.rs:269,1115,2528`),
intercepted before any OxFunc dispatch — OxFunc has no `LAMBDA` kernel. OxFunc's higher-order
kernels (`MAP`/`REDUCE`/`SCAN`/`BYROW`/`BYCOL`/`MAKEARRAY`) read **only** `arity` + the opaque
handle (`crates/oxfunc_core/src/functions/callable_helpers.rs:117`), which they pass straight
to the invoker; the builtin-helper-vs-user distinction lives inside OxFml's invoker (it holds
the concrete carrier). They never touch body/params/closure. So OxFunc needs only
**{arity, opaque handle, summary}**.

### 4.7 Core principle (unchanged)

OxCalc never parses or introspects formulas. OxFml owns all parsing/binding/callable detection
and invocation; OxFunc owns the calculus + higher-order orchestration; the callable handle is
opaque to OxFunc and OxCalc.

### 4.8 Callable factoring for node-as-function

The motivating path is:

```
MyNode: =LAMBDA(x, x + 1)
Other:  =MyNode(12)
```

`MyNode` evaluates to a `CalcValue` whose core projection is `Error(#CALC!)` and whose rich
payload is `RichValue::Callable`. That value can be stored on the node, cloned through OxCalc
publication, and supplied back to OxFml when `Other` resolves `MyNode` in call position.

The factoring is intentionally split:

1. **OxFunc value type:** owns only the portable carrier shape:
   `CallableValue { arity, summary, handle: Rc<dyn OpaqueCallable> }`.
2. **OxFml binding:** owns the callable implementation behind `OpaqueCallable`: parameters,
   compiled-body cache/body handle, closure/value captures, host-reference/provenance facts,
   and invocation machinery.
3. **OxCalc node storage:** owns the latest `CalcValue` as a node value and clones the `Rc`
   through publication/intake. It does not parse formula text to discover callability.
4. **Dependency reporting:** captured-reference edges are reported by OxFml on the evaluation
   candidate/commit surface and consumed by OxCalc invalidation. They are not derived from
   `CalcValue` equality and are not exposed as OxFunc kernel data.

This avoids all rejected alternatives:

1. no `EvalValue::Lambda` session-token escaping into node storage,
2. no OxFunc dependency on OxFml AST/binding types,
3. no separate node-level `invocable` side field,
4. no persistent callable serialization,
5. no OxCalc formula parsing to reconstruct callable meaning.

Compiled/interpreted/cached/shared/JIT status is behind the opaque OxFml binding. W1's
compiled-body cache can share the expensive body object across equivalent lambdas, while each
runtime callable value still has its own `Rc` handle for the closure/provenance identity that
must travel as a value.

## 5. Scope

In scope:

1. OxFunc value-model refactor (`EvalValue` → `CalcValue`; `CoreValue`; `RichValue` enum with
   `Callable`; `Object`; `CallableValue`; `OpaqueCallable`); generalized
   `CallableInvoker`; retired `LambdaValue` (W2).
2. OxFml eval on `CalcValue`; `SPECIAL.LAMBDA` produces the callable carrier; invoker downcasts
   and runs without per-call recompile; full-scope (IF/LET/curried) by construction (W3).
3. OxCalc node value becomes `CalcValue` stored directly (replacing the stringly seam); derived
   display; node-as-function intake replacing the W074 exclusion; captured-ref dependency edges
   (W4).
4. DnaTreeCalc node-as-function producer corpus + at-scale evidence (W5).

Out of scope (deferred, designed-for):

1. Rich-data `RichValue` members beyond the callable (Image, ExtendedError, Entity,
   FormattedNumber, dynamic-array anchor) — the shape hosts them; populating them is later work.
2. Callables nested inside an OxFunc container (array/entity of lambdas) — the only case needing
   the carrier to live *inside* an OxFunc rich value.
3. Persistence/serialization of the carrier (callables rebuild from formulas on load).
4. Cross-workspace references, raw reference-literal arrays, dynamic-INDIRECT-in-raw-context,
   strict-excel profile (separately deprioritized).

## 6. Workstreams / Initial Epic Lanes

### W2 — OxFunc value-model refactor (the foundation, big-bang)

1. Define `CoreValue`, the `RichValue` **enum** (`Callable` peer to the existing structured
   rich object payload), `CalcValue`, `CallableValue`, `OpaqueCallable`. Move `EvalValue`
   variants into `CoreValue` **minus `Lambda`** (→ `RichValue::Callable`). Fold the existing
   structured `RichValue` payload into `RichValue::Object(RichObjectValue)`, retarget the
   `RichValueData::RichValue` / `ExtendedValue::RichValue` references.
2. **Replace `EvalValue` with `CalcValue`** uniformly across OxFunc — returned from and passed
   to every kernel as args. Compiler-guided fix of the ~5k sites: scalar/array kernels read
   `.core` and emit `CalcValue { core, rich: None }`; a non-`None` `rich` that isn't meaningful
   to a scalar kernel errors via the `#CALC!` core. Mechanical breadth, shallow depth.
3. **Generalize `CallableInvoker`** (`callable_helpers.rs:~117`, 3 impls): take `&CallableValue`
   instead of `&LambdaValue`; use `arity` for arg shaping and pass the opaque `handle` straight
   to the (OxFml) invoker. Update `MAP`/`REDUCE`/`SCAN`/`BYROW`/`BYCOL`/`MAKEARRAY`/`GROUPBY`/
   `PIVOTBY` to read the callable from `rich = Some(Callable(..))` (via `require_callable`,
   `~817`). The builtin-vs-user branch stays inside OxFml's invoker — no `origin_kind()` on the
   trait unless a kernel proves it needs one.
4. Retire `LambdaValue` (fields collapse into `CallableValue` + the OxFml-side binding behind
   the handle).

### W3 — OxFml: produce/consume `CalcValue`; callable carrier; invoker

1. Eval in terms of `CalcValue`; scalars are `CalcValue { core, rich: None }`. Replace
   `EvalValue` in `eval/mod.rs` and the result/host/consumer/publication surfaces.
2. **`SPECIAL.LAMBDA` produces a callable.** `evaluate_lambda_call`
   (`eval/mod.rs:~4250`, today building `EvalValue::Lambda` at ~4317) returns
   `CalcValue { core: Error(#CALC!), rich: Some(Callable(CallableValue { handle: Rc::new(<OxFml binding>), arity, summary })) }`.
   The binding (compiled body or W1 compiled-body-cache handle + closure/value captures +
   OxFml provenance needed for invocation and dependency reporting) lives behind the `Rc`; the
   per-frame token `CallableRegistry` indirection is replaced by this `Rc` ownership.
3. **Invoker downcasts and runs.** `OxFmlCallableInvoker::invoke(&CallableValue, args)`
   (`eval/mod.rs:~5826`) does `handle.as_any().downcast_ref::<OxFmlCallableBinding>()` and runs
   the already-compiled, zero-based body in the invoking frame, binding args + closure into that
   frame's slots. No per-call recompile (W1 cache). Name-captures re-resolve live via the host
   callback; value-captures (currying) are baked into the binding. Captured-reference facts are
   reported through the existing OxFml/OxCalc candidate/commit surfaces rather than embedded in
   `CoreValue` or exposed to OxFunc kernels. Reuse `lambda_binding_from_defined_name_binding`
   (`~4802`).
4. **Full scope falls out** — `=IF(c, LAMBDA…, LAMBDA…)`, `=LET(g, LAMBDA…, g)`, and curried
   lambdas flow through as values; the carrier was created at the inner `LAMBDA`.
5. **Cache/JIT-ready without changing the value type** — the binding behind the `Rc` may point
   to shared compiled-body cache entries or later optimized/JIT bodies with interior mutability
   (forward-looking; not required for v1). The `CalcValue` shape does not distinguish compiled,
   cached, shared, or interpreted callables; those are OxFml implementation choices behind
   `OpaqueCallable`.

### W4 — OxCalc: `CalcValue` node values + node-as-function intake

1. **Node value becomes `CalcValue` — the same carrier, stored directly.** Replace the stringly
   path (`published_values: BTreeMap<TreeNodeId, String>` at
   `../OxCalc/src/oxcalc-core/src/consumer.rs:88`; the `seam::ValuePayload` from
   `candidate.value_delta.published_payload`; the `value_payload_to_string`/`string_to_eval_value`
   round-trip at `../OxCalc/src/oxcalc-core/src/treecalc.rs:5448,5423`). A callable node is
   `CalcValue { core: Error(#CALC!), rich: Some(Callable) }` — no extra invocable field. **Derive
   display** from the value via a typed formatter mirroring DnaOneCalc; a callable node displays
   `#CALC!` from the core error enum. `OxCalcTreeNodeView.value_text` (`consumer.rs:170`) becomes
   a projection. Carrier clones through the publication pipeline are cheap `Rc` bumps.
2. **Inter-node re-supply** hands the stored `CalcValue` directly to the engine (no stringly
   round-trip — replaces `runtime_binding_for_reference`'s re-parse at `treecalc.rs:4050`). A
   callable node supplies its `CalcValue`-with-`Callable`; OxFml downcasts the handle and
   invokes. **Replace** `typed_exclusion:node_as_function_w074_pending` (`consumer.rs:3247`).
3. **Dependency edges** from the captured-ref facts OxFml reports; editing a captured node
   re-evaluates callers. These facts travel on the evaluation publication/candidate surface,
   not through `CalcValue` equality or the opaque handle. The callable's `Rc` lifetime ties to
   the node value.
4. **No persistence** — node-stored callables re-materialize from formulas on load.

### W5 — DnaTreeCalc: producer corpus + evidence

Activate `active_node_functions_corpus` (`dtc-z0i.8`): the §7 anchor example; capture
(defining-scope ref) + edit-captured invalidation; callable-calls-callable; full-scope
(IF/LET/curried-returning lambda nodes); at-scale (`MAP` over a large array invoking a
node-defined lambda) confirming no per-call recompile (W1 cache).

## 7. Anchor example (consolidated paths — the two should evaluate near-identically)

```
A: =3                A: =3
B: =LAMBDA(X, X+A)   C: =LET(f, LAMBDA(X, X+A), f(2))   →  5 ; edit A → recompute
C: =B(2)        →  5 ; edit A → recompute
```

The only real difference is *where the callable lives* (node `B`'s value vs the LET-local `f`);
the control thread weaving OxCalc↔OxFml is otherwise the same. Keep these paths consolidated.

## 8. Verification

Existing regression nets to keep green (the safety net for the big-bang):

1. OxFunc (~112 tests): `callable_helpers.rs` unit tests for MAP/REDUCE/SCAN/BYROW/BYCOL/
   MAKEARRAY; `groupby_fn.rs`/`pivotby_fn.rs`; `tests/oxfml_seam_integration.rs`,
   `tests/direct_call_array_input_seams_integration.rs`.
2. OxFml (~55 tests): `evaluator_tests.rs` (LAMBDA/LET/closure/higher-order/recursion/curried),
   `callable_calls_callable_tests.rs`, `callable_portable_result_tests.rs`,
   `higher_order_callable_tests.rs`, the W1 `compiled_body_cache_*` tests.
3. OxCalc: `let_lambda_capturing_sibling_node_resolves_and_invalidates` + ~30
   `published_values`/`value_text` tests (adapt to compare via *derived display* of `CalcValue`).
4. DnaTreeCalc: `active_node_functions_corpus` (currently asserts the
   `node_as_function_w074_pending` exclusion; flips to resolved/computed).

New tests this refactor must add:

1. *Value-model invariants* (OxFunc `oxfunc_value_types`): `CalcValue` core+rich construction; a
   callable's `core == Error(#CALC!)`; `CallableValue` `PartialEq` = `Rc::ptr_eq(handle)` +
   `arity`; `Debug` delegates to the carrier.
2. *Callable round-trip* (OxFml): `SPECIAL.LAMBDA` → `CalcValue` with `rich = Some(Callable)` and
   `#CALC!` core; invoker downcasts the handle and runs; **no recompile on a second invoking
   frame** (W1 cache hit); full-scope (IF/LET/curried) invokes.
3. *Lifetime* (OxFml/OxCalc): a node-stored callable's `Rc::strong_count` drops to 0 when the
   node is cleared/re-pointed; structural-cache sharing = a clone bumps the count.
4. *OxCalc intake* (replaces the W074 exclusion): the §7 anchor — `B = LAMBDA(X, X+A)`,
   `C = B(2)` resolves & computes to 5; edit `A` → `C` recomputes (captured-ref dependency edge);
   the `LET` form matches; a set-valued callee (`@CHILDREN(1)(…)`) still rejects.
5. *Display* (OxCalc): the typed `format_cell_value_for_display` mirrors DnaOneCalc; a callable
   node displays `#CALC!`.
6. *At-scale* (DnaTreeCalc/OxFml): `MAP` over a large array invoking a node-defined lambda —
   assert W1-cache hits (no per-call recompile).

Commands: `cargo test -p oxfunc_value_types -p oxfunc_core`;
`cargo test --manifest-path ../OxFml/crates/oxfml_core/Cargo.toml`;
`cargo test -p oxcalc-core`; DnaTreeCalc host tests.

## 8A. Refactor Readiness Notes

Preflight on `2026-05-31` found the following current-state anchors:

1. `oxfunc_core::value` is a wholesale re-export of `oxfunc_value_types`; the value-crate
   rename lands everywhere immediately.
2. `EvalValue` appears across 252 Rust files under `crates/`; this is a true big-bang
   compiler-guided migration, not a narrow callable-helper patch.
3. Lambda/callable value shapes appear across 53 Rust files. The real production seam is
   concentrated in `crates/oxfunc_core/src/functions/callable_helpers.rs`,
   `group_pivot_common.rs`, `function_call.rs`, and `surface_dispatch.rs`; many remaining hits
   are ordinary unsupported-kind matches or tests.
4. Rich/extended value surfaces are concentrated in 7 Rust files. `image_fn.rs`,
   `hyperlink_fn.rs`, `now_fn.rs`, `today_fn.rs`, and `surface_dispatch.rs` are the meaningful
   non-callable rich/presentation lanes.

Recommended execution order:

1. **Value-crate scaffolding first.** Introduce `CoreValue`, `CalcValue`, `RichValue::Object`,
   `RichValue::Callable`, `CallableValue`, and `OpaqueCallable` in
   `crates/oxfunc_value_types/src/lib.rs`; preserve temporary helper constructors/conversions so
   ordinary scalar kernels can be migrated mechanically.
2. **Adapter/preparation layer second.** Update `CallArgValue`, `PreparedArgValue`, array-cell
   conversions, coercion, and reference resolution to use `CalcValue.core` deliberately. This is
   where `Empty` and `Missing` admission must be enforced.
3. **Callable helpers third.** Change `CallableInvoker` and `require_callable` from
   `&LambdaValue` / `EvalValue::Lambda` to `&CallableValue` / `RichValue::Callable`. Keep
   helper-function dispatch details in the test/mock invokers or OxFml invoker, not on
   `OpaqueCallable`.
4. **Rich/extended fold fourth.** Replace `ExtendedValue::RichValue` and presentation wrappers
   with `CalcValue { core, rich: Some(...) }` or an explicit side-channel where the value model
   should not own presentation.
5. **Mechanical kernel sweep last.** Migrate the broad scalar/array kernel surface from
   `EvalValue::*` to `CalcValue` constructors and `.core` matches. Do this after the central
   constructors and helper accessors exist, otherwise every file will invent its own pattern.

Implementation guardrails:

1. Do not keep a compatibility `type EvalValue = CalcValue` alias as the final state; it hides
   `.core` decisions and defeats the W098 audit.
2. Do not expose OxFml binding/body/cache types through `OpaqueCallable`.
3. Do not derive callable equality from `summary`, arity alone, or structural body identity;
   v1 equality is handle identity plus arity.
4. Do not encode captured-reference dependency edges in `CalcValue`; they belong on the
   OxFml/OxCalc publication/candidate surface.

## 9. Key files (verified anchors)

1. **OxFunc** `crates/oxfunc_value_types/src/lib.rs` (`EvalValue` ~552, `LambdaValue` ~495,
   `RichValue`/`RichValueData`/`ExtendedValue` ~303–644 — fold/replace),
   `crates/oxfunc_core/src/functions/callable_helpers.rs` (`CallableInvoker` ~117,
   `require_callable` ~817, MAP/REDUCE/SCAN/…), `functions/{groupby_fn.rs, pivotby_fn.rs}`
   (token tag-checks at :419/:465 are in *test* invokers — production dispatch is the invoker's
   job). Largest mechanical-churn sites: `functions/surface_dispatch.rs` (~472),
   `functions/xlookup.rs` (~122).
2. **OxFml** `crates/oxfml_core/src/eval/mod.rs`: `SPECIAL_*` (~269), special-form routing
   (~1115, ~2528), `evaluate_lambda_call` (~4250, builds `EvalValue::Lambda` ~4317),
   `CallableRegistry` + W1 compiled-body cache (~930/~1013), `OxFmlCallableInvoker` (~5826),
   `lambda_binding_from_defined_name_binding` (~4802). Result/seam surfaces:
   `EvaluationOutput.oxfunc_value` (~473), `host/mod.rs:1103` (`published_worksheet_value`,
   Lambda→#CALC! at 1105), `interface/mod.rs` (HostFunctionProvider ~243, ReturnedValueSurface
   ~707), `publication/mod.rs`, `consumer/runtime/mod.rs:677`, `oxfunc_adapter/mod.rs`,
   `seam/mod.rs:26` (`ValuePayload` — the stringly path W4 replaces).
3. **OxCalc** `src/oxcalc-core/src/treecalc.rs` (`adapt_oxfml_runtime_candidate` ~4759,
   `value_payload_to_string` ~5448, `string_to_eval_value` ~5423, `runtime_binding_for_reference`
   ~4050), `src/oxcalc-core/src/consumer.rs` (`published_values` ~88, `OxCalcTreeNodeView`/
   `value_text` ~164/170, node-as-function exclusion ~3247), plus `coordinator.rs`/`repository.rs`
   publication pipeline.
4. **Reference pattern** (DnaOneCalc) `src/dnaonecalc-host/src/adapters/oxfml/{types.rs
   (worksheet_error_literal ~60, FormulaValuePresentation ~213), live_bridge.rs
   (format_eval_value_for_display ~1326)}`.

## 10. Risks / open items

1. **Blast radius:** ~5,000 `EvalValue` mentions across ~250 OxFunc files become `CalcValue`.
   Mechanical breadth, shallow depth (kernels gain `.core`). Big-bang on a coordinated branch;
   compiler + existing suites are the net. No serde/FFI/hashing/structural-callable-equality
   reliance (sweep-confirmed).
2. **`PartialEq`/identity for callables:** equality = `Rc::ptr_eq(handle)` + `arity` (reference
   identity); the sweep confirmed no kernel/test relies on structural callable equality.
3. **Lifetime is the Rc, not a registry:** the node's `Rc` handle keeps the OxFml binding alive;
   structural-cache sharing = `Rc::clone`; node clear/edit drops it. No token-map GC, no
   per-frame rehydration of a "durable definition".
4. **Cross-frame invocation:** the compiled body behind the handle is zero-based and binds into
   the invoking frame's slots (existing path); the W1 cache ensures no per-call recompile.
5. **`RichValue` enum-vs-object payload:** the existing structured rich payload becomes
   `RichValue::Object(RichObjectValue)`. Splitting common modern cases into typed peers
   (Image/Entity/FormattedNumber/…) is allowed later when evidence or implementation pressure
   justifies it. The v1 choice keeps string-keyed extensibility intact.
6. **Incremental-recalc lifetime:** confirm the `Rc`-on-node model interacts correctly with
   OxCalc's dirty/invalidation (clean callable nodes keep a live `Rc`; dirtied/edited ones drop
   and re-materialize).
7. **Names** are provisional ("portable_callable" is dropped from the public model — it's just a
   callable; "portable" is an OxFml-internal build detail).

## 11. Companion worksets (downstream — create when each phase starts)

1. OxFml **W077** — eval on `CalcValue`; `SPECIAL.LAMBDA` callable carrier; invoker downcast;
   full-scope-by-construction (this packet's W3).
2. OxCalc **W059** — node `CalcValue` + node-as-function intake; derived display; captured-ref
   dependency edges (this packet's W4).
3. DnaTreeCalc — node-as-function producer corpus + at-scale evidence (this packet's W5).

This packet references these as downstream lanes; it does not pre-create them.

## 12. Bead Workset

Per `docs/worksets/README.md`, `.beads/` owns live execution truth; this doc is
planning/provenance. A fresh OxFunc epic owns this packet and links to the kept cross-repo
workstream beads:

1. OxFunc epic (this packet) → owns `W098`.
2. Kept, linked as `related`: `oxf-ahi7` (W2), `fml-oh8.2` (W3, OxFml store),
   `calc-4vs8.73` (W4, OxCalc store), `dtc-z0i.8` (W5, DnaTreeCalc store).

W098 supersedes stale terminology inside those related beads where it conflicts with this
packet, but does not fold, reparent, or close their execution ownership.

## 13. Closure Condition

`W098` is complete for declared scope only when:

1. the value model in §4 is recorded as design-of-record and registered in
   `docs/WORKSET_REGISTER.md` + `docs/worksets/README.md`,
2. the OxFunc epic bead exists, owns the doc, and links the kept W2–W5 workstream beads,
3. the workstream sequencing (W2 → W3 → W4 → W5) and the verification/test plan in §8 are
   captured for execution,
4. the downstream companion worksets (§11) are named as future lanes,
5. no surface claims the refactor has been executed — this packet is design + tracking only.
