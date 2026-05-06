# HO-FN-012 - DNA OneCalc consumption of OxFunc canonical runtime function registry

Status: ready_for_downstream_consumption
Direction: OxFunc -> DNA OneCalc
Origin workset: W091_CANONICAL_RUNTIME_FUNCTION_REGISTRY
Date: 2026-05-03

## Purpose

DNA OneCalc must stop maintaining a comprehensive built-in function list, arity table, or parameter-shape table. OxFunc now exposes the canonical runtime function registry surface for built-ins, UDF registrations, and capability overlays.

## Required downstream behavior

- Use `oxfunc_core::registry::builtin_registry()` for the canonical built-in snapshot.
- Use `FunctionRegistry::built_ins()` when the host needs a mutable runtime registry instance.
- Enumerate functions with `FunctionRegistry::iter()`; do not materialize a separate authoritative list in DNA OneCalc.
- Resolve popup/help rows from `FunctionEntry.surface_name`, `FunctionEntry.display_signature.signature_display`, and `FunctionEntry.display_signature.parameters`.
- Register and unregister UDFs through `FunctionRegistry::register_udf()` and `FunctionRegistry::unregister_udf()`.
- Apply workbook/session capability changes through `CapabilityOverlay` and `FunctionRegistry::with_capability_overlay()`.
- Treat capability-projected unavailability as presentation/evaluation availability, not as deletion from the canonical registry.

## API fields DNA OneCalc should consume

- `FunctionEntry.meta.function_id`: stable OxFunc function identifier, for example `FUNC.NOW`.
- `FunctionEntry.surface_name`: worksheet-facing display name, for example `NOW`.
- `FunctionEntry.meta.arity.min` and `FunctionEntry.meta.arity.max`: canonical arity bounds.
- `FunctionEntry.display_signature.signature_display`: display signature, for example `NOW()`.
- `FunctionEntry.display_signature.parameters`: ordered parameter descriptors with `name`, `optional`, and `repeats`.
- `CapabilityScopedFunctionEntry.availability`: overlay result for capability-scoped UI or evaluator admission.

## Hard prohibition

DNA OneCalc must not keep a second comprehensive function catalog such as `DEFAULT_FUNCTION_NAMES`, host-local arity maps, or host-local parameter-name tables. Small caches derived from OxFunc at runtime are allowed only when invalidated by registry/UDF/capability changes and never treated as source of truth.

## Regression target from the inbound report

The `NOW` popup must be derived from OxFunc as zero-argument metadata: `NOW()`, with no generated `arg1`, `arg2`, `arg3`, or `additional_args` fallback.

## OxFunc evidence

- `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1266 passed, 1 ignored.
- `cargo check --manifest-path crates/oxfunc_core/Cargo.toml --target wasm32-unknown-unknown --lib`: passed.
- `tools/xll-addin/oxfunc_xll/export_specs.csv` regenerated from OxFunc core after named-parameter export wiring.

## Downstream migration checklist

- Remove any comprehensive DNA OneCalc built-in function list.
- Replace function autocomplete/help source with OxFunc registry iteration.
- Replace arity/parameter popup source with OxFunc `FunctionEntry.display_signature` and `FunctionMeta.arity`.
- Route UDF registration changes into an OxFunc `FunctionRegistry` instance.
- Route capability/session overlays through OxFunc `CapabilityOverlay`.
- Add a regression check that `NOW` surfaces as `NOW()`.

## 2026-05-03 metadata preservation update

DNA OneCalc consumers that previously received OxFml `LibraryContextSnapshotEntry` fields should read equivalent OxFunc-owned fields from `FunctionEntry.registry_metadata`. Use `display_signature` and `ParameterDescriptor` for signatures and argument labels; the old `arity_shape_note` channel is intentionally not exposed.

## 2026-05-03 future-facing registry cleanup

DNA OneCalc must not consume `arity_shape_note`; OxFunc does not expose it in the runtime registry API. Use `display_signature`, `ParameterDescriptor`, and `FunctionMeta.arity`.


## 2026-05-04 future-facing registry hardening

DNA OneCalc should treat OxFunc registry entries as runtime-owned data. UDF function identifiers are no longer constrained to static built-in metadata shape at the registry boundary. Do not mirror V1/W44 snapshot rows as a host-local catalog.
