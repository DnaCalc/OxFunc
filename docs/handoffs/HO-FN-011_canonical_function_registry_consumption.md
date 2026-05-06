# HO-FN-011 - Canonical Function Registry Consumption

Direction: `OxFunc->OxFml`

Status: `filed`

Filed date: `2026-05-03`

Source workset:
1. `W091`

Target:
1. OxFml formula-language, runtime library-context, and editor-help consumers.

## 1. Scope

This handoff tells OxFml that OxFunc is promoting a canonical runtime function
registry lane and that OxFml should migrate away from host-filled string arity
or signature channels as source truth.

## 2. Proposed Seam Rule

OxFml should consume function identity, arity, display signature, and parameter
descriptor truth from OxFunc registry entries or OxFunc-generated immutable
registry snapshots.

OxFml should not rely on downstream hosts to populate comprehensive function
lists or free-text `arity_shape_note` values for ordinary function help and
signature display.

## 3. Intended Shared Shape

Minimum current-phase shared concepts:
1. `FunctionEntry`
   - `FunctionMeta`
   - canonical surface name
   - display signature
   - optional help summary/detail
   - source classification
2. `SignatureForm`
   - ordered parameter descriptors
   - trailing-repeat flag
3. `ParameterDescriptor`
   - name
   - optional flag
   - repeats flag
   - optional parameter help
4. capability-scoped registry view
   - availability/gating projection over registry entries

## 4. Snapshot And Runtime Consequences

OxFunc's `V1` CSV remains a projection and integration artifact.

`W049` runtime `LibraryContextProvider` / immutable
`LibraryContextSnapshot` remains the runtime carrier direction, but the
snapshot source for comprehensive function truth should be the OxFunc registry,
not a host-maintained list.

`W069` / `W071` witness payloads remain enrichment attached to registry-backed
snapshot entries. They are not a second catalog.

## 5. Requested OxFml Actions

1. Confirm that OxFml can switch function lookup and editor-help surfaces to
   OxFunc registry entries or registry-derived snapshots.
2. Identify any OxFml-only fields needed on `FunctionEntry` before the first
   registry-backed consumer migration.
3. Stop treating host-filled `arity_shape_note` as the source of ordinary
   signature truth once the registry surface is available.
4. Preserve capability/provider unavailable states as a registry view concern
   rather than deleting entries or substituting host-local lists.
5. Keep UDF registration as runtime registry mutation rather than a permanent
   host-owned function-list fork.

## 6. Current Status

execution_state: `in_progress`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

open_lanes:
1. OxFunc registry implementation,
2. OxFml acknowledgement,
3. consumer migration design,
4. validation evidence after the registry API exists.

## 2026-05-03 W091 API update

OxFunc exposes the canonical runtime function registry through `oxfunc_core::registry`. OxFml should consume this API rather than maintaining a comprehensive parser/evaluator-local function list. Required surfaces are `builtin_registry()`, `FunctionRegistry::iter()`, `lookup_by_surface_name()`, `lookup_by_id()`, `register_udf()`, `unregister_udf()`, `CapabilityOverlay`, and `with_capability_overlay()`.

The `NOW` regression target from DNA OneCalc is covered by OxFunc registry metadata: `NOW()` is zero-argument and must not be represented downstream as a variadic `argN` shape.

## 2026-05-03 metadata preservation update

OxFml migration should map existing `LibraryContextSnapshotEntry` seam/status fields from `FunctionEntry.registry_metadata` rather than retaining a separate function registry. The old `arity_shape_note` channel is intentionally not exposed and must not drive function help signatures.

## 2026-05-03 future-facing registry cleanup

OxFml must not preserve or parse `arity_shape_note` as a function-help or arity channel. The OxFunc registry exposes the future-facing source through `display_signature`, `ParameterDescriptor`, and `FunctionMeta.arity` only.


## 2026-05-04 future-facing registry hardening

OxFml migration should target owned runtime registry entries (`RegistryFunctionMeta`) and validated registry construction (`FunctionRegistry::try_from_entries`). OxFml should not preserve a separate unchecked function-list constructor or a V1/W44-derived registry source.

## 2026-05-04 OxFml W068 landing acknowledgement

OxFml W068 landing was acknowledged via `docs/handoffs/HANDOFF-OXFUNC-005_W068_CANONICAL_REGISTRY_CONSUMPTION_LANDING.md`. OxFunc agrees that HO-FN-011 consumption matches the intended contract, with the narrow follow-up that `FunctionEntry.meta` is now owned `RegistryFunctionMeta` rather than built-in-only static `FunctionMeta`.
