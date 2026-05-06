# OxFunc Canonical Runtime Function Registry Contract

Status: `active_contract_seed`
Date: 2026-05-03

## 1. Purpose

Define the OxFunc-owned direction for the only comprehensive function registry
used by DNA Calc consumers.

This contract seed responds to the DNA OneCalc handoff:
`../DnaOneCalc/docs/HANDOFF_OXFUNC_CANONICAL_FUNCTION_REGISTRY.md`.

## 2. Core Rule

OxFunc owns the comprehensive function registry for built-in functions,
operators admitted as callable surfaces, and runtime-registered UDF entries.

No other lane, host, replay tool, IDE, or downstream consumer should maintain a
separate comprehensive function list, arity table, or parameter-signature table.

Allowed downstream projections:
1. immutable snapshots emitted by OxFunc,
2. witness payloads keyed to OxFunc registry identities,
3. capability-scoped registry views,
4. host-local caches generated from an OxFunc registry snapshot.

Forbidden downstream projections:
1. hand-authored default function-name lists,
2. string-only arity or signature paraphrases used as source truth,
3. independently maintained comprehensive function tables,
4. synthesized `arg1`, `arg2`, or variadic fallbacks where OxFunc should carry
   real parameter descriptors.

## 3. Registry Ownership Shape

The intended Rust surface is a crate-public `oxfunc_core::registry` module.

Required first concepts:
1. `FunctionRegistry`
   - iterable catalog view,
   - lookup by canonical surface name,
   - lookup by stable function id,
   - UDF registration and unregistration,
   - capability-overlay projection.
2. `FunctionEntry`
   - execution metadata from `FunctionMeta`,
   - canonical surface name,
   - display signature,
   - optional short and long help text,
   - source classification.
3. `SignatureForm`
   - ordered parameter descriptors,
   - trailing-repeat flag for variadic forms.
4. `ParameterDescriptor`
   - canonical parameter name,
   - required or optional state,
   - repeat classification,
   - optional short parameter help.
5. `CapabilityOverlay`
   - function availability projection for a run or host profile,
   - no mutation of the underlying registry.

## 4. Runtime Mutability Rule

Runtime mutability has two separate lanes:
1. UDF registration changes the registry entry set.
2. capability overlays project availability over the entry set.

UDF registration must not be modeled as a host-owned completion list or a
synthetic `LibraryContextSnapshotEntry` row detached from OxFunc registry truth.

Capability changes must not delete or rewrite base registry entries. They should
produce a scoped view that can state whether a function is available, gated, or
rejected for the current host capability profile.

## 5. Relationship To Existing OxFunc Artifacts

`OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` remains a pinned projection and
integration artifact. It is not the final registry owner and must not be treated
as a second catalog.

`W049` remains the runtime provider and immutable snapshot model anchor.
`W091` owns the new registry API and the migration from catalog projection to
runtime-mutable source truth.

`W069` / `W071` witness payloads remain enrichment keyed by OxFunc registry
identity plus snapshot generation. Witness rows do not become a parallel
function list.

## 6. Parameter Descriptor Rule

Every built-in function linked into `oxfunc_core` must carry real parameter
descriptors before the registry lane can close.

Minimum descriptor requirements:
1. canonical parameter names,
2. required versus optional state,
3. variadic trailing-repeat state where applicable,
4. consistency with `FunctionMeta.arity`.

Fallback parameter synthesis such as `arg1`, `arg2`, or a generic variadic label
is not an acceptable final state for linked built-ins.

## 7. Downstream Consumer Rule

Downstream consumers should obtain completion, arity, signature-help, and
parameter-help data by querying an OxFunc registry or an OxFunc-generated
snapshot derived from that registry.

Current consumers affected by this rule include:
1. OxFml parser, binder, semantic planner, and editor-help surfaces,
2. DNA OneCalc completion and signature-help surfaces,
3. future OxIde language services,
4. OxReplay and host tooling that need function identity or signatures,
5. OxCalc and OxVba host integration surfaces that consume function metadata.

## 8. OxFml Seam Consequence

OxFml should stop consuming arity or parameter-shape truth through a host-filled
free-text `arity_shape_note` channel.

The intended OxFml direction is:
1. function recognition and lookup read OxFunc registry entries,
2. signature help reads `FunctionEntry.display_signature`,
3. host-specific availability reads a capability-scoped registry view,
4. registered-external and UDF entries are represented as OxFunc registry truth
   or OxFunc-owned registry snapshots rather than host-maintained function
   mirrors.

The outbound OxFunc handoff for this seam is:
`docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md`.

## 9. Workset Ownership

Execution owner:
1. `docs/worksets/W091_CANONICAL_RUNTIME_FUNCTION_REGISTRY.md`

Live execution state:
1. `.beads/` through `br`

Primary source input:
1. `../DnaOneCalc/docs/HANDOFF_OXFUNC_CANONICAL_FUNCTION_REGISTRY.md`

Related current OxFunc surfaces:
1. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
2. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
3. `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`
4. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
5. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 10. Current Status

execution_state: `in_progress`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

open_lanes:
1. implement crate-public registry API,
2. populate real parameter descriptors for every linked built-in function,
3. implement UDF registration and unregistration,
4. implement capability overlay views,
5. migrate OxFml and host consumers away from duplicate lists and string arity
   channels,
6. add required host and wasm validation evidence.

## 2026-05-03 implementation evidence

The first OxFunc-local registry surface is available in `oxfunc_core::registry`. It centralizes built-in discovery, runtime UDF registration, capability overlays, canonical lookup, and ordered parameter descriptors. The registry derives built-in entries from OxFunc's XLL/export catalog and signature seed corpus, then normalizes required/optional/repeat metadata against `FunctionMeta.arity` so downstream consumers do not invent fallback arity or `argN` names.

Evidence:

- `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1266 passed, 1 ignored.
- `cargo check --manifest-path crates/oxfunc_core/Cargo.toml --target wasm32-unknown-unknown --lib`: passed.
- `tools/xll-addin/oxfunc_xll/export_specs.csv` regenerated from OxFunc core.

## 2026-05-03 registry metadata preservation follow-up

`FunctionEntry.registry_metadata` now exposes the V1 library-context per-function seam/status fields from OxFunc registry entries. This prevents OxFml from retaining a registry-like catalog solely to preserve `LibraryContextSnapshotEntry` metadata.

The old `arity_shape_note` value is intentionally not exposed in the runtime registry API. Consumers must use `FunctionEntry.display_signature`, ordered `ParameterDescriptor` rows, and `FunctionMeta.arity`.

## 2026-05-03 future-facing registry cleanup

The runtime registry API intentionally omits the old free-text `arity_shape_note` channel. Consumers must use `FunctionEntry.display_signature`, ordered `ParameterDescriptor` rows, and `FunctionMeta.arity`.


## 2026-05-04 future-facing registry hardening

The future-facing registry entry shape uses owned `RegistryFunctionMeta` rather than borrowed static built-in metadata. This allows runtime UDF registrations and capability-shaped catalogs to carry workbook/session-owned function identifiers. `FunctionRegistry::try_from_entries` validates externally supplied entry sets; callers should not construct unchecked comprehensive registries.

`tools/generate-registry-context-seed.ps1` regenerates the OxFunc-owned registry metadata seed from the historical V1 snapshot while normalizing combined rows into individual runtime function IDs. The W44 V1 snapshot generator is guarded as legacy-only and is not a future source of registry truth.
