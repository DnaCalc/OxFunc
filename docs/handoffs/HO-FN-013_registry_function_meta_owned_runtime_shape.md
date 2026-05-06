# HO-FN-013 - Owned runtime registry metadata shape

Status: filed
Direction: OxFunc -> OxFml
Source workset: W091
Target workset: OxFml W068 follow-up
Filed date: 2026-05-04
Related inbound: HANDOFF-OXFUNC-005
Related prior handoff: HO-FN-011

## Purpose

Record the narrow OxFunc registry API hardening made after OxFml landed W068 canonical registry consumption.

OxFml's W068 direction is correct: function help must use `oxfunc_core::registry` and must not reintroduce `arity_shape_note`, synthetic signatures, `argN`, `additional_args`, or a comprehensive OxFml function list.

## API shape to consume

`FunctionEntry.meta` is now `RegistryFunctionMeta`, not the built-in-only `FunctionMeta` type.

`RegistryFunctionMeta` preserves the fields OxFml needs:

1. `function_id: String`,
2. `arity`,
3. `determinism`,
4. `volatility`,
5. `host_interaction`,
6. `thread_safety`,
7. `arg_preparation_profile`,
8. `coercion_lift_profile`,
9. `kernel_signature_class`,
10. `fec_dependency_profile`,
11. `surface_fec_dependency_profile`.

The intentional change is ownership: runtime UDF and capability-shaped registry entries can carry workbook/session-owned function identifiers without requiring leaked or static IDs.

## Stable future-facing sources

1. Function list and lookup: `FunctionRegistry`.
2. Built-ins: `builtin_registry()`.
3. UDF mutation: `register_udf()` / `unregister_udf()`.
4. Capability projection: `CapabilityOverlay` / `with_capability_overlay()`.
5. Signature display: `FunctionEntry.display_signature.signature_display`.
6. Parameter labels: `FunctionEntry.display_signature.parameters`.
7. Arity: `FunctionEntry.meta.arity`.
8. Seam/status metadata: `FunctionEntry.registry_metadata`.

## Prohibited compatibility fallback

Do not reintroduce any downstream parser for `arity_shape_note` or host-local signature synthesis. OxFunc intentionally does not expose the old free-text arity channel in the runtime registry API.

## Requested OxFml action

Update any W068 landing code that type-assumes `FunctionMeta` or borrowed static function IDs. Treat `FunctionEntry.meta.function_id` as owned runtime data and borrow it as `&str` only at call sites that need a borrowed key.
