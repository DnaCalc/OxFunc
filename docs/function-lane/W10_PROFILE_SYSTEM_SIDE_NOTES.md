# W10 Profile/System Side Notes

Status: `active`
Workset: `W10`

Purpose:
1. capture interesting or problematic situations discovered during W10 execution,
2. isolate potential changes to profile/classification/coercion/deref systems,
3. preserve concrete follow-up items without blocking W10 closure.

## Notes Log

1. `SUM` direct-scalar-vs-array-like coercion split:
   - current OxFunc aggregate preparation now carries explicit input-structure classes for:
     - `direct_scalar`
     - `direct_array_literal`
     - `reference_derived`
     - `opaque_array_value`
   - `SUM` uses direct-scalar policy only for `direct_scalar`; all array-like origins use scan policy.
   - explicit fallback when upstream source structure is absent: treat raw evaluated arrays as `opaque_array_value` and apply scan policy.
   - profile-system implication:
     - OxFunc should model the direct-scalar versus array-like distinction explicitly without pretending `SUM` needs raw reference identity.
     - OxFml should preserve richer upstream structure only where later functions actually require it.
2. `IF` lazy branch evaluation:
   - eager pre-adapter preparation of all arguments conflicts with strict branch laziness guarantees.
   - profile-system implication:
     - add explicit `arg_evaluation_strategy` class (eager/lazy-branch/selective).
3. Reference-observable and reference-returning functions:
   - `INDEX` and `XLOOKUP` need reference identity visible on input and on selected results.
   - `INDIRECT` is different: it is values-only on input, but may return a reference result.
   - profile-system implication:
     - add return-shape classification axis (`value_only`, `reference_possible`, `array_shape_only`, `array_payload`).
4. Volatile/time functions (`NOW`):
   - current function core has no shared provider seam beyond ad hoc local traits.
   - profile-system implication:
     - standardize provider seams (`time_provider`, `random_provider`) in runtime and formal scaffolding.
5. Dynamic array payload gap (`SEQUENCE`):
   - W10 originally exposed a shape-only gap for `SEQUENCE`; the current runtime now materializes full row-major payload arrays and the Lean model must stay aligned with payload, not just shape.
   - profile-system implication:
     - keep array payload semantics first-class in function/runtime/formal layers and avoid regressing to shape-only placeholders.
6. Seed-mode declaration needed for profile clarity:
   - W10 originally used seed-mode language for `MATCH`, `XLOOKUP`, `INDIRECT`, and `INDEX`; after the current closure pass the remaining concern is not seed-mode coverage for those functions, but keeping reference-return, caller-context, and host-boundary semantics explicit in the contracts.
   - profile-system implication:
     - keep any future partial-mode declaration machine-visible, but do not use seed-language to mask already-closed function semantics.
7. Reference-return policy needs first-class contract axis:
   - `INDEX`/`XLOOKUP` can return references from reference-observable selection, while `INDIRECT` can return references from values-only text interpretation.
   - profile-system implication:
     - add `return_reference_policy` (`never`, `possible`, `required`) plus `return_provenance_policy` notes.
8. Provider seams are now heterogeneous:
   - `NOW` uses an explicit provider trait; other host-sensitive functions still rely on ad hoc context assumptions.
   - profile-system implication:
     - standardize provider seam contracts (`time_provider`, `caller_context_provider`, `external_provider`) with shared runtime and Lean signatures.
9. XLL codegen is now profile-derived across the function catalog:
   - U-vs-Q variant emission is generated from `FunctionMeta` profile rules (not hand-curated function lists).
   - profile-system implication:
     - keep function metadata sufficient for mechanical export policy derivation, with explicit handling for very-high-arity signatures and non-scalar payload lanes.
10. XLOOKUP reference-return observability is now empirically pinned:
   - `CELL("address", XLOOKUP(...))` and `SUM(XLOOKUP(...):XLOOKUP(...))` confirm reference identity and range composition lanes.
   - profile-system implication:
     - preserve `return_reference_policy` as first-class contract metadata and keep it distinct from deref policy.
