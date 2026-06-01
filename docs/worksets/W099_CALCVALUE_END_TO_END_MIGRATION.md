# W099 CalcValue End-To-End Migration

Status: `planned`

## 1. Purpose

Own the execution plan for the full OxFunc migration to `CalcValue` as the single value type
used throughout the OxFunc function stack, plus the required OxFml callable follow-through so
callables are actually produced and consumed as `CalcValue` / `RichValue::Callable` values rather
than as legacy `EvalValue::Lambda` tokens.

`W098` is the design-of-record for the unified value model. `W099` is the successor execution
workset: it turns that design into a staged, evidence-backed migration plan and then into code
only when the gate criteria below are satisfied.

The terminal target is intentionally strict:

1. function-facing arguments, prepared arguments, array cells, references, scalar results,
   dynamic-array results, callable values, and rich/object values all travel through `CalcValue`,
2. legacy `EvalValue`, `EvalArray`, `ArrayCellValue`, `CallArgValue`, `PreparedArgValue`, and
   `ExtendedValue` names are deleted from the final tree rather than preserved as aliases,
   bridges, side-cars, or compatibility shims,
3. legacy reference providers (`ReferenceResolver`, `ReferenceTextResolver`, direct
   `ReferenceLike.target`, and runtime `HOST_REF_*` identity handling) are deleted from the final
   tree rather than preserved as aliases, bridges, side-cars, or compatibility shims,
4. any temporary compatibility conversion introduced during migration has been removed before
   the terminal gate,
5. no function kernel or public OxFunc value-call path uses a legacy carrier as its native semantic
   value type,
6. OxFml no longer uses `EvalValue::Lambda` as the native callable value representation:
   `SPECIAL.LAMBDA`, defined-name callable resolution, helper callables, higher-order invocation,
   returned/curried lambdas, publication, and re-supply all use `CalcValue` with
   `RichValue::Callable` backed by a real `OxFmlCallableBinding` handle,
7. `CoreValue::Reference` carries the W098/W060 typed reference payload, and FEC exposes a
   `ReferenceSystemProvider` as the single OxFunc-facing reference capability instead of the
   legacy `ReferenceResolver` / `ReferenceTextResolver` split,
8. existing Excel parity evidence remains green, and any behavior change discovered during the
   migration is either proven to improve Excel compliance or promoted through the ordinary bug
   stream.

## 2. Relationship To W098

`W098_UNIFIED_VALUE_MODEL_AND_CALLABLE_RICH.md` defines the target model:

1. `CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }` is the value carrier.
2. Kernels read `.core` deliberately.
3. Callable values move from `EvalValue::Lambda` to `RichValue::Callable`.
4. Structured rich values fold into the `RichValue` layer.
5. OxFml owns the concrete callable binding behind `OpaqueCallable`; W099 must carry the OxFml
   follow-through because a callable value model remains only partial while OxFml still treats
   `EvalValue::Lambda` as the primary callable carrier.
6. References remain `CoreValue::Reference`, but the reference payload moves from the current
   `ReferenceLike { kind, target }` scaffold to the W098/W060 typed host/profile identity:
   `system`, `identity`, and optional display metadata. The FEC bundle gains a
   `ReferenceSystemProvider` that replaces and subsumes the current `ReferenceResolver` and
   `ReferenceTextResolver` split.

`W099` must not reopen that model casually. It may refine the model only when migration evidence
shows a concrete contradiction, missing invariant, or unsafe edge.

## 2A. Batch 0 Scaffold Review

W099 starts with an existing partial scaffold committed as `da0c023`
(`Scaffold CalcValue value model`). Treat this as **Batch 0 scaffold/staging**, not as a
settled migration batch.

Committed Batch 0 scaffold:

1. `crates/oxfunc_value_types/src/lib.rs` introduces `CoreValue`, `CalcValue`, `CalcArray`,
   `RichValue::{Object, Callable, Presentation, ErrorMetadata}`, `CallableValue`, and
   `OpaqueCallable` while retaining the legacy value carriers.
2. `image_fn.rs` and `surface_dispatch.rs` are adjusted for `RichValue::Object(...)`.
3. The scaffold intentionally retains temporary legacy carriers and compatibility conversions so
   later W099 batches can migrate call sites under compiler control.

Review corrections and scope updates already required by W098:

1. `CalcArray` must be a representation-level array of `CalcValue`. It must admit nested arrays,
   missing values, rich values, and callables as representable elements. Boundary/function policy
   may reject them later, but the array carrier must not encode those restrictions.
2. `CallableValue` equality is handle identity plus arity. It must not compare `summary`, body
   text, structural body identity, or arity alone.
3. `EvalValue::Lambda -> CalcValue` fallback conversion is staging-only. The real new-world path
   is OxFml producing or projecting an `EvaluationOutput::calc_value()` with
   `RichValue::Callable` backed by an `OxFmlCallableBinding` handle. A placeholder legacy handle
   in OxFunc is not a valid final callable implementation and must disappear with
   `EvalValue::Lambda`.
4. OxFml is not far enough along to remain a passive downstream dependency. W099 scope therefore
   includes an OxFml callable-completion lane: replace the legacy `EvalValue::Lambda` runtime
   carrier with `CalcValue` / `RichValue::Callable`, keep the existing portable-callable binding
   facts, and prove higher-order invocation still downcasts and runs through the real
   `OxFmlCallableBinding`.
5. The current `ReferenceLike { kind, target }`, `ReferenceResolver`, and `ReferenceTextResolver`
   pieces are staging-era reference plumbing and must be corrected in the first foundation batch.
   They may remain only behind compatibility constructors/adapters while call sites migrate, but
   W099 must not start broad call-boundary, dispatch, or reference-sensitive function migration
   on top of the old `.target` / `ReferenceKind` / split-resolver shape.
6. `ReferenceSystemProvider` is an OxFunc FEC/function-call capability, not value data. Passive
   reference identity types belong with `CalcValue`; host/profile implementations belong outside
   OxFunc, with OxCalc owning the active TreeCalc/reference-profile behavior.

W099 implementation work must continue by turning this review into explicit child beads and a
first real migration-batch plan: preserve the useful scaffold, remove staging bridges as their
call sites migrate, and keep unrelated behavior changes or formatting churn out of semantic
migration commits.

## 2B. Early Foundation Batch Requirement

The first implementation batch after planning is a foundation-shape batch, not a broad value
carrier sweep. It must put the right `CalcValue` and FEC shape in place before thousands of call
sites are touched.

Foundation-shape batch scope:

1. replace the scaffold `ReferenceLike { kind, target }` endpoint with the W098 typed
   reference payload shape: `system`, `identity`, and optional display metadata. A compatibility
   textual constructor may exist, but the native payload must not expose universal `.target`;
2. add the `ReferenceSystemProvider` trait and minimal request/result/error types with the W098
   operation families: describe, dereference, enumerate, query facts, resolve text, and
   transform/compose. Enumeration must be abstract enough to admit sparse/lazy readers even if
   the first implementation uses a concrete sparse collection;
3. update `FunctionExecutionContextBundle` / `FunctionExecutionContext` so new code consumes one
   reference-system provider rather than separate `ReferenceResolver` and
   `ReferenceTextResolver` capabilities;
4. add compatibility adapters from the old resolver/text-resolver tests and call sites into the
   new provider so the repository can migrate incrementally without preserving the old shape as
   the native target;
5. add value-crate and core tests proving textual and opaque references can be represented,
   described/dereferenced through the provider, and never require `HOST_REF_*` or display text as
   runtime identity.

Foundation-shape acceptance gate:

1. no new broad migration batch starts until `ReferenceLike` has the W098 typed identity shape;
2. new FEC-facing code has a `ReferenceSystemProvider` slot available;
3. old resolver/text-resolver APIs are either adapted into that provider or explicitly marked
   compatibility-only with deletion owners;
4. focused tests cover textual identity, opaque identity, provider dereference, provider text
   resolution, and display-as-non-identity;
5. a source scan records the current `HOST_REF_`, `ReferenceLike.target`, `ReferenceKind`,
   `ReferenceResolver`, and `ReferenceTextResolver` surface before broad migration begins.

Non-goals for the first foundation batch:

1. it does not need to implement every host/profile transform perfectly;
2. it does not need to migrate every function kernel to `CalcValue`;
3. it must not treat the compatibility adapters as final architecture.

After this batch, broad W099 migration can rely on the correct direction: `CalcValue` has the
right reference payload, FEC has the right provider slot, and old resolver/text-resolver APIs are
already on the deletion path.

## 3. Planning Round 1: CallArgValue Versus EvalValue

Current code shape:

```rust
pub enum EvalValue {
    Number(f64),
    Text(ExcelText),
    Logical(bool),
    Error(WorksheetErrorCode),
    Array(EvalArray),
    Reference(ReferenceLike),
    Lambda(LambdaValue),
}

pub enum CallArgValue {
    Eval(EvalValue),
    MissingArg,
    EmptyCell,
    Reference(ReferenceLike),
}
```

The difference is not just naming:

1. `EvalValue` is the current evaluated-value carrier. It represents ordinary scalar results,
   arrays, references, errors, and session-local lambda tokens. It is used for both function
   return values and already-evaluated arguments.
2. `CallArgValue` is the current function-call argument wrapper. Its non-`Eval` cases are
   argument-position facts that were not representable in legacy `EvalValue`:
   - `MissingArg` means the argument was syntactically omitted or intentionally absent.
   - `EmptyCell` means the argument position received an empty-cell value.
   - `Reference` preserves reference identity for functions that need reference-visible
     semantics instead of only the materialized value.
   - `Eval(EvalValue)` carries the ordinary evaluated value case.
3. `CallArgValue::Reference` duplicates `EvalValue::Reference` today. That duplication is a
   migration smell: in the unified model the reference-like value is simply
   `CalcValue { core: CoreValue::Reference(..), rich: None }`.
4. `CallArgValue::MissingArg` and `CallArgValue::EmptyCell` currently stand outside `EvalValue`.
   In the `CalcValue` model these are first-class core cases:
   `CoreValue::Missing` and `CoreValue::Empty`.
5. `CallArgValue` is therefore a legacy argument carrier whose remaining distinctions are now
   covered by `CalcValue.core`. It should not survive as a new `CalcArg` / `ArgumentSlot`
   wrapper unless a concrete audited counterexample proves that `CoreValue` is insufficient.

Initial planning conclusion:

`CalcValue` should replace both `EvalValue` and `CallArgValue` at the function-call boundary.
`CalcValue.core` covers every current `CallArgValue` case:

```rust
CallArgValue::Eval(EvalValue::Number(n))      -> CoreValue::Number(n)
CallArgValue::Eval(EvalValue::Text(t))        -> CoreValue::Text(t)
CallArgValue::Eval(EvalValue::Logical(b))     -> CoreValue::Logical(b)
CallArgValue::Eval(EvalValue::Error(e))       -> CoreValue::Error(e)
CallArgValue::Eval(EvalValue::Array(a))       -> CoreValue::Array(...)
CallArgValue::Eval(EvalValue::Reference(r))   -> CoreValue::Reference(r)
CallArgValue::MissingArg                      -> CoreValue::Missing
CallArgValue::EmptyCell                       -> CoreValue::Empty
CallArgValue::Reference(r)                    -> CoreValue::Reference(r)
```

The remaining safety concern is not an argument-wrapper concern. It is a preparation/resolution
concern: functions that distinguish direct arrays from reference-derived arrays must continue to
preserve that distinction by starting from `CoreValue::Array` versus `CoreValue::Reference` and
by recording any needed transient preparation facts outside the value carrier where required.
There is no final `PreparedArgValue` or aggregate-provenance value carrier in the W098 target.

## 4. Migration Strategy

Use compiler-guided breadth, but gate each semantic category before the broad mechanical sweep.

1. Inventory every value-shaped type and classify it as value, argument metadata, array storage,
   rich/presentation carrier, reference carrier, or compatibility shim.
2. Define the final public and crate-internal ABI names before editing thousands of call sites.
3. Execute the early foundation-shape batch from §2B: typed `CoreValue::Reference` payload,
   `ReferenceSystemProvider`, and FEC provider slot with compatibility adapters.
4. Move central constructors, coercion helpers, array iteration, and error constructors to
   `CalcValue` first.
5. Convert argument preparation next, because it decides the hard cases: missing, empty,
   omitted, reference-visible, array-lift, and scalar coercion.
6. Convert function dispatch and typed kernels in controlled batches, keeping tests green after
   each batch.
7. Fold rich and extended values into `CalcValue` only after ordinary scalar/array/reference
   paths are stable.
8. Complete the OxFml callable lane before claiming callable migration: `SPECIAL.LAMBDA` and every
   helper/defined-name/returned-lambda path must produce or preserve `RichValue::Callable` with a
   real OxFml binding handle, and higher-order invocation must consume that carrier directly.
9. Remove temporary compatibility conversions last, with a final grep/audit gate proving no
   legacy carrier or old reference provider remains.

## 5. Initial Workstreams

1. **Inventory and taxonomy:** map every `EvalValue`, `CallArgValue`, `EvalArray`,
   `ArrayCellValue`, `ExtendedValue`, `LambdaValue`, `ReferenceLike`, `ReferenceResolver`, and
   `ReferenceTextResolver` use to its semantic role.
2. **Foundation shape:** update `oxfunc_value_types` and `oxfunc_core` so the native target shape
   exists before broad migration: typed `ReferenceLike`, `ReferenceSystemProvider`, FEC provider
   slot, compatibility adapters, and focused representation/provider tests. This workstream also
   records the split between passive value payloads and execution capabilities.
3. **Call-boundary migration:** replace `CallArgValue` with `CalcValue` at the call boundary,
   with `CoreValue::Missing`, `CoreValue::Empty`, and `CoreValue::Reference` carrying the
   former wrapper cases.
4. **Array model migration:** replace `EvalArray` / `ArrayCellValue` with `CalcArray` /
   `CalcValue` storage without losing Excel array-cell error and empty-cell semantics.
5. **Dispatch ABI migration:** change `FunctionCallTarget`, surface dispatch, scratch builders,
   and adapter calls to return and accept `CalcValue`.
6. **Kernel batch migration:** migrate functions by substrate/risk group, with parity tests per
   batch.
7. **OxFunc callable and rich migration:** move OxFunc lambda/callable and rich/object lanes onto
   `RichValue`.
8. **OxFml callable completion:** migrate OxFml's callable runtime from `EvalValue::Lambda` /
   `LambdaValue` tokens to `CalcValue` / `RichValue::Callable`; update `SPECIAL.LAMBDA`,
   builtin/helper callable production, defined-name callable production, returned/curried
   callable flow, `EvaluationOutput`, publication, re-supply, and `OxFmlCallableInvoker` so the
   real `OxFmlCallableBinding` handle is the executable carrier.
9. **Compatibility-shim removal:** remove every legacy value type; quarantine is allowed only as
   a migration-local step with an explicit deletion owner.
10. **Old reference-provider removal:** delete compatibility adapters, old resolver/text-resolver
   traits, direct `.target` call-site assumptions, and any active `HOST_REF_*` runtime identity.
11. **Cross-repo integration:** coordinate OxFml, OxCalc W060, and DnaTreeCalc follow-on work
   when the OxFunc ABI changes.

## 6. Safety Gates

No broad implementation batch should start until these planning gates are filled in:

1. current-type inventory with owner decisions,
2. FEC reference-system foundation decision record showing the first-batch typed
   `ReferenceLike`, `ReferenceSystemProvider`, FEC provider slot, compatibility adapters, and
   deletion path for `ReferenceResolver` / `ReferenceTextResolver`,
3. call-boundary decision record showing `CallArgValue` is fully represented by
   `CalcValue.core`,
4. array-cell decision record,
5. dispatch ABI decision record,
6. OxFml callable-carrier decision record and path inventory,
7. first kernel-batch list,
8. rollback/revert strategy for each code batch,
9. test matrix and Excel parity evidence requirement.

Each implementation gate must include:

1. `cargo test -p oxfunc_value_types -p oxfunc_core`,
2. `cargo test --manifest-path ../OxFml/crates/oxfml_core/Cargo.toml` for any OxFml callable
   batch,
3. focused tests for the migrated substrate,
4. grep/audit evidence for the type being retired from that substrate,
5. Excel replay or differential evidence when behavior changes.

## 7. Design Inputs From W098

W099 does not own unresolved end-state value-model decisions. It executes the W098 target:

1. `CalcValue` is the one system value type across inputs, outputs, prepared frame values,
   array cells, node values, rich values, and callable values.
2. There is no final `PreparedArgValue`, aggregate-provenance value carrier, bridge, shim,
   side-car, or alias.
3. `CalcArray` is an array of `CalcValue`; boundary/function policy is ported onto
   `CalcValue`, not encoded as narrower representation types.
4. References use `CoreValue::Reference`; value-only and reference-visible handling is expressed
   by function metadata and preparation policy. The reference payload is the W098/W060 typed
   host/profile identity, and reference behavior routes through the FEC `ReferenceSystemProvider`
   rather than the old resolver/text-resolver split.
5. Rich handling is declared by `RegistryFunctionMeta.rich_value_usage`; rich-blind functions
   use `.core`.
6. OxFml callable bindings live behind `OpaqueCallable`; OxFunc sees `CallableValue`, not OxFml
   AST/binding/cache internals.
7. W099 reaches its terminal gate only after the legacy value carriers and old reference
   providers are deleted and an audit proves no unowned old value or reference-provider path
   remains.

## 8. Evidence To Gather Next

1. Before broad migration, count every current use of `ReferenceResolver`, `ReferenceTextResolver`,
   `resolve_eval_value`, `resolve_reference_values`, `ReferenceLike.target`, `ReferenceKind`, and
   `HOST_REF_`; classify each as compatibility constructor, provider operation, textual-reference
   fact, or stringly identity that must be removed. This classification feeds the first
   foundation-shape batch, not a late cleanup pass.
2. Count and classify all `CallArgValue` match sites by target `CoreValue` mapping.
3. Count and classify all `EvalValue::Reference` and `CallArgValue::Reference` match sites,
   then collapse both to the `CoreValue::Reference` migration path unless a counterexample is
   found.
4. Identify every function that branches on `MissingArg` or `EmptyCell` and map those branches
   to `CoreValue::Missing` or `CoreValue::Empty`.
5. Identify every array conversion that maps unsupported `CalcValue.core` cases to
   `#VALUE!`, because those are likely semantic decisions rather than mechanical conversions.
6. Count and classify every OxFml `EvalValue::Lambda`, `LambdaValue`, `CallArgValue::Eval(
   EvalValue::Lambda(_))`, `PortableCallableValue`, and `EvaluationOutput::calc_value()` path.
   Decide which paths become direct `CalcValue` production, which become callable-aware
   re-supply/invocation plumbing, and which are staging-only compatibility paths.
7. Build the first migration ledger from the classification and assign each legacy carrier,
   compatibility adapter, and old reference-provider path to a deletion batch.

## 9. Closure Gate

W099 reaches its terminal gate only when the full OxFunc function stack uses `CalcValue` as the
native semantic value type, OxFunc reference behavior routes through `ReferenceSystemProvider`
with the typed W098/W060 reference payload, OxFml uses `CalcValue` / `RichValue::Callable` as the
native callable value representation, and the final audit shows no unowned legacy value or old
reference-provider path remains.

Until then, report this workset as `scope_partial`, `target_partial`, and `integration_partial`.

## 9A. Final Refactoring Review Corrections

Fresh review as a code-refactoring plan leaves these constraints on the execution sequence:

1. the first code batch must be a foundation-shape batch, not a broad call-site sweep;
2. compatibility adapters are allowed only when they keep the repo compiling during migration,
   and every adapter must have a deletion owner and deletion batch at creation time;
3. no batch may make old names look like final architecture by type aliasing
   `EvalValue = CalcValue`, wrapping `CalcValue` in a new argument carrier, or preserving
   `ReferenceResolver` / `ReferenceTextResolver` as the native FEC abstraction;
4. each broad migration batch must retire one semantic surface, not merely add parallel
   `CalcValue` entry points beside legacy entry points;
5. behavior-changing fixes discovered during migration must be isolated, tested, and justified as
   Excel-compliance improvements or routed through the ordinary bug stream;
6. every batch must leave the tree formatted, compiling, and with focused tests for the migrated
   substrate, unless a batch is explicitly marked as a coordinated cross-repo break and repaired
   by the immediately following bead;
7. final audit is a source-level deletion audit, not just a public-API audit.

The terminal code shape is "as if planned with `CalcValue` from the start":

1. public and crate-internal function-call APIs accept and return `CalcValue`;
2. function kernels match on `CalcValue.core` and inspect `.rich` only through declared
   metadata/admission policy;
3. arrays are `CalcArray` of `CalcValue`;
4. references are `CoreValue::Reference(ReferenceLike)` with typed host/profile identity;
5. FEC exposes one `ReferenceSystemProvider` reference capability;
6. callables are `RichValue::Callable(CallableValue)` with an OxFml-owned opaque handle;
7. old value carriers, old reference providers, old array cells, old extended-value carriers,
   and migration adapters are deleted.

## 10. Sequential Bead Breakdown

Live execution truth belongs in `.beads/`.

Initial epic:

1. `oxf-im4m` - W099 CalcValue end-to-end migration planning.

The child beads have been created in this order. Later beads may be split by risk group, but the
order below is the dependency order. Concrete bead ids are recorded in each heading.

### W099-001 Inventory And Deletion Ledger (`oxf-im4m.1`)

Scope:

1. count and classify all uses of `EvalValue`, `CallArgValue`, `PreparedArgValue`,
   `EvalArray`, `ArrayCellValue`, `ExtendedValue`, `LambdaValue`, `ReferenceLike`,
   `ReferenceKind`, `ReferenceResolver`, `ReferenceTextResolver`, `resolve_eval_value`,
   `resolve_reference_values`, and `HOST_REF_`;
2. assign each use to final owner: `CalcValue`, `CalcArray`, `RichValue`, `CallableValue`,
   `ReferenceSystemProvider`, transient local preparation fact, test fixture, or deletion;
3. create a deletion ledger mapping every compatibility adapter and legacy name to the bead that
   removes it.

Current artifact:

1. `docs/worksets/W099_CALCVALUE_INVENTORY_AND_DELETION_LEDGER.md`
2. `docs/worksets/W099_CALCVALUE_OCCURRENCE_LEDGER.csv`

Acceptance:

1. inventory artifact committed under W099;
2. deletion owners recorded for every legacy carrier/provider;
3. no implementation migration starts without this ledger.

### W099-002 Value-Type Foundation Shape (`oxf-im4m.2`)

Scope:

1. correct `oxfunc_value_types` to the W098 endpoint: `CoreValue`, `CalcValue`, `CalcArray`,
   `RichValue::{Object, Callable, Presentation, ErrorMetadata}`, `CallableValue`,
   `OpaqueCallable`, and typed `ReferenceLike`;
2. ensure `CalcArray` stores `CalcValue` and admits missing, empty, nested arrays, rich values,
   callables, and references at the representation level;
3. ensure `CallableValue` equality is handle identity plus arity;
4. provide constructors/accessors that make `.core` and `.rich` usage deliberate.

Acceptance:

1. value-type invariant tests pass;
2. no final-looking `EvalValue = CalcValue` alias exists;
3. current compatibility conversions are explicitly marked migration-only in code and ledger.

### W099-003 ReferenceSystemProvider FEC Foundation (`oxf-im4m.3`)

Scope:

1. add `ReferenceSystemProvider` plus minimal request/result/error types for describe,
   dereference, enumerate, query facts, resolve text, and transform/compose;
2. add the provider slot to `FunctionExecutionContextBundle` and `FunctionExecutionContext`;
3. adapt old resolver/text-resolver tests and call sites through temporary compatibility adapters;
4. add tests for textual identity, opaque identity, display-as-non-identity, provider
   dereference, and provider text resolution.

Acceptance:

1. new FEC-facing code can use `ReferenceSystemProvider`;
2. old `ReferenceResolver` / `ReferenceTextResolver` are no longer the native abstraction;
3. compatibility adapters have deletion owners;
4. source scan records current `HOST_REF_` and `.target` residue.

### W099-004 Central CalcValue Construction And Coercion Helpers (`oxf-im4m.4`)

Scope:

1. move scalar, error, empty, missing, array, reference, rich, and callable constructors to
   `CalcValue`;
2. port central coercion and projection helpers to `CalcValue.core`;
3. make rich-aware access helper(s) metadata-driven rather than ad hoc.

Acceptance:

1. existing scalar/coercion tests pass through the new helpers;
2. new code paths do not construct legacy carriers except through ledgered migration adapters.

### W099-005 Call-Boundary Migration (`oxf-im4m.5`)

Scope:

1. replace `CallArgValue` at the function-call boundary with `CalcValue`;
2. map omitted and empty arguments to `CoreValue::Missing` and `CoreValue::Empty`;
3. keep direct-array versus reference-visible behavior by starting from `CoreValue::Array` versus
   `CoreValue::Reference`;
4. update `FunctionCallScratch` and call-target invocation surfaces.

Acceptance:

1. no native call-boundary API takes `CallArgValue`;
2. focused missing/empty/reference-visible tests pass;
3. any remaining `CallArgValue` mentions are ledgered compatibility residue only.

### W099-006 Array Model Migration (`oxf-im4m.6`)

Scope:

1. replace `EvalArray` / `ArrayCellValue` storage with `CalcArray` / `CalcValue`;
2. port array iteration, shape validation, row-major access, spill/dynamic-array helpers, and
   array-cell coercion policy;
3. preserve Excel empty-cell and error-cell semantics.

Acceptance:

1. array, dynamic-array, and direct-call array seam tests pass;
2. unsupported-element policy is explicit and tested;
3. remaining `EvalArray` / `ArrayCellValue` uses are ledgered for deletion.

### W099-007 Preparation And Adapter Migration (`oxf-im4m.7`)

Scope:

1. remove `PreparedArgValue` as a value carrier;
2. represent preparation results as `CalcValue` plus transient local facts where needed;
3. port aggregate/reference-derived facts without creating aggregate-provenance values;
4. update function adapters to consume `CalcValue`.

Acceptance:

1. no final `PreparedArgValue` replacement type exists;
2. aggregate/reference-sensitive tests pass;
3. transient facts do not escape as value carriers.

### W099-008 Dispatch ABI Migration (`oxf-im4m.8`)

Scope:

1. migrate surface dispatch, generated dispatch-by-index, `FunctionCallTarget`, scratch builders,
   and registry invocation paths to `CalcValue`;
2. keep metadata-driven preparation and rich-awareness visible at the dispatch boundary.

Acceptance:

1. public OxFunc value-call path returns `CalcValue`;
2. generated dispatch is refreshed;
3. focused call-target parity tests pass.

### W099-009 Reference-Sensitive Function Migration (`oxf-im4m.9`)

Scope:

1. move `INDEX`, `OFFSET`, `INDIRECT`, reference operators, sparse/aggregate consumers, and
   caller-context reference functions onto `ReferenceSystemProvider`;
2. remove direct `.target` parsing from function logic except compatibility textual constructors
   scheduled for deletion;
3. route dereference, enumeration, facts, text resolution, and transform/compose through FEC.

Acceptance:

1. reference-visible/value-only behavior is tested;
2. provider calls are capability-checked;
3. no function kernel treats display text as identity.

### W099-010 Rich And Extended Value Fold (`oxf-im4m.10`)

Scope:

1. replace `ExtendedValue` with `CalcValue { core, rich }`;
2. map structured rich objects to `RichValue::Object`;
3. map presentation and error metadata to `RichValue::Presentation` /
   `RichValue::ErrorMetadata`;
4. add/update `RegistryFunctionMeta.rich_value_usage` and exports.

Acceptance:

1. `IMAGE`, `_webimage`, `HYPERLINK`, `NOW`, and `TODAY` rich/presentation tests pass;
2. rich-blind functions degrade through `.core`;
3. no final `ExtendedValue` path remains.

### W099-011 OxFunc Callable Migration (`oxf-im4m.11`)

Scope:

1. change `CallableInvoker` and callable helpers to use `CallableValue`;
2. update `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`, `GROUPBY`, and `PIVOTBY`;
3. remove `LambdaValue` assumptions from OxFunc kernels and tests except temporary adapters.

Acceptance:

1. higher-order OxFunc tests pass;
2. callable equality is handle identity plus arity;
3. no kernel consumes `EvalValue::Lambda` as the native callable path.

### W099-012 Kernel Batch Migration (`oxf-im4m.12`)

Scope:

1. migrate typed kernels by substrate/risk group from `EvalValue` matches to `CalcValue.core`;
2. keep behavior changes isolated and evidenced;
3. refresh focused tests per batch.

Suggested split:

1. arithmetic/comparison/text core;
2. lookup/reference-adjacent functions after W099-009;
3. dynamic-array shaping;
4. financial/date/time/statistical;
5. provider-bound functions after FEC surfaces are stable.

Acceptance:

1. each substrate batch removes native legacy carrier use from its files;
2. focused parity tests pass;
3. no unrelated formatting churn is mixed into semantic migration commits.

### W099-013 OxFml Callable And CalcValue Follow-Through (`oxf-im4m.13`)

Scope:

1. migrate OxFml eval/output/publication/re-supply surfaces to `CalcValue`;
2. make `SPECIAL.LAMBDA`, defined-name callables, returned/curried lambdas, and helper callables
   produce `RichValue::Callable`;
3. update `OxFmlCallableInvoker` to downcast and invoke the opaque handle.

Acceptance:

1. OxFml callable suites pass;
2. no native `EvalValue::Lambda` callable path remains;
3. publication/re-supply preserves callable `CalcValue`.

### W099-014 OxCalc W060 Integration (`oxf-im4m.14`)

Scope:

1. wire OxCalc TreeCalc runtime references to typed `ReferenceLike`;
2. replace active runtime `HOST_REF_*` identities with host-owned reference handles;
3. implement the TreeCalc `ReferenceSystemProvider`;
4. keep dependency graph construction on `BoundFormula` / `BoundExpr`, not calc-time
   `CalcValue::Reference`.

Acceptance:

1. OxCalc W060 canonical tests pass;
2. no active runtime `HOST_REF_*` identity remains;
3. CTRO examples preserve graph/runtime separation.

### W099-015 Legacy Type And Adapter Deletion (`oxf-im4m.15`)

Scope:

1. delete `EvalValue`, `CallArgValue`, `PreparedArgValue`, `EvalArray`, `ArrayCellValue`,
   `ExtendedValue`, `LambdaValue`, `ReferenceResolver`, `ReferenceTextResolver`, migration
   adapters, and old conversion helpers;
2. remove old generated exports or compatibility modules;
3. refresh imports and public API docs.

Acceptance:

1. source scan proves deleted names are absent except historical docs/tests explicitly outside
   active code;
2. no bridge/shim/side-car path remains;
3. code reads as native `CalcValue` architecture.

### W099-016 Final Audit And Cross-Repo Validation (`oxf-im4m.16`)

Scope:

1. run full OxFunc tests and focused Excel parity evidence;
2. run affected OxFml/OxCalc/DnaTreeCalc suites;
3. run source scans for old names, old reference identity, and compatibility adapters;
4. update workset/register/beads with final evidence and any residual follow-up that is outside
   W099 terminal scope.

Acceptance:

1. final audit shows no unowned legacy value or old reference-provider path;
2. all required local and downstream validation passes or failures are recorded with blockers;
3. W099 terminal gate evidence is ready for closure review under OPERATIONS.md.
