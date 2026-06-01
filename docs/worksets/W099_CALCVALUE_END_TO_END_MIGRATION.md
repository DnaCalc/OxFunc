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
3. any temporary compatibility conversion introduced during migration has been removed before
   closure,
4. no function kernel or public OxFunc value-call path uses a legacy carrier as its native semantic
   value type,
5. OxFml no longer uses `EvalValue::Lambda` as the native callable value representation:
   `SPECIAL.LAMBDA`, defined-name callable resolution, helper callables, higher-order invocation,
   returned/curried lambdas, publication, and re-supply all use `CalcValue` with
   `RichValue::Callable` backed by a real `OxFmlCallableBinding` handle,
6. existing Excel parity evidence remains green, and any behavior change discovered during the
   migration is either proven to improve Excel compliance or promoted through the ordinary bug
   stream.

## 2. Relationship To W098

`W098_UNIFIED_VALUE_MODEL_AND_CALLABLE_RICH.md` defines the target model:

1. `CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }` is the value carrier.
2. Kernels read `.core` deliberately.
3. Callable values move from `EvalValue::Lambda` to `RichValue::Callable`.
4. Structured rich values fold into the `RichValue` layer.
5. OxFml owns the concrete callable binding behind `OpaqueCallable`; W099 must carry the OxFml
   follow-through because a callable value model is not complete while OxFml still treats
   `EvalValue::Lambda` as the primary callable carrier.

`W099` must not reopen that model casually. It may refine the model only when migration evidence
shows a concrete contradiction, missing invariant, or unsafe edge.

## 2A. Batch 0 Scaffold Review

W099 starts with an existing partial scaffold committed as `da0c023`
(`Scaffold CalcValue value model`). Treat this as **Batch 0 scaffold/staging**, not as a
completed migration batch.

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

W099 implementation work must continue by turning this review into explicit child beads and a
first real migration-batch plan: preserve the useful scaffold, remove staging bridges as their
call sites migrate, and keep unrelated behavior changes or formatting churn out of semantic
migration commits.

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
by recording provenance in the prepared-value layer where required.

## 4. Migration Strategy

Use compiler-guided breadth, but gate each semantic category before the broad mechanical sweep.

1. Inventory every value-shaped type and classify it as value, argument metadata, array storage,
   rich/presentation carrier, reference carrier, or compatibility shim.
2. Define the final public and crate-internal ABI names before editing thousands of call sites.
3. Move central constructors, coercion helpers, array iteration, and error constructors to
   `CalcValue` first.
4. Convert argument preparation next, because it decides the hard cases: missing, empty,
   omitted, reference-visible, array-lift, and scalar coercion.
5. Convert function dispatch and typed kernels in controlled batches, keeping tests green after
   each batch.
6. Fold rich and extended values into `CalcValue` only after ordinary scalar/array/reference
   paths are stable.
7. Complete the OxFml callable lane before claiming callable migration: `SPECIAL.LAMBDA` and every
   helper/defined-name/returned-lambda path must produce or preserve `RichValue::Callable` with a
   real OxFml binding handle, and higher-order invocation must consume that carrier directly.
8. Remove temporary compatibility conversions last, with a final grep/audit gate proving no
   legacy carrier remains.

## 5. Initial Workstreams

1. **Inventory and taxonomy:** map every `EvalValue`, `CallArgValue`, `EvalArray`,
   `ArrayCellValue`, `ExtendedValue`, and `LambdaValue` use to its semantic role.
2. **Call-boundary migration:** replace `CallArgValue` with `CalcValue` at the call boundary,
   with `CoreValue::Missing`, `CoreValue::Empty`, and `CoreValue::Reference` carrying the
   former wrapper cases.
3. **Array model migration:** replace `EvalArray` / `ArrayCellValue` with `CalcArray` /
   `CalcValue` storage without losing Excel array-cell error and empty-cell semantics.
4. **Dispatch ABI migration:** change `FunctionCallTarget`, surface dispatch, scratch builders,
   and adapter calls to return and accept `CalcValue`.
5. **Kernel batch migration:** migrate functions by substrate/risk group, with parity tests per
   batch.
6. **OxFunc callable and rich migration:** move OxFunc lambda/callable and rich/object lanes onto
   `RichValue`.
7. **OxFml callable completion:** migrate OxFml's callable runtime from `EvalValue::Lambda` /
   `LambdaValue` tokens to `CalcValue` / `RichValue::Callable`; update `SPECIAL.LAMBDA`,
   builtin/helper callable production, defined-name callable production, returned/curried
   callable flow, `EvaluationOutput`, publication, re-supply, and `OxFmlCallableInvoker` so the
   real `OxFmlCallableBinding` handle is the executable carrier.
8. **Compatibility-shim removal:** remove or quarantine every legacy value type.
9. **Cross-repo integration:** coordinate OxCalc and DnaTreeCalc follow-on work when the
   OxFunc ABI changes.

## 6. Safety Gates

No implementation batch should start until these planning gates are filled in:

1. current-type inventory with owner decisions,
2. call-boundary decision record showing `CallArgValue` is fully represented by
   `CalcValue.core`,
3. array-cell decision record,
4. dispatch ABI decision record,
5. OxFml callable-carrier decision record and path inventory,
6. first kernel-batch list,
7. rollback/revert strategy for each code batch,
8. test matrix and Excel parity evidence requirement.

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
   by function metadata and preparation policy.
5. Rich handling is declared by `RegistryFunctionMeta.rich_value_usage`; rich-blind functions
   use `.core`.
6. OxFml callable bindings live behind `OpaqueCallable`; OxFunc sees `CallableValue`, not OxFml
   AST/binding/cache internals.
7. W099 closes only after the legacy value carriers are deleted and an audit proves no unowned
   old value path remains.

## 8. Evidence To Gather Next

1. Count and classify all `CallArgValue` match sites by target `CoreValue` mapping.
2. Count and classify all `EvalValue::Reference` and `CallArgValue::Reference` match sites,
   then collapse both to the `CoreValue::Reference` migration path unless a counterexample is
   found.
3. Identify every function that branches on `MissingArg` or `EmptyCell` and map those branches
   to `CoreValue::Missing` or `CoreValue::Empty`.
4. Identify every array conversion that maps unsupported `CalcValue.core` cases to
   `#VALUE!`, because those are likely semantic decisions rather than mechanical conversions.
5. Count and classify every OxFml `EvalValue::Lambda`, `LambdaValue`, `CallArgValue::Eval(
   EvalValue::Lambda(_))`, `PortableCallableValue`, and `EvaluationOutput::calc_value()` path.
   Decide which paths become direct `CalcValue` production, which become callable-aware
   re-supply/invocation plumbing, and which are staging-only compatibility paths.
6. Build the first migration ledger from the classification and assign each legacy carrier to a
   deletion batch.

## 9. Closure Gate

W099 reaches its terminal gate only when the full OxFunc function stack uses `CalcValue` as the
native semantic value type, OxFml uses `CalcValue` / `RichValue::Callable` as the native callable
value representation, and the final audit shows no unowned legacy value path remains.

Until then, report this workset as `scope_partial`, `target_partial`, and `integration_partial`.

## 10. Bead Rollout

Live execution truth belongs in `.beads/`.

Initial epic:

1. `oxf-im4m` - W099 CalcValue end-to-end migration planning.

First child beads should be created from the settled W098 decisions. The likely first children
are:

1. `CallArgValue` / `EvalValue` classification and call-boundary replacement record.
2. `EvalArray` / `ArrayCellValue` classification and array-cell decision record.
3. OxFml callable-carrier inventory and `EvalValue::Lambda` replacement plan.
4. Dispatch ABI migration decision record.
5. First kernel-batch selection and test matrix.
