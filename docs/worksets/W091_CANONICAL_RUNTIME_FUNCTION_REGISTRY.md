# WORKSET - Canonical Runtime Function Registry (W091)

## 1. Purpose

Own the OxFunc-side correction for a canonical, runtime-mutable function
registry so DNA Calc has one comprehensive function list and signature source.

This workset responds to the DNA OneCalc handoff:
`../DnaOneCalc/docs/HANDOFF_OXFUNC_CANONICAL_FUNCTION_REGISTRY.md`.

## 2. Problem Statement

The current system still allows downstream consumers to build or supply
function lists and arity/signature hints outside OxFunc.

Observed symptom:
1. DNA OneCalc displayed `NOW(*arg1, arg2, arg3, additional_args)`.
2. `NOW` is a zero-argument function in OxFunc metadata.
3. The wrong display came from a host-built `LibraryContextSnapshot` using a
   generic `arity_shape_note` string rather than querying OxFunc registry truth.

Architectural issue:
1. OxFunc has per-function metadata and catalog accessors.
2. OxFunc does not yet expose a first-class registry API for iteration,
   lookup, parameter descriptors, runtime UDF registration, and capability
   overlays.
3. Hosts and OxFml can therefore drift into duplicate function-list ownership.

## 3. Scope

This workset owns:
1. crate-public `oxfunc_core::registry` API design and implementation,
2. `FunctionEntry` / `SignatureForm` / `ParameterDescriptor` runtime shape,
3. built-in registry population from the linked OxFunc function catalog,
4. real parameter descriptors for every linked built-in function,
5. UDF registration and unregistration semantics,
6. capability overlay view semantics,
7. migration guidance for OxFml, DNA OneCalc, and other consumers,
8. reconciliation of `V1` CSV, `W049` runtime provider, and `V2` witness
   projections so none become a second comprehensive function list.

## 4. Out Of Scope

This workset does not own:
1. formula grammar, parse, or bind implementation in OxFml,
2. DNA OneCalc UI implementation changes,
3. OxReplay or OxIde consumer migrations,
4. final localized help text coverage,
5. full Excel locale/version sweeps for every function signature,
6. external proprietary source discovery.

## 5. Dependencies

Depends on:
1. `W070`
2. `W044`
3. `W049`

Related upstream and downstream inputs:
1. DNA OneCalc handoff:
   `../DnaOneCalc/docs/HANDOFF_OXFUNC_CANONICAL_FUNCTION_REGISTRY.md`
2. OxFml current inbound observation ledger:
   `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. OxFml-side companion note referenced by DNA OneCalc:
   `../DnaOneCalc/docs/HANDOFF_OXFML_FUNCTION_HELP_FROM_OXFUNC_REGISTRY.md`

## 6. Parent Doctrine And Spec Surfaces

1. `CHARTER.md`
2. `OPERATIONS.md`
3. `docs/function-lane/OXFUNC_CANONICAL_RUNTIME_FUNCTION_REGISTRY_CONTRACT.md`
4. `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`
5. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
6. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
7. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
8. `docs/handoffs/HANDOFF-OXFUNC-004_canonical_runtime_function_registry.md`
9. `docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md`

## 7. Initial Epic Lanes

1. registry API and data model,
2. built-in catalog wiring and drift detection,
3. parameter descriptor population,
4. UDF registration and unregister semantics,
5. capability overlay view semantics,
6. downstream and OxFml migration handoff,
7. wasm and host validation floor,
8. truth-surface reconciliation for `V1`, `W049`, `W069`, and `W071`.

## 8. Closure Condition

The workset may only close when:
1. `oxfunc_core::registry` is public and documented,
2. every linked built-in function has non-synthetic signature descriptors
   consistent with its arity,
3. UDF registration and unregistration round-trip with explicit collision
   handling,
4. capability overlay views can express host/provider availability without
   mutating base registry truth,
5. OxFunc-local consumers use registry truth rather than a separate
   comprehensive list,
6. OxFml-facing migration has a filed handoff and acknowledged consumer plan,
7. DNA OneCalc-facing migration has an acknowledged consumer plan,
8. host and wasm validation evidence has been recorded,
9. no OxFunc document still describes the `V1` CSV or witness payloads as the
   owning comprehensive function list.

## 9. Current Status

execution_state: `in_progress`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

open_lanes:
1. registry API implementation,
2. parameter descriptor corpus,
3. UDF registration,
4. capability overlays,
5. OxFml migration,
6. DNA OneCalc migration,
7. wasm validation,
8. truth-surface reconciliation.

reviewed inbound observations:
1. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` was reviewed for this workset.
2. The relevant unresolved observation is the current OxFml direction toward a
   runtime library-context interface rather than long-term build-time catalog
   ingestion.
3. `W091` responds by making the runtime library-context source a registry
   projection owned by OxFunc rather than a downstream-maintained catalog.

## 2026-05-03 W091 execution evidence

OxFunc now exposes the canonical runtime function registry from `oxfunc_core::registry`:

- `builtin_registry()` for the shared built-in registry snapshot.
- `FunctionRegistry::built_ins()` for mutable runtime registries.
- `FunctionRegistry::iter()`, `lookup_by_surface_name()`, and `lookup_by_id()` for canonical discovery.
- `register_udf()` and `unregister_udf()` for runtime UDF mutations.
- `CapabilityOverlay` and `with_capability_overlay()` for workbook/session capability projections.

Validation evidence:

- `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1266 passed, 1 ignored.
- `cargo check --manifest-path crates/oxfunc_core/Cargo.toml --target wasm32-unknown-unknown --lib`: passed.
- XLL export specs regenerated from OxFunc core after named-parameter wiring.

Downstream notes:

- OxFml consumption note: `docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md`.
- DNA OneCalc consumption note: `docs/handoffs/HO-FN-012_dnaonecalc_registry_consumption.md`.

## 2026-05-03 OxFunc-local gate status and audit

execution_state: `local_target_satisfied_downstream_ack_pending`

scope_completeness: `scope_partial`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. OxFml downstream acknowledgement and consumer migration are outside OxFunc-local write authority.
2. DNA OneCalc downstream acknowledgement and UI migration are outside OxFunc-local write authority.
3. Full workspace integration test remains affected by the pre-existing OxFml dev-test import mismatch for `oxfml_core::format::current_excel_host_context`.

Pre-Closure Verification Checklist for the OxFunc-local target:
1. Function contract rows complete and promoted for all in-scope functions? Yes: W091 does not promote per-function semantic rows; registry descriptors cover every linked built-in function.
2. Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? Yes: no new per-function semantic substrate obligation was introduced by the registry metadata seam.
3. Rust implementation and required tests pass for all in-scope functions? Yes for OxFunc-local registry scope: `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib` passed.
4. At least one deterministic replay artifact exists per in-scope function behavior? Yes for this metadata seam: deterministic registry unit tests cover catalog coverage, arity/descriptor consistency, `NOW()`, UDF mutation, collision behavior, and capability overlay projection.
5. Evidence links complete and reproducible? Yes: commands and handoff files are recorded in this workset and the feature worklist.
6. Version scope explicit on both axes? Yes: W091 is registry ownership and runtime metadata infrastructure; locale/version signature sweeps remain explicitly out of scope.
7. Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? Yes: no new function-semantic discrepancy was introduced; `NOW()` zero-argument metadata is preserved from OxFunc metadata.
8. XLL verification-seam limitations documented where material? Yes: XLL export specs were regenerated from OxFunc core; no new XLL host limitation materially qualifies the registry claim.
9. Cross-repo impact assessed and handoff filed if evaluator-facing clauses affected? Yes: HO-FN-011 for OxFml and HO-FN-012 for DNA OneCalc are filed.
10. No known semantic gap remains in declared OxFunc-local scope? Yes for the registry ownership/mutation surface; downstream acknowledgement remains an integration lane.
11. Completion language audit passed? Yes: status remains three-axis and downstream lanes are reported as partial.
12. `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? Yes.
13. Execution-state blocker surface updated? Yes: W091 `.beads/` execution beads were closed with evidence; downstream acknowledgement is reported as an open integration lane rather than an OxFunc-local blocker.

Completion Claim Self-Audit for the OxFunc-local target:
1. Scope re-read: Pass for the user-requested OxFunc-local migration target; W091's downstream acknowledgement clauses remain outside this repo's write authority and are listed as open integration lanes.
2. Gate criteria re-read: Pass for OxFunc-local gates: registry API, built-in descriptors, UDF registration, capability overlay, local consumers/export specs, handoff notes, host/lib test, and wasm check have evidence.
3. Silent scope reduction check: Pass; the report explicitly separates OxFunc-local target satisfaction from downstream acknowledgement/migration.
4. Looks-done-but-is-not pattern check: Pass; registry code is exercised by unit tests, handoffs are not treated as downstream completion, and no placeholder implementation is reported as implementation.
5. Result: OxFunc-local target satisfied; W091 full cross-repo integration remains partial until sibling repos acknowledge and migrate.

## 2026-05-03 registry metadata preservation follow-up

OxFunc now carries the V1 library-context per-function seam/status fields directly on `FunctionEntry.registry_metadata`. This is the OxFunc-owned replacement for OxFml carrying those fields as part of a registry-like `LibraryContextSnapshotEntry` catalog.

Preserved fields include:
1. `lane_id`
2. `entry_kind`
3. `registration_source_kind`
4. `surface_stable_id`
5. `xlcall_builtin_symbol`
6. `xlcall_builtin_code`
7. `canonical_surface_name`
8. `name_resolution_table_ref`
9. `semantic_trait_profile_ref`
10. `gating_profile_ref`
11. `version_marker`
12. `category`
13. `interesting`
14. `metadata_status`
15. `special_interface_kind`
16. `admission_interface_kind`
17. `preparation_owner`
18. `runtime_boundary_kind`
19. `interface_contract_ref`
20. `source_catalog_ref`



The old `arity_shape_note` channel is intentionally not present in the runtime registry API. Consumers must use `display_signature` and ordered `ParameterDescriptor` rows for arity, parameter names, optionality, and repeat shape.

Evidence:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib registry`: passed, 8 passed.

## 2026-05-03 future-facing registry cleanup

OxFunc intentionally does not expose the old free-text `arity_shape_note` channel in the runtime registry API. The future-facing signature source is only `FunctionEntry.display_signature`, `FunctionEntry.display_signature.parameters`, and `FunctionMeta.arity`.


## 2026-05-04 future-facing registry hardening

OxFunc registry hardening changes:
1. `FunctionEntry.meta` now uses owned `RegistryFunctionMeta`, so runtime UDF entries can carry workbook/session-owned function identifiers instead of requiring `FunctionMeta.function_id: &'static str`.
2. unchecked `FunctionRegistry::from_entries` has been replaced by validating `FunctionRegistry::try_from_entries`.
3. `registry_context_seed.rs` is generated by `tools/generate-registry-context-seed.ps1`, which expands combined V1 rows such as `FUNC.LEFT, LEFTB` into per-runtime function IDs.
4. registry tests now require each built-in entry's `registry_metadata.surface_stable_id` to match its runtime function ID.
5. `tools/w44-probe/generate-w44-library-context-snapshot.ps1` is guarded as legacy-only and cannot be run accidentally as a future source of registry truth.
6. private combined signature-seed rows were removed; runtime signature lookup is keyed by individual function IDs.

Future-facing source rule:
1. OxFunc runtime registry entries own function identity, signature shape, parameter descriptors, capability overlays, UDF registration state, and per-function seam/status metadata.
2. V1 CSV and W44 generator artifacts are historical inputs only, not future authoritative registries.

## 2026-05-04 OxFml W068 landing acknowledgement

OxFunc processed `../OxFml/docs/handoffs/HANDOFF-OXFUNC-005_W068_CANONICAL_REGISTRY_CONSUMPTION_LANDING.md`.

Result:
1. OxFunc acknowledges OxFml W068 landed the intended HO-FN-011 registry-consumption direction.
2. OxFunc agrees with removal of `arity_shape_note`, synthetic signature suffixes, `argN`, and `additional_args` from OxFml editor-help paths.
3. OxFunc filed `docs/handoffs/HO-FN-013_registry_function_meta_owned_runtime_shape.md` because OxFunc has since hardened `FunctionEntry.meta` to owned `RegistryFunctionMeta` for runtime/UDF-safe identifiers.

execution_state: `acknowledged_with_api-shape_followup_filed`

scope_completeness: `scope_complete`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. OxFml W068 follow-up for owned `RegistryFunctionMeta` if its landed code type-assumed `FunctionMeta`.
2. DNA OneCalc registry consumption remains downstream.
3. Function and parameter description corpus remains future work.

## 2026-05-06 current-reference metadata fill

execution_state: `current_reference_metadata_seeded`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

Current-reference sources used:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
2. `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json`
3. `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_MIXED_TRANCHE_A.json`

Registry metadata now seeded from those local sources:
1. `FunctionEntry.short_description` is populated from the current English
   baseline catalog where that catalog carries a description.
2. `FunctionEntry.long_description` is populated only from the bounded rich
   `V2` witness seed rows whose detail text is consumer-facing rather than
   generic tranche scaffold prose.
3. `ParameterDescriptor.short_description` is populated from bounded rich
   `V2` `arg_specs.arg_behavior_note` rows when the witness argument name
   matches the registry parameter name at the same position.
4. Generic tranche seed prose from W69/W71 artifacts is intentionally not
   imported as user-facing help text.
5. The W28 localized-name library remains a name/category/description
   localization corpus, not an English parameter-description corpus.

Generated artifact:
1. `crates/oxfunc_core/src/registry_help_seed.rs`
2. generator: `tools/generate-registry-help-seed.ps1`
3. generation result for this pass: `510` registry-help seeds, `10`
   long-description seeds, and `33` parameter-help seeds across `10`
   functions.

Outstanding items to research or author in a later run:
1. authoritative English per-parameter descriptions for all linked built-ins
   with arguments,
2. remaining English parameter-name/signature placeholder rows in
   `registry_signature_seed.rs`,
3. long-form English function help, remarks, examples, and edge-case notes
   beyond the single catalog summary column,
4. parameter type hints, units, and accepted value sets where Excel documents
   or empirical replay can support them,
5. localized parameter names and localized parameter/help descriptions,
6. locale-specific function-help differences beyond localized function names,
7. Excel channel/version and workbook-compatibility variation for signatures,
   names, and help wording,
8. clean-room provenance labels for any later imported documentation corpus,
9. generated guard coverage that distinguishes real metadata from placeholder
   signatures and generic tranche scaffold text.

No validation command was run for this metadata-fill pass.
