# W098 Unified Value Model — CalcValue (core + optional rich) And Callable As A Rich Value

Status: `planned`

> Names (`CoreValue` / `RichValue` / `CalcValue` / `CallableValue` / `OpaqueCallable`) are
> design-of-record names for W098/W099.
>
> Note on numbering: `W2`–`W5` below are the **value-model workstream** labels (continuing
> from the already-landed `W1` OxFml compiled-body cache). They are distinct from the OxFunc
> workset id `W098`. The workstreams span four repos (OxFunc/OxFml/OxCalc/DnaTreeCalc); this
> OxFunc packet is the authoritative design-of-record because OxFunc owns the value type.
> OxCalc W060 is an added reference-system companion lane that sharpens the reference payload
> and FEC provider endpoint without changing the core W2-W5 callable/value workstream labels.

## 1. Purpose

Own the design-of-record for replacing OxFunc's evaluation value type `EvalValue` with a
single uniform value, **`CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }`** —
"core + optional rich" — and for representing a **callable** as one of the `RichValue` types
carried by an opaque, refcounted handle. This is the foundation that lets a TreeCalc node
hold a `=LAMBDA(...)` and be invoked by name from other nodes (node-as-function), while
keeping the callable opaque to OxFunc and owned by OxFml.

W098 also records the CalcValue reference endpoint: references remain `CoreValue::Reference`,
but the payload is a typed host/profile identity operated on through a FEC
`ReferenceSystemProvider`, not through stringly `HOST_REF_*` identities or the legacy
resolver/text-resolver split.

This packet captures the full design and the cross-repo workstream plan. It does **not**
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
6. Reference-system companion lane: `../OxCalc/docs/worksets/W060_CALC_TIME_REFERENCE_REPRESENTATION_AND_HOST_REFERENCE_SYSTEM.md`,
   which identifies `CoreValue::Reference` as the right value lane while replacing
   `HOST_REF_*` runtime identity with typed host/profile reference identity and a FEC reference
   system.
7. Design dialogue: session `64923573-2a4e-4346-b8cc-d3f88d011f45` (the `core + optional rich`
   shape; callable as one of the RichValue types; Rc-handle lifetime; no persistence).

## 4. The Value Model (design of record)

### 4.1 The shape — core + optional rich (a struct, not a union)

```rust
// Core calculus value — XLOPER12-like; the `.core` every CalcValue carries.
enum CoreValue { Number(f64), Text(ExcelText), Logical(bool), Error(WorksheetErrorCode),
                 Empty, Missing, Array(CalcArray), Reference(ReferenceLike) }

// Extensible rich layer — Callable is ONE of the RichValue types, a peer of the existing
// structured/linked-data rich. The current `RichValue` struct
// { value_type: RichValueType, fallback: RichValueData, kvps } — string-keyed,
// runtime-extensible — becomes the structured/object payload inside the broader `RichValue`
// enum.
enum RichValue {
    Object(RichObjectValue),           // existing kvp/linked-data rich (image, entity, ...)
    Callable(CallableValue),           // the new callable type — one of the rich types
    Presentation(PresentationValue),   // display/formatting hint attached to a core value
    ErrorMetadata(ErrorMetadataValue), // error-surface metadata attached to an error core
}

// THE uniform value: returned from and passed to OxFunc as args; stored by OxCalc on a node.
struct CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }

// Universal array carrier. Representation-level arrays hold CalcValues. Later policy may
// restrict particular contexts, but the system value type must not depend on legacy array cells.
struct CalcArray { shape: ArrayShape, cells: Vec<CalcValue> }

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

**Core projection rule:** every `CalcValue` has a canonical ordinary value projection in
`.core`. `rich` never replaces `core`; it only augments it. Consumers that are not rich-aware,
or whose metadata does not admit the specific rich type, must be able to degrade through
`.core` with deterministic Excel-compatible behavior. Rich-aware consumers may inspect
`.rich`, but the value remains a `CalcValue { core, rich }` pair rather than a rich payload
standing alone.

The model is intentionally two-tier, not one flat extended enum. `CoreValue` captures the
traditional Excel-compatible value gamut shared by C API/XLOPER12-style interop, COM/VBA
automation and UDF exchange, and ordinary worksheet formula values. `RichValue` carries the
modern and DNA Calc-specific semantic payloads that exceed that core projection. Every
rich value still has a coherent `.core` projection for compatibility, coercion, publication
fallback, display fallback, and degradation.

### 4.1A Reference payload and FEC reference-system target

References remain `CoreValue::Reference`, not rich values. The W098 endpoint adopts the
OxCalc `W060_CALC_TIME_REFERENCE_REPRESENTATION_AND_HOST_REFERENCE_SYSTEM.md` direction:
the reference payload is a typed host/profile identity and all active reference behavior goes
through a FEC-hosted reference-system provider. The current
`ReferenceLike { kind: ReferenceKind, target: String }` scaffold is therefore a textual /
compatibility payload only, not the final reference model.

Target shape, names provisional:

```rust
struct ReferenceLike {
    system: ReferenceSystemId,
    identity: ReferenceIdentity,
    display: Option<ReferenceDisplay>,
}

struct ReferenceSystemId(String); // e.g. "excel.grid.v1", "dna.treecalc.v1"

enum ReferenceIdentity {
    Textual(TextualReferenceIdentity),
    Opaque(ReferenceHandle),
    Composite(CompositeReferenceIdentity),
}

struct TextualReferenceIdentity {
    kind: TextualReferenceKind,
    text: ExcelText,
}

struct ReferenceHandle {
    id: ReferenceHandleId, // host-owned opaque identity; not parsed by OxFunc
}

struct ReferenceHandleId { /* opaque bytes / integer / interned host id */ }

struct CompositeReferenceIdentity {
    operation: CompositeReferenceOperation,
    members: Vec<ReferenceLike>,
}

struct ReferenceDisplay {
    text: ExcelText, // diagnostics/display only; never functional identity
}
```

Ownership split:

1. `ReferenceLike`, `ReferenceSystemId`, `ReferenceIdentity`, and display-only metadata belong
   with the value model because they are passive `CalcValue` payloads.
2. `ReferenceSystemProvider` belongs with the OxFunc function-call/FEC layer because it is an
   execution capability, not value data.
3. Host/profile concrete implementations belong outside OxFunc. OxCalc implements the current
   TreeCalc/reference-profile behavior; OxFml owns syntax and binding callbacks where text must
   be parsed or rebound.

The current `ReferenceKind` vocabulary may survive only inside
`TextualReferenceIdentity` / compatibility constructors, or as request/fact enums where a
textual reference system needs it. OxFunc kernels must not rely on `display.text` or on the
concrete storage of `ReferenceHandleId` as semantic identity. Opaque handles are correlation /
lookup identities owned by the active reference system, not reference text and not durable
workbook syntax.

The FEC bundle should contain one reference-system provider that replaces and subsumes the
current `ReferenceResolver` and `ReferenceTextResolver` split. Target shape, names provisional:

```rust
trait ReferenceSystemProvider {
    fn capabilities(&self) -> ReferenceSystemCapabilities;

    fn describe_reference(
        &self,
        reference: &ReferenceLike,
        request: ReferenceDescribeRequest,
    ) -> Result<ReferenceDescription, ReferenceSystemError>;

    fn dereference(
        &self,
        reference: &ReferenceLike,
        request: ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceSystemError>;

    fn enumerate_values(
        &self,
        reference: &ReferenceLike,
        request: ReferenceEnumerationRequest,
    ) -> Result<ReferenceEnumeration, ReferenceSystemError>;

    fn query_facts(
        &self,
        reference: &ReferenceLike,
        request: ReferenceFactRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError>;

    fn resolve_text(
        &self,
        request: ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError>;

    fn transform(
        &self,
        reference: &ReferenceLike,
        request: ReferenceTransformRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError>;
}
```

The intended operation families are:

1. describe for diagnostics, trace, replay, and display;
2. dereference to a non-reference top-level `CalcValue` for value-only semantics;
3. enumerate sparse/lazy reference values with shape, defined positions, blank/empty
   distinction, ordering, duplicate semantics where applicable, and reader identity;
4. query reference facts such as area count, extent, anchor/address, caller-sensitive address,
   and shape;
5. resolve text in the current execution context for `INDIRECT`-style semantics, with parsing
   and binding owned by OxFml/OxCalc rather than OxFunc;
6. transform/compose references for `OFFSET`, reference-form `INDEX`, union, intersection, and
   structural selector application.

`FunctionExecutionContextBundle` should therefore move from separate
`resolver: &R` plus `reference_text_resolver: Option<&dyn ReferenceTextResolver>` fields toward
one `reference_system: &dyn ReferenceSystemProvider` field. Other providers such as locale,
time, random, RTD, registered-external, callable invocation, and host-info may remain separate
FEC capabilities unless later work proves they should also be grouped. The important W098
boundary is that OxFunc asks for reference capabilities; OxCalc implements the active
TreeCalc/grid/host profile mechanics; OxFml owns syntax and binding; and `HOST_REF_*` strings
do not survive as runtime reference identity.

`ReferenceEnumeration` is deliberately named as an abstraction rather than a fixed container:
the first implementation may be a concrete sparse value collection for current tests, but the
API shape must leave room for host-owned sparse/lazy readers. A dereference result is
"non-reference" at its top-level `CalcValue.core`; array elements remain `CalcValue` because
`CalcArray` is universal, and any later profile/function restriction on nested references is a
policy layer rather than a representation limit.

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
`ExtendedValue`'s role (core + rich + presentation superset) is subsumed by `CalcValue`;
current presentation wrappers map to `RichValue::Presentation`. The string-keyed
`RichValueType`/`RichValueData` extensibility is preserved inside `RichValue::Object` —
runtime-unknown rich types still ride there, which is the extensibility the design requires.

### 4.4A Legacy rich/extended mapping to `CalcValue`

There is no known current rich-value usage that cannot map cleanly into `CalcValue`, provided
the migration preserves the core projection rule:

1. every `CalcValue` has exactly one `core`,
2. `rich` is optional metadata/payload attached to that core,
3. ordinary value semantics, fallback display, coercion degradation, and unsupported-kernel
   behavior start from `.core`,
4. rich-aware consumers inspect `rich` only when their metadata admits that rich type.

Current legacy-to-target map:

| Current carrier | Current producers | Target `CalcValue` |
| --- | --- | --- |
| `ExtendedValue::Core(EvalValue::Number/Text/Logical/Error/Array/Reference)` | extended dispatch wrapper for ordinary functions and provider failures | `CalcValue { core: CoreValue::<matching case>, rich: None }` |
| `ExtendedValue::RichValue(Box<RichValue>)` where old `RichValue` is `{ value_type, fallback, kvps }` | `IMAGE` / `_webimage` | `CalcValue { core: rich_object_fallback_to_core(fallback), rich: Some(Rc::new(RichValue::Object(RichObjectValue { value_type, fallback, kvps }))) }` |
| `ExtendedValue::ValueWithPresentation { value, hint }` | `NOW`, `TODAY`, `HYPERLINK` | `CalcValue { core: CoreValue::from(value), rich: Some(Rc::new(RichValue::Presentation(PresentationValue { hint }))) }` |
| `ExtendedValue::ErrorWithMetadata { code, surface }` | no current ordinary function producer found in OxFunc scan; reserved extended-error lane | `CalcValue { core: CoreValue::Error(code), rich: Some(Rc::new(RichValue::ErrorMetadata(ErrorMetadataValue { surface }))) }` |
| `EvalValue::Lambda(lambda)` | current OxFml callable token path and helper tests | `CalcValue { core: CoreValue::Error(#CALC!), rich: Some(Rc::new(RichValue::Callable(CallableValue { arity, summary, handle }))) }` |

Old structured rich payload mapping:

| Current structured rich type | Target type |
| --- | --- |
| `RichValue` struct | `RichValue::Object(RichObjectValue)` |
| `RichValueType` | `RichObjectType` |
| `RichValueData::Number/Text/Logical/Error/EmptyCell` | `RichObjectData::Number/Text/Logical/Error/Empty` |
| `RichValueData::Array(RichArray)` | `RichObjectData::Array(CalcArray)` where each element is a `CalcValue` |
| `RichValueData::RichValue(Box<RichValue>)` | `RichObjectData::Object(Box<RichObjectValue>)` |
| `RichValueKeyValue` | `RichObjectKeyValue` |

Function-specific current mappings:

1. `IMAGE`: old `ExtendedValue::RichValue(_webimage)` becomes a `CalcValue` whose `core` is the
   `_webimage` fallback text and whose `rich` is `RichValue::Object(_webimage)`. Provider errors
   remain `CalcValue { core: Error(...), rich: None }` unless a later extended-error lane adds
   metadata.
2. `HYPERLINK`: old `ValueWithPresentation { value: Text(display), style: Hyperlink }` becomes
   `CalcValue { core: Text(display), rich: Some(Presentation(style=Hyperlink)) }`.
3. `NOW` / `TODAY`: old `ValueWithPresentation { value: Number(serial), number_format:
   DateLike }` becomes `CalcValue { core: Number(serial), rich: Some(Presentation(DateLike)) }`.
4. Callable values: old session-local `EvalValue::Lambda` is not an object rich value; it maps
   to `RichValue::Callable` with `#CALC!` as its core projection.

Migration-only legacy projection:

1. `ExtendedValue -> CalcValue` is lossless for current OxFunc producers.
2. `CalcValue -> ExtendedValue` may exist only as a W099 staging aid while legacy call sites are
   being removed:
   - `rich = None` projects to `ExtendedValue::Core(core_to_eval_value(core))` when the core is
     representable by legacy `EvalValue`,
   - `RichValue::Object` projects to `ExtendedValue::RichValue`,
   - `RichValue::Presentation` projects to `ExtendedValue::ValueWithPresentation`,
   - `RichValue::ErrorMetadata` projects to `ExtendedValue::ErrorWithMetadata`,
   - `RichValue::Callable` has no faithful legacy `ExtendedValue` projection; any staging-only
     fallback is the `#CALC!` core unless the caller is the callable-aware OxFml path.
3. No `CalcValue -> ExtendedValue` projection survives the W099 end state.

Arrays are representation-level arrays of `CalcValue`. Rich values may appear as array elements
because an element is a `CalcValue { core, rich }`. W098 does not impose a representation-level
ban on rich elements, callable elements, empty elements, or nested-array elements while the
legacy value model is being cleared out. Later semantic/admission policy may reject some of
those cases at particular boundaries or in particular functions, but the universal system value
type should be broad enough to carry them.

Each `CalcValue` holds one optional `RichValue`. That is the W098 target shape. Multiple rich
facets on one `CalcValue` are not represented by W098/W099 and are outside the end-state model
for this migration.

### 4.4B Rich-awareness metadata rule

The value model is universal, but rich handling is not implicit. Function metadata must say when
an input position or return value admits rich handling.

Rule:

1. every function receives and returns `CalcValue`,
2. every function can always degrade through `.core`,
3. a function may inspect, preserve, transform, or produce `.rich` only when its metadata marks
   that input or return position as rich-aware,
4. rich-blind inputs must treat rich payloads through the core projection rule unless the
   function's ordinary Excel semantics explicitly reject the core value,
5. rich-producing returns must mark the return surface as rich-aware so downstream consumers know
   to preserve `.rich` rather than projecting to `.core` only.

Current metadata shape:

1. `FunctionMeta` carries broad execution facts: arity, determinism, volatility, host
   interaction, thread safety, `ArgPreparationProfile`, `CoercionLiftProfile`,
   `KernelSignatureClass`, and FEC dependency profiles.
2. `RegistryFunctionMeta` wraps `FunctionMeta` and adds `SemanticKernelMetadata`,
   `ArgAdmissionMetadata`, metadata version keys, and `producer_capability_set_keys`.
3. `ArgAdmissionMetadata` currently has coarse variants:
   - `ExistingArgPreparation { profile }`,
   - `RichArgAccepted { required_capability_set_keys }`,
   - `SparseRangeAccepted { extent_class, cardinality_class }`.
4. `ParameterDescriptor` carries signature display data: name, optional/repeats, and optional
   help text.
5. There is no current per-input rich-awareness flag and no current return rich-awareness flag.
   `IMAGE` exposes producer capability keys, and `ArgAdmissionMetadata::RichArgAccepted` exists
   for rich argument admission, but this is not yet the simple per-input/per-return metadata W098
   needs.

Chosen W098 target shape:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RichValueHandling {
    CoreOnly,      // default: ignore/degrade through `.core`
    RichAware,    // may inspect/preserve/transform `.rich`
}
```

`RichValueUsageMetadata` belongs on `RegistryFunctionMeta`, not directly on `FunctionMeta`.
`FunctionMeta` stays the compact static execution-shape carrier authored by function modules.
Rich usage is registry/admission/export metadata: it is parameter-aligned, versioned, visible in
registry snapshots, and may be generated or overlaid from catalog/signature data.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichValueUsageMetadata {
    pub input_rich_value_handling: Vec<RichValueHandling>, // aligned with signature parameters
    pub repeating_input_rich_value_handling: Option<RichValueHandling>,
    pub return_rich_value_handling: RichValueHandling,
}

pub struct RegistryFunctionMeta {
    // existing fields...
    pub rich_value_usage: RichValueUsageMetadata,
    pub rich_value_usage_version: String,
}
```

`ParameterDescriptor` remains signature display/help metadata. Consumers that want
per-parameter rich flags read the parameter-aligned projection from
`RegistryFunctionMeta.rich_value_usage`. Only add a direct `FunctionMeta` field if a later
runtime-hot path proves it needs a compact constant there.

Initial defaults and examples:

1. default for all ordinary inputs: `CoreOnly`,
2. default for ordinary returns: `CoreOnly`,
3. `IMAGE` return: `RichAware` because it produces `_webimage` rich object payload,
4. `HYPERLINK`, `NOW`, `TODAY` returns: `RichAware` because they produce presentation rich
   payloads,
5. rich-consuming functions mark the relevant input position `RichAware`; only those positions
   may inspect `.rich`,
6. functions that merely tolerate rich inputs by degrading to `.core` stay `CoreOnly`.

Version/export implication:

1. add `rich_value_usage_version` to the registry snapshot identity,
2. export one compact return column, e.g. `return_rich_value_handling`,
3. export compact input columns, e.g. `input_rich_value_handling` as `CoreOnly|RichAware|...`
   aligned to the signature parameter order, and `repeating_input_rich_value_handling` for
   trailing repeat parameters,
4. keep capability-set metadata separate: rich-awareness says the function may use/preserve
   `.rich`; capability keys say which concrete rich object features are required or produced.

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

1. OxFunc value-model refactor (`EvalValue` → `CalcValue`; `CoreValue`; `CalcArray`;
   `RichValue` enum with `Object`, `Callable`, `Presentation`, and `ErrorMetadata`;
   `CallableValue`; `OpaqueCallable`); generalized
   `CallableInvoker`; retired `LambdaValue` (W2).
2. OxFml eval on `CalcValue`; `SPECIAL.LAMBDA` produces the callable carrier; invoker downcasts
   and runs without per-call recompile; full-scope (IF/LET/curried) by construction (W3).
3. OxCalc node value becomes `CalcValue` stored directly (replacing the stringly seam); derived
   display; node-as-function intake replacing the W074 exclusion; captured-ref dependency edges
   (W4).
4. DnaTreeCalc node-as-function producer corpus + at-scale evidence (W5).

Out of scope for W098/W099:

1. Adding more typed `RichValue` variants beyond `Object`, `Callable`, `Presentation`, and
   `ErrorMetadata`. Runtime-unknown rich objects remain represented by
   `RichValue::Object(RichObjectValue)`.
2. Boundary-specific restrictions on callable/rich/nested-array values inside arrays. The system
   representation admits arrays of `CalcValue`; later function/profile policy may reject
   specific contained values where Excel behavior or host constraints require it.
3. Persistence/serialization of the carrier (callables rebuild from formulas on load).
4. Cross-workspace references, raw reference-literal arrays, dynamic-INDIRECT-in-raw-context,
   strict-excel profile (separately deprioritized).

## 6. Workstreams / Initial Epic Lanes

### W2 — OxFunc value-model refactor (the foundation, big-bang)

1. Define `CoreValue`, the `RichValue` **enum** (`Callable` peer to the existing structured
   rich object payload), `CalcValue`, `CalcArray`, `CallableValue`, `OpaqueCallable`, and rich
   metadata variants. Move `EvalValue` variants into `CoreValue` **minus `Lambda`** (→
   `RichValue::Callable`). Fold the existing structured `RichValue` payload into
   `RichValue::Object(RichObjectValue)`, retarget the `RichValueData::RichValue` /
   `ExtendedValue::RichValue` references, and map presentation/error metadata through §4.4A.
2. **Replace `EvalValue` with `CalcValue`** uniformly across OxFunc — returned from and passed
   to every kernel as args. Compiler-guided fix of the ~5k sites: scalar/array kernels read
   `.core` and emit `CalcValue { core, rich: None }` unless the function is rich-aware by
   metadata. Rich-blind functions degrade through `.core` under the core projection rule.
   Mechanical breadth, shallow depth.
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
2. *Reference model and FEC provider* (OxFunc): textual and opaque `ReferenceLike` identities
   carry system/identity/display separately; display is non-functional; compatibility textual
   constructors do not reintroduce universal `.target`; provider-backed dereference and text
   resolution route through `ReferenceSystemProvider`; source scans show no active
   `HOST_REF_*` runtime identity.
3. *Callable round-trip* (OxFml): `SPECIAL.LAMBDA` → `CalcValue` with `rich = Some(Callable)` and
   `#CALC!` core; invoker downcasts the handle and runs; **no recompile on a second invoking
   frame** (W1 cache hit); full-scope (IF/LET/curried) invokes.
4. *Lifetime* (OxFml/OxCalc): a node-stored callable's `Rc::strong_count` drops to 0 when the
   node is cleared/re-pointed; structural-cache sharing = a clone bumps the count.
5. *OxCalc intake* (replaces the W074 exclusion): the §7 anchor — `B = LAMBDA(X, X+A)`,
   `C = B(2)` resolves & computes to 5; edit `A` → `C` recomputes (captured-ref dependency edge);
   the `LET` form matches; a set-valued callee (`@CHILDREN(1)(…)`) still rejects.
6. *Display* (OxCalc): the typed `format_cell_value_for_display` mirrors DnaOneCalc; a callable
   node displays `#CALC!`.
7. *At-scale* (DnaTreeCalc/OxFml): `MAP` over a large array invoking a node-defined lambda —
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

1. **Foundation shape first.** Introduce the final-direction passive value shapes in
   `crates/oxfunc_value_types/src/lib.rs`: `CoreValue`, `CalcValue`, `CalcArray`,
   `RichValue::Object`, `RichValue::Callable`, `RichValue::Presentation`,
   `RichValue::ErrorMetadata`, `CallableValue`, `OpaqueCallable`, and the typed
   `ReferenceLike` payload (`system`, `identity`, optional display). In `oxfunc_core`, add the
   FEC `ReferenceSystemProvider` slot and compatibility adapters for old resolver/text-resolver
   tests before broad migration begins. Preserve temporary helper constructors/conversions only
   as migration aids.
2. **Adapter/preparation layer second.** Move call arguments, prepared frame values, array-cell
   conversions, coercion, and reference resolution onto `CalcValue.core` deliberately. Delete
   `CallArgValue`, `PreparedArgValue`, `EvalArray`, and `ArrayCellValue` as carriers once their
   behavior is ported; delete `ReferenceResolver` / `ReferenceTextResolver` after all provider
   compatibility adapters are gone.
3. **Callable helpers third.** Change `CallableInvoker` and `require_callable` from
   `&LambdaValue` / `EvalValue::Lambda` to `&CallableValue` / `RichValue::Callable`. Keep
   helper-function dispatch details in the test/mock invokers or OxFml invoker, not on
   `OpaqueCallable`.
4. **Rich/extended fold fourth.** Replace `ExtendedValue::RichValue` and presentation wrappers
   with `CalcValue { core, rich: Some(...) }` according to §4.4A.
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
5. Do not encode runtime host references as `HOST_REF_*` strings or as a universal
   `ReferenceLike.target`; host-owned reference identity belongs in the typed
   `ReferenceIdentity` payload and is operated on only through the FEC reference-system provider.
6. Do not start broad call-boundary, dispatch, or reference-sensitive function migration on top
   of the old reference scaffold. The typed reference payload and FEC provider slot are part of
   the first foundation shape, not a late cleanup.

## 8B. End-State Design Decisions

This decision list belongs to W098 because it clarifies the desired end-state value model. W099
consumes these decisions during migration; it should not invent a different final shape.

Resolved design decisions:

1. **Universal value carrier:** `CalcValue` is the one system value type. Function inputs,
   function outputs, prepared frame values, array cells, node values, rich values, and callable
   values are all represented as `CalcValue`.
2. **No parallel prepared-value carrier:** there is no independent `PreparedArgValue` /
   aggregate-provenance value type in the target model. Preparation may compute transient local
   facts, but values in the call stack/frame remain `CalcValue`.
3. **Universal arrays:** `CalcArray` is an array of `CalcValue`. `CoreValue::Missing`,
   `CoreValue::Empty`, nested arrays, callables, and rich values are representable as array
   elements. Any rejection/coercion is boundary/function policy ported from existing behavior,
   not a narrower representation type.
4. **References:** functions needing reference-visible semantics receive
   `CoreValue::Reference(ReferenceLike)` where `ReferenceLike` is the typed host/profile identity
   from §4.1A. Value-only functions may receive post-dereferenced `CalcValue`s according to
   metadata. Direct-array versus reference-derived behavior follows from `CoreValue::Array`
   versus `CoreValue::Reference`, not a separate value carrier. Dereference, sparse/lazy
   enumeration, reference-fact query, text resolution, and transform/composition go through the
   FEC `ReferenceSystemProvider`; OxFunc must not parse opaque reference handles or rely on
   display text as identity.
5. **Rich values:** rich/object/presentation/error-metadata values are represented by
   `CalcValue { core, rich }` using the core projection rule and the §4.4A mapping. Multiple
   rich facets are not supported by the current `CalcValue` shape and are not an open design
   issue for W098/W099.
6. **Callables:** callables are represented as `RichValue::Callable` on `CalcValue` with a
   `#CALC!` core projection. `EvalValue::Lambda` is deleted during migration; no final callable
   bridge or side carrier remains.
7. **Metadata admits rich handling:** rich-aware input and return behavior is declared by
   `RegistryFunctionMeta.rich_value_usage` (§4.4B). Rich-blind functions use `.core`.
8. **No final bridges, shims, or side-cars:** temporary compatibility conversions may exist only
   inside W099 migration commits and must be removed before W099 closure. After the full
   migration, delete the old value carriers (`EvalValue`, `CallArgValue`, `EvalArray`,
   `ArrayCellValue`, `PreparedArgValue`, `ExtendedValue`) rather than preserving aliases.

Execution work still required by W099:

1. put the typed reference payload and FEC `ReferenceSystemProvider` foundation in place,
2. migrate all OxFunc-local APIs and kernels to `CalcValue`,
3. port current boundary/function policy onto `CalcValue` matches,
4. add/update `RegistryFunctionMeta.rich_value_usage` metadata and exports,
5. run the full local and downstream validation matrix,
6. remove all legacy value carriers and old reference providers and prove their absence by
   grep/audit.

## 9. Key files (verified anchors)

1. **OxFunc** `crates/oxfunc_value_types/src/lib.rs` (`EvalValue` ~552, `LambdaValue` ~495,
   `RichValue`/`RichValueData`/`ExtendedValue` ~303–644 — fold/replace; `ReferenceLike` /
   `ReferenceKind` scaffold — replace with typed identity payload),
   `crates/oxfunc_core/src/function_call.rs` (`FunctionExecutionContextBundle` and
   `FunctionExecutionContext` — add `ReferenceSystemProvider` slot),
   `crates/oxfunc_core/src/resolver.rs` (`ReferenceResolver` / `ReferenceTextResolver` — replace
   or adapt during migration),
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
   ~4050, runtime `HOST_REF_*` / sparse-reference binding paths covered by OxCalc W060),
   `src/oxcalc-core/src/consumer.rs` (`published_values` ~88, `OxCalcTreeNodeView`/`value_text`
   ~164/170, node-as-function exclusion ~3247), plus `coordinator.rs`/`repository.rs`
   publication pipeline.
4. **Reference pattern** (DnaOneCalc) `src/dnaonecalc-host/src/adapters/oxfml/{types.rs
   (worksheet_error_literal ~60, FormulaValuePresentation ~213), live_bridge.rs
   (format_eval_value_for_display ~1326)}`.

## 10. Execution Risks

1. **Blast radius:** ~5,000 `EvalValue` mentions across ~250 OxFunc files become `CalcValue`.
   Mechanical breadth, shallow depth (kernels gain `.core`). Big-bang on a coordinated branch;
   compiler + existing suites are the net. No serde/FFI/hashing/structural-callable-equality
   reliance (sweep-confirmed).
2. **Callable identity:** callable equality is `Rc::ptr_eq(handle)` + `arity` (reference
   identity). Migration must remove any implementation that derives callable equality from
   `summary`, arity alone, or structural body identity.
3. **Lifetime is the Rc, not a registry:** the node's `Rc` handle keeps the OxFml binding alive;
   structural-cache sharing = `Rc::clone`; node clear/edit drops it. No token-map GC, no
   per-frame rehydration of a "durable definition".
4. **Cross-frame invocation:** the compiled body behind the handle is zero-based and binds into
   the invoking frame's slots (existing path); the W1 cache ensures no per-call recompile.
5. **`RichValue` enum-vs-object payload:** the existing structured rich payload becomes
   `RichValue::Object(RichObjectValue)`. The W098 target keeps string-keyed extensibility there
   rather than adding typed `Image` / `Entity` / `FormattedNumber` variants in this migration.
6. **Reference-system seam:** the new typed `ReferenceLike` payload and
   `ReferenceSystemProvider` must land before broad migration. Otherwise new `CalcValue` code
   will preserve `ReferenceLike.target` and `HOST_REF_*` assumptions that W060 is explicitly
   trying to remove.
7. **Incremental-recalc lifetime:** confirm the `Rc`-on-node model interacts correctly with
   OxCalc's dirty/invalidation (clean callable nodes keep a live `Rc`; dirtied/edited ones drop
   and re-materialize).
8. **No hidden compatibility residue:** W099 must prove by grep/audit that the old value
   carriers and old reference providers are deleted rather than aliased or retained behind
   compatibility wrappers.

## 11. Companion worksets (downstream — create when each phase starts)

1. OxFml **W077** — eval on `CalcValue`; `SPECIAL.LAMBDA` callable carrier; invoker downcast;
   full-scope-by-construction (this packet's W3).
2. OxCalc **W059** — node `CalcValue` + node-as-function intake; derived display; captured-ref
   dependency edges (this packet's W4).
3. OxCalc **W060** — calc-time reference representation and host reference system:
   `CalcValue::Reference` typed identity plus FEC `ReferenceSystemProvider` replacing
   `HOST_REF_*` runtime identity and the old resolver/text-resolver split.
4. DnaTreeCalc — node-as-function producer corpus + at-scale evidence (this packet's W5).

This packet references or links these downstream lanes; it does not fold their execution
ownership into W098.

## 12. Bead Workset

Per `docs/worksets/README.md`, `.beads/` owns live execution truth; this doc is
planning/provenance. A fresh OxFunc epic owns this packet and links to the kept cross-repo
workstream beads:

1. OxFunc epic (this packet) → owns `W098`.
2. Kept, linked as `related`: `oxf-ahi7` (W2), `fml-oh8.2` (W3, OxFml store),
   `calc-4vs8.73` (W4, OxCalc store), OxCalc W060 reference-system lane, and `dtc-z0i.8`
   (W5, DnaTreeCalc store).

W098 supersedes stale terminology inside those related beads where it conflicts with this
packet, but does not fold, reparent, or close their execution ownership.

## 13. Closure Condition

`W098` is complete for declared scope only when:

1. the value model in §4 is recorded as design-of-record and registered in
   `docs/WORKSET_REGISTER.md` + `docs/worksets/README.md`,
2. the OxFunc epic bead exists, owns the doc, and links the kept W2–W5 workstream beads plus
   the OxCalc W060 reference-system companion lane,
3. the workstream sequencing (W2 → W3 → W4 → W5), the W060 reference-system dependency, and the
   verification/test plan in §8 are captured for execution,
4. the downstream companion worksets (§11) are named as future lanes,
5. no surface claims the refactor has been executed — this packet is design + tracking only.
