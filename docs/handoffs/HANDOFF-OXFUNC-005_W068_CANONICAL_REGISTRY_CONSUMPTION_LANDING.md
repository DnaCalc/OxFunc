# HANDOFF-OXFUNC-005 - W068 Canonical Registry Consumption Landing

Status: acknowledged
Direction: OxFml -> OxFunc
Source repo/workset: OxFml/W068
Target repo/workset: OxFunc/W091
Filed date: 2026-05-04
Acknowledged date: 2026-05-04
Upstream source: ../OxFml/docs/handoffs/HANDOFF-OXFUNC-005_W068_CANONICAL_REGISTRY_CONSUMPTION_LANDING.md
Related OxFunc handoffs: HO-FN-011, HO-FN-013

## OxFunc acknowledgement

OxFunc acknowledges that the OxFml W068 landing described in the upstream handoff matches the intended HO-FN-011 contract direction:

1. editor help consumes `oxfunc_core::registry`,
2. built-in help uses `builtin_registry()`,
3. host/UDF-mutated registries use `FunctionRegistry`,
4. capability tweaks use `CapabilityOverlay`,
5. signatures come from `FunctionEntry.display_signature.signature_display`,
6. argument labels come from ordered `ParameterDescriptor` rows,
7. arity bounds come from registry entry metadata,
8. unknown registry lookup returns no help packet,
9. `LibraryContextSnapshot` remains an overlay for availability, admission, provenance, and replay rather than a signature-truth source.

OxFunc also acknowledges the removed OxFml paths as correct future-facing cleanup:

1. no `LibraryContextSnapshotEntry.arity_shape_note`,
2. no editor `parse_arity_shape_note`,
3. no synthetic signature suffix generation,
4. no synthetic `argN` / `additional_args` argument help,
5. no fixture/export/import fields whose only purpose was free-text arity carriage.

## Required follow-up note

After HO-FN-011 was filed, OxFunc hardened the runtime registry entry shape so `FunctionEntry.meta` is now `RegistryFunctionMeta` with owned `function_id: String`, rather than the static built-in `FunctionMeta` type. The arity and semantic-class fields remain present on `FunctionEntry.meta`, but downstream consumers should not type-assume `FunctionMeta`.

This shape is documented for OxFml in `docs/handoffs/HO-FN-013_registry_function_meta_owned_runtime_shape.md`.

## Status report

execution_state: `acknowledged_with_api-shape_followup_filed`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. OxFml consumption of the owned `RegistryFunctionMeta` shape is a downstream follow-up.
2. DNA OneCalc consumption of the same owned runtime shape remains a downstream follow-up.
3. Function and parameter description corpus remains future work.
