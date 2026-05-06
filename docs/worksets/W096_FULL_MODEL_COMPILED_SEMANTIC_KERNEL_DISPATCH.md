# W096 Full-Model Compiled Semantic Kernel Dispatch

Status: `planned`

## 1. Purpose

Shape OxFunc's function-surface and dispatch architecture for long-term full-model optimization, where large Excel workbooks can be recalculated through resolved function handles, graph-level planning, concurrency, cacheable runtime contexts, and future compiled backends without moving function semantics out of OxFunc.

## 2. Strategic Objective

OxFunc should become a compilable semantic kernel library for Excel-compatible function and operator behavior.

The long-term target is not only faster lambda hot loops. It is a stack where:

1. OxFml owns formula structure, scopes, slots, references, LET/LAMBDA binding, control-flow planning, and calculation-graph scheduling.
2. OxFunc owns every function and operator semantic: coercion, lifting, reference visibility, error behavior, host/runtime dependencies, and array shape behavior.
3. Downstream evaluators call resolved OxFunc handles rather than repeatedly passing surface strings through a broad dispatcher.
4. Pure inner kernels remain typed Rust functions that can later be inlined, specialized, vectorized, lowered, or translated into another execution representation.
5. Surface-level execution has one uniform full-catalog ABI suitable for interpreted, cached, concurrent, and compiled graph backends.

## 3. Problem Statement

The current surface path is good enough for correctness-oriented invocation but not ideal for large optimized calculation graphs:

1. Function calls are still primarily routed through string-based broad dispatch.
2. Surface functions commonly expose generic `&impl ReferenceResolver` signatures, which blocks a simple full-catalog handler table with one erased ABI.
3. Function metadata is useful but not yet rich enough for whole-graph scheduling and lowering decisions.
4. Scratch/state reuse exists only in localized paths and is not yet a first-class surface-call contract.
5. The current optimization pass started from lambda-body hot loops, but the more important future is whole-model recalculation throughput.

## 4. Architectural Direction

The W096 direction is a uniform full-catalog model:

1. Resolve function id or surface name once into a stable call-site handle.
2. Execute through a full-catalog dispatch table keyed by a stable dispatch key or catalog ordinal.
3. Use one erased runtime-context ABI for all surface handlers.
4. Keep the old compatibility call path as a wrapper if useful, but do not optimize around it.
5. Keep typed inner kernels separate from the erased surface ABI.
6. Generate or table-drive dispatch for the whole function catalog rather than adding hand-picked fast paths for `INDEX`, `HSTACK`, arithmetic operators, or lambda helpers.

## 5. In Scope

1. `SurfaceCallSite` as the resolved immutable function-handle surface.
2. `SurfaceCallRuntime` as the canonical runtime context for resolver, locale, time, random, host info, callable invocation, RTD, and registered external providers.
3. A uniform full-catalog `SurfaceHandler` ABI.
4. Mechanical resolver-signature refactor toward `&(impl ReferenceResolver + ?Sized)` or `&dyn ReferenceResolver` where needed for handler-table compatibility.
5. Generated or table-driven dispatch over all built-in functions and admitted operator surfaces.
6. Compatibility wrappers from existing string-based public APIs onto resolved dispatch.
7. Scratch/state contracts for repeated call-site invocation and array-lift/arg-preparation reuse.
8. Metadata expansion for graph-level optimization.
9. Parity tests proving resolved/table dispatch matches the current surface path across the full representative matrix.
10. OxFml handoff for formula-plan consumption once the OxFunc-local seam is stable.

## 6. Out of Scope

1. Moving function-specific semantics into OxFml.
2. Hand-specializing a few currently hot functions as the primary architecture.
3. Changing Excel-visible function semantics for performance.
4. Claiming whole-model optimization completion from the dispatch refactor alone.
5. Implementing OxFml calculation-graph scheduling or compiled formula plans in this repo.
6. Non-parity helper extensions such as `REDUCE.STOP`.

## 7. Required Metadata For Future Optimizers

W096 should make these facts available at or near the resolved call site:

1. function id,
2. canonical surface name,
3. arity,
4. argument preparation profile,
5. volatility,
6. determinism,
7. thread-safety class,
8. host interaction class,
9. adapter-level FEC dependency profile,
10. surface-level FEC dependency profile,
11. callable/lambda argument positions,
12. reference visibility and reference-result behavior,
13. array-lift and broadcast behavior,
14. scalar/array shape behavior,
15. error propagation class where known,
16. locale, time, random, caller-context, host, external, and resolver dependency facts,
17. whether a function is pure after argument preparation under a fixed runtime context.

## 8. Initial Epic Lanes

1. Resolved call-site API consolidation.
2. Erased runtime context and provider ABI.
3. Resolver-signature mechanical refactor.
4. Full-catalog dispatch table generation or table authoring.
5. String API compatibility wrappers onto resolved dispatch.
6. Inner-kernel preservation and typed-kernel inventory.
7. Scratch/state reuse contract.
8. Optimizer metadata enrichment.
9. Full-catalog parity and representative Excel-seam tests.
10. OxFml compiled-plan consumption handoff.

## 9. Acceptance Intent

W096 reaches its OxFunc-local gate when:

1. every built-in registry entry and admitted operator surface resolves to a uniform dispatch key,
2. the primary value-call path can invoke through the full-catalog handler table without re-matching on surface strings,
3. existing public string-based calls are compatibility wrappers rather than the optimized internal path,
4. function-specific semantics remain inside OxFunc,
5. no function is promoted through a special one-off fast path as a substitute for full-catalog dispatch,
6. focused parity tests cover arithmetic operators, reference-visible functions, array-lift cases, `INDEX`, `HSTACK`/`VSTACK`, volatile/time/random functions, host-sensitive functions, helper/callable functions, RTD/registered-external provider seams, and locale-sensitive functions,
7. at least one full-registry invariant test proves every built-in entry has a dispatch handler or an explicit non-invokable classification,
8. OxFml receives a handoff describing the resolved call-site API, runtime context, trace/value-mode implication, and remaining integration obligations.

## 10. Trace Mode Decision

Trace mode is not an OxFunc semantic decision by itself.

The intended split is:

1. OxFunc value execution remains the canonical semantic result path.
2. OxFml owns whether a compiled plan runs in value-only mode or full verification-trace mode.
3. If trace mode is required on hot paths, OxFml should use reusable trace templates and stamp per-call values rather than forcing OxFunc to allocate per-call trace structures.
4. OxFunc may expose metadata and stable call-site ids needed for trace templates, but should not make tracing part of ordinary value invocation.

## 11. Relationship To Existing Worksets

1. W091 owns the canonical runtime function registry and remains the source for registry truth.
2. W093 owns UDF registration and name-resolution seam design.
3. W094 owns locale-profile facts consumed by runtime contexts.
4. W095 owns callable batching and lambda-helper hot-loop allocation work.
5. W096 generalizes the W095 call-site direction from lambda hot loops to whole-model optimized recalculation.

## 12. Tracking

1. Primary code surfaces:
   `crates/oxfunc_core/src/surface_call.rs`,
   `crates/oxfunc_core/src/functions/surface_dispatch.rs`,
   `crates/oxfunc_core/src/registry.rs`,
   `crates/oxfunc_core/src/xll_export_specs.rs`
2. Upstream dependency:
   `OxFml` compiled formula/evaluation-plan design.
3. Downstream dependency:
   `DnaOneCalc` large-model recalculation and performance replay.
4. Bead:
   `oxf-tu5k`.

## 2026-05-07 OxFunc-local execution update

execution_state: `local_target_satisfied_downstream_ack_pending`

Evidence:

1. `cargo check --manifest-path crates\oxfunc_core\Cargo.toml` passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib surface_call`
   passed with 8 focused W096 tests.
3. The focused parity matrix covers operator dispatch, `INDEX`,
   `HSTACK`/`VSTACK`, volatile time/random, host-sensitive, helper/callable,
   locale-sensitive, RTD provider, registered-external provider, scratch reuse,
   planning metadata, and full built-in registry dispatch-key resolution.
4. `HO-FN-016` is filed and registered for OxFml compiled-plan consumption.

Remaining downstream lane:

1. OxFml acknowledgement and adoption of `SurfaceCallSite`,
   `SurfaceCallRuntime`, and `SurfaceCallScratch` in compiled/hot paths.

## 13. Reporting Contract

All W096 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Current status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: open execution bead, mechanical resolver-signature refactor, full-catalog handler table, metadata enrichment, parity tests, OxFml handoff.
