# HO-FN-016 - Compiled Surface Call-Site and Full-Catalog Index Dispatch

Direction: `OxFunc -> OxFml`
Source repo/workset: `OxFunc/W096`
Target repo/workset: `OxFml compiled lambda/model evaluation follow-up`
Filed: `2026-05-07`
Status: `filed`

## Purpose

Tell OxFml that W096 has opened the OxFunc-side seam for generic compiled
formula plans and whole-model optimization without moving Excel function
semantics into OxFml.

OxFml should be able to resolve a function/operator call site once, retain a
stable handle in its compiled plan, and invoke OxFunc through that handle rather
than repeatedly passing surface names through the broad dispatcher.

## OxFunc Surfaces

W096 adds or advances these surfaces:

1. `oxfunc_core::surface_call::SurfaceCallSite`
2. `oxfunc_core::surface_call::SurfaceCallRuntime`
3. `oxfunc_core::surface_call::SurfaceCallScratch`
4. `oxfunc_core::surface_call::SurfaceCallHoistPolicy`
5. `oxfunc_core::functions::surface_dispatch::SurfaceDispatchKey`
6. resolved-key invocation through `eval_surface_value_call_with_dispatch_key(...)`
7. compatibility string invocation through `eval_surface_value_call_with_callable(...)`

The compatibility string API is preserved, but it now resolves to a
`SurfaceDispatchKey` and delegates through the resolved dispatch path.

The generated backend dispatches by catalog index rather than by repeatedly
matching function-id strings. Non-dispatchable catalog entries remain explicit
`#VALUE!` outcomes through the generated fallback rather than OxFml-side
special cases.

## Metadata Available To OxFml

`SurfaceCallSite` exposes:

1. function id,
2. surface name,
3. canonical surface name,
4. `FunctionMeta`,
5. arity,
6. volatility,
7. determinism,
8. host interaction class,
9. adapter and surface FEC dependency profiles,
10. callable argument specs and arity-dependent ordinals,
11. hoistability checks under explicit runtime-context policy.

These fields are intended to support generic plan construction, purity gates,
constant hoisting, and future graph-level optimization without hard-coding
function-specific behavior in OxFml.

## Runtime Contract

`SurfaceCallRuntime` bundles the runtime-only providers used by OxFunc function
semantics:

1. reference resolver,
2. now provider or fixed now serial,
3. random provider or fixed random value,
4. locale context,
5. host-info provider,
6. callable invoker,
7. RTD provider,
8. registered external provider.

OxFml should keep formula structure, lexical slots, LET/LAMBDA binding,
reference binding, child evaluation order, and trace policy in OxFml. OxFunc
continues to own function/operator semantics, coercion, reference handling,
array lifting, and error behavior.

## Scratch / Reuse Contract

`SurfaceCallScratch` is an optional reusable argument buffer for repeated
compiled call-site invocation. It is not a semantic cache. OxFml may reuse it
inside hot loops or compiled graph execution as long as argument construction and
call ordering remain unchanged.

Future scratch expansion should remain generic and must not require OxFml to
special-case individual functions such as `INDEX`, `HSTACK`, arithmetic
operators, or helper functions.

## Requested OxFml Follow-Up

OxFml should:

1. replace repeated broad dispatcher calls in compiled/hot paths with
   `SurfaceCallSite` handles where formula binding has already resolved the
   function/operator identity,
2. keep value-only versus full trace execution as an OxFml trace-mode decision,
3. use `SurfaceCallSite` metadata for generic purity/hoisting gates rather than
   function-specific OxFml tables,
4. continue using `CallableInvoker::invoke_many(...)` from `HO-FN-015` for
   lambda-helper hot-loop batching,
5. report any missing metadata needed for full-model graph compilation as a
   narrow OxFunc follow-up rather than adding a mirror registry in OxFml.

## Validation State

OxFunc W096 library validation:

1. `cargo check --manifest-path crates\oxfunc_core\Cargo.toml` passes.
2. The generated catalog-index dispatcher compiles in `oxfunc_core`.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib surface_call`
   passes with 8 focused W096 tests, including operator, `INDEX`,
   `HSTACK`/`VSTACK`, volatile time/random, host-sensitive, helper/callable,
   locale-sensitive, RTD, registered-external provider, scratch, metadata, and
   full-registry dispatch-key coverage.

## Open Lanes

1. OxFml acknowledgement and compiled-plan consumption design,
2. downstream OxFml adoption of `SurfaceCallSite`, `SurfaceCallRuntime`, and
   `SurfaceCallScratch` in compiled/hot paths.
