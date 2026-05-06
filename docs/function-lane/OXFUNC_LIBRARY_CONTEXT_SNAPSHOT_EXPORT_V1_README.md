# OxFunc Library Context Snapshot Export V1

Supersession note:
1. For current admission counts and outstanding-row truth, this readme is superseded by `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`.
2. This file remains the field-level description of the `V1` snapshot artifact, but it does not override the current `W051` reconciliation of supported, preview, deferred, hidden-backlog, and snapshot-stale rows.
3. For the current OxFunc-owned shared-interface freeze candidate over the seam-relevant non-deferred surface, use `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.
4. For the canonical runtime-mutable function registry direction, use `docs/function-lane/OXFUNC_CANONICAL_RUNTIME_FUNCTION_REGISTRY_CONTRACT.md` and `docs/worksets/W091_CANONICAL_RUNTIME_FUNCTION_REGISTRY.md`; this `V1` export is a projection artifact, not a second comprehensive function list.

## 1. Purpose
This is the first explicit OxFunc-local export artifact intended to serve as the external library-context snapshot for OxFml parse, bind, semantic planning, and replay correlation.

Artifact:
1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

This is a stabilization artifact, not a final cross-repo ABI.

## 2. Current Coverage
Current scope:
1. built-in worksheet functions
2. current exported evaluable operator surface plus one explicitly modeled special operator row
3. current-baseline OxFunc local canonical catalog plus operator rows
4. `534` total rows:
   - `511` functions
   - `23` operators

Current exclusions:
1. non-exported operator universe beyond the current exported operator set plus the explicitly modeled implicit-intersection row
2. externally registered functions
3. callable-value entries as direct catalog rows
4. runtime capability or provider/session state

## 3. Export Fields
Current fields:
1. `snapshot_id`
2. `snapshot_generation`
3. `source_commit_short`
4. `source_commit_full`
5. `source_tree_state`
6. `lane_id`
7. `entry_kind`
8. `registration_source_kind`
9. `surface_stable_id`
10. `xlcall_builtin_symbol`
11. `xlcall_builtin_code`
12. `canonical_surface_name`
13. `name_resolution_table_ref`
14. `semantic_trait_profile_ref`
15. `gating_profile_ref`
16. `version_marker`
17. `category`
18. `interesting`
19. `arity_min`
20. `arity_max`
21. `arg_preparation_profile`
22. `coercion_lift_profile`
23. `kernel_signature_class`
24. `determinism_class`
25. `volatility_class`
26. `host_interaction_class`
27. `thread_safety_class`
28. `fec_dependency_profile`
29. `surface_fec_dependency_profile`
30. `metadata_status`
31. `special_interface_kind`
32. `admission_interface_kind`
33. `preparation_owner`
34. `runtime_boundary_kind`
35. `arity_shape_note`
36. `interface_contract_ref`
37. `source_catalog_ref`

## 4. Field Meaning
1. `snapshot_id`
   - stable id for this exported snapshot family
2. `snapshot_generation`
   - generation date for this emitted export
3. `source_commit_short`
   - current repo commit that produced this snapshot export
4. `source_commit_full`
   - full current repo commit hash that produced this snapshot export
5. `source_tree_state`
   - `clean` or `dirty` tree state for the snapshot generation run
6. `lane_id`
   - current lane owner; fixed here as `oxfunc`
7. `entry_kind`
   - currently `built_in_function` or `built_in_operator`
8. `registration_source_kind`
   - first-pass statement of where the row comes from:
     - `built_in_catalog_function`
     - `built_in_operator_export`
     - `doc_modeled_operator`
9. `surface_stable_id`
   - current OxFunc-local stable function id candidate, emitted as `FUNC.<CANONICAL_NAME>`
10. `xlcall_builtin_symbol`
   - current `XLCALL.H` built-in `xlf*` symbol when a matched built-in code exists for the row
11. `xlcall_builtin_code`
   - current `XLCALL.H` numeric built-in function code when a matched built-in code exists for the row
12. `canonical_surface_name`
   - canonical English surface name from the current local catalog, or current operator canonical name
13. `name_resolution_table_ref`
   - pointer to the current multilingual name table seed used for localized function resolution work, or the current operator-name placeholder ref
14. `semantic_trait_profile_ref`
   - current OxFunc-local profile-family ref for function-surface semantics/admission
15. `gating_profile_ref`
   - current OxFunc-local static gating family ref
16. `version_marker`
   - current support-harvest version marker when present
17. `category`
   - support-page category carried through from the canonical catalog
18. `interesting`
   - current planning-interest flag from the canonical catalog
19. `arity_min` / `arity_max`
   - first-pass arity exposure for OxFml parse/bind work
20. `arg_preparation_profile`
   - first-pass statement of whether arguments are expected values-only or refs-visible at the adapter seam
21. `coercion_lift_profile`
   - current OxFunc-local coercion/admission family indicator
22. `kernel_signature_class`
   - coarse kernel-shape classification
23. `determinism_class`
   - deterministic, time-dependent, pseudo-random, or external-event dependent
24. `volatility_class`
   - current recalc/invalidation posture
25. `host_interaction_class`
   - current host/session interaction class
26. `thread_safety_class`
   - current runtime thread-safety posture
27. `fec_dependency_profile`
   - current adapter-level dependency summary
28. `surface_fec_dependency_profile`
   - current surface pipeline dependency summary
29. `metadata_status`
   - current extraction status for the detailed profile columns:
     - `function_meta_extracted`
     - `function_meta_curated`
     - `catalog_only`
     - `doc_modeled`
30. `special_interface_kind`
   - first-pass signal that a row is seam-heavy rather than ordinary
31. `admission_interface_kind`
   - first-pass indication of whether the row is an ordinary call, helper-formation form, higher-order call, operator form, or host-subscription call
32. `preparation_owner`
   - first-pass indication of where preparation/formation responsibility mainly sits
33. `runtime_boundary_kind`
   - first-pass indication of the runtime seam OxFml should expect after preparation
34. `arity_shape_note`
   - free-form first-pass note for special argument-shape or helper/operator admission details
35. `interface_contract_ref`
   - current best contract/workset artifact to follow for seam-heavy rows
36. `source_catalog_ref`
   - authoritative source row family for this export generation

## 5. Reading Guidance For OxFml
Current intended use:
1. parse/name recognition:
   - use `canonical_surface_name`
   - join to `name_resolution_table_ref` for localized names when the row is a function
2. bind:
   - use `surface_stable_id`
   - use `entry_kind`
   - use `gating_profile_ref`
   - use the detailed profile columns when `metadata_status = function_meta_extracted`, `function_meta_curated`, or `doc_modeled`
   - when `special_interface_kind <> ordinary`, also use:
     - `admission_interface_kind`
     - `preparation_owner`
     - `runtime_boundary_kind`
     - `arity_shape_note`
     - `interface_contract_ref`
3. semantic planning:
   - preserve `surface_stable_id`
   - preserve `semantic_trait_profile_ref`
   - preserve snapshot identity fields
   - preserve detailed profile fields where present
   - preserve `special_interface_kind`
   - preserve `interface_contract_ref`
4. replay/proving-host correlation:
   - preserve `snapshot_id`
   - preserve `snapshot_generation`
   - preserve `source_commit_short`
   - preserve `source_commit_full`
   - preserve `source_tree_state`
   - preserve `surface_stable_id`
   - preserve `xlcall_builtin_symbol`
   - preserve `xlcall_builtin_code`

## 6. Current Honest Limits
1. This export includes the full current `W45` non-`@` operator surface plus one explicitly modeled `FUNC.OP_IMPLICIT_INTERSECTION` row, not the full future operator universe.
2. Seam-heavy rows still require `interface_contract_ref` for their full shared-model reading even when the detailed profile columns are populated; the export remains guidance, not a self-sufficient seam contract.
3. The current export is generated from the current local tree and now states that explicitly via `source_commit_short`, `source_commit_full`, and `source_tree_state`; a `dirty` row set is still useful for bounded integration rounds, but it is not the same thing as a clean committed release artifact.
4. `semantic_trait_profile_ref` and `gating_profile_ref` are currently family refs, not fully dereferenceable per-row downstream contracts.
5. `admission_interface_kind`, `preparation_owner`, `runtime_boundary_kind`, and `arity_shape_note` are first-pass OxFunc guidance fields, not yet locked shared vocabulary.
6. This export does not itself inline localized names; it points to the current multilingual seed table.
7. This export does not carry runtime capability, provider availability, caller-context, or host-query payload facts.
8. The current shared freeze wording for seam-facing interpretation is now acknowledged cross-repo, but this CSV remains a `V1` stabilization artifact rather than a permanent runtime ABI.
9. The current `V1` export now carries the exact `W44` stale-row refresh:
   - `0` hidden backlog snapshot entries remain centralized in `W051` after `W068`,
   - the exact `114` documented-complete rows from the first-pass stale inventory have been refreshed out of false `catalog_only` state.
10. `W058` normalized the hidden ordinary backlog for execution purposes, and `W059` / `W060` / `W061` / `W062` / `W063` / `W064` / `W065` / `W066` / `W067` / `W068` have now removed the full `192` machine-clean execution rows; the published `V1` snapshot artifact now aligns to `0` current hidden snapshot entries and `0` current normalized execution rows.

Current built-in C API interop examples:
1. `FUNC.SUM`
2. `FUNC.CALL`
3. `FUNC.REGISTER.ID`
4. `FUNC.RTD`

Those rows now expose:
1. `xlcall_builtin_symbol`
2. `xlcall_builtin_code`
3. the OxFunc stable id on the same row,
4. seam refs through `interface_contract_ref`.

Current presentation-aware examples:
1. `FUNC.NOW`
2. `FUNC.TODAY`
3. `FUNC.HYPERLINK`

Those rows now expose:
1. extracted `FunctionMeta` profile columns,
2. `special_interface_kind = presentation_hinting_function`,
3. `runtime_boundary_kind = extended_value_with_presentation_hint`,
4. specific `interface_contract_ref` values back to the current function-slice contract or value-model note.

Current locale/profile/provider seam examples:
1. `FUNC.ASC`
2. `FUNC.DBCS`
3. `FUNC.JIS`
4. `FUNC.NUMBERVALUE`
5. `FUNC.TRANSLATE`

Those rows now expose:
1. curated detailed profile columns,
2. `special_interface_kind = width_conversion_host_profile` for `ASC` / `DBCS` / `JIS`,
3. `special_interface_kind = locale_default_profiled_parse` for `NUMBERVALUE`,
4. `special_interface_kind = provider_language_request` for `TRANSLATE`,
5. `runtime_boundary_kind = typed_host_width_conversion_mode` for the width-conversion family,
6. `runtime_boundary_kind = ordinary_eval_with_locale_defaults` for `NUMBERVALUE`,
7. `runtime_boundary_kind = host_provider_projection` for `TRANSLATE`,
8. direct `interface_contract_ref` pointers to the current `W034` / `W035` / `W036` contract notes.

## 7. Authoritative Sources
Current authoritative source surfaces:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
2. `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv`
3. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V1.md`
4. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V2.md`
5. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`

## 8. Downstream Consumer Guidance (DNA OneCalc And Other Hosts)

### 8.1 Reading This Export As A Downstream Consumer
Downstream consumers such as `DNA OneCalc` should read this export together with two companion documents:
1. `OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` - field stability classification, help/signature payload shape, and runtime direction,
2. `OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md` - admission categories, labeling rules, and honest seam-heavy row status.

### 8.2 Mandatory Overlay Reading
This export must be read together with `W050` and `W051`, not by itself:
1. `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md` - `17` explicitly deferred functions,
2. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md` - current authoritative non-deferred outstanding-row report,
3. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv` - now intentionally empty after `W068`,
4. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv` - now intentionally empty after `W068`,
5. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv` - original first-pass provenance inventory.

A row present in the snapshot export does not by itself mean the function is fully supported. The downstream consumer must read the snapshot together with `W050` and `W051` to determine the correct admission category. Current `W051` consumer-facing totals are:
1. `517` supported rows,
2. `0` preview rows,
3. `17` deferred rows,
4. `0` hidden non-deferred backlog snapshot entries,
5. `0` normalized hidden ordinary execution rows after `W068`,
6. the exact first-pass stale-row set still lives in `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv` as refresh provenance.

Current shared-interface reading:
1. no explicit `W051` preview row remains after `W014` current-phase closure,
2. there is no remaining hidden non-deferred snapshot-entry backlog in the current active non-deferred surface,
3. the current machine-clean execution split for the same backlog is `0` rows after `W068`,
4. the current OxFunc-owned shared-interface candidate is documented separately in `OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.

### 8.3 Stability Tiers
The downstream metadata contract classifies every export field into one of three stability tiers:
1. **Stable** - safe to treat as durable identity and structural facts,
2. **Usable-but-provisional** - safe to consume for display and planning, but vocabulary/coverage may evolve,
3. **Current-tree-hint-only** - informational pointers, not for hard coupling.

See `OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` Section 3 for the full classification table.

### 8.4 Help And Signature Payloads
No OxFunc-backed help or signature payload retrieval is frozen yet. The downstream metadata contract now defines both:
1. the preferred first OneCalc-facing payload shapes for function help, argument help, and signature help metadata, and
2. the first explicit `SemanticWitnessEntry` `V2` schema and stability tiers.

See `OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` Section 4.

### 8.4A First Mixed Witness Reading Rule
The first widened mixed `V2` tranche after the bounded `HLOOKUP` / `VLOOKUP`
seed is:
1. `FUNC.SUM`
2. `FUNC.IF`
3. `FUNC.XLOOKUP`
4. `FUNC.LET`
5. `FUNC.IMAGE`
6. `FUNC.HYPERLINK`
7. `FUNC.GROUPBY`
8. `FUNC.OP_IMPLICIT_INTERSECTION`

Current intentional exclusion:
1. `FUNC.RTD`
   - leave outside the first widened tranche until the retained `W043` seam
     lane is narrowed further for witness-facing rollout.

Current widened witness surfaces:
1. artifact:
   - `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_MIXED_TRANCHE_A.json`
2. generator:
   - `tools/w69-probe/run-w69-mixed-tranche-a-witness-generator.ps1`

Downstream reading rule:
1. join `V2` witness rows to `V1` export rows on `surface_stable_id`,
2. continue to read support status and deferred/not-complete overlay truth from
   `V1` plus `W050` / `W051`,
3. use witness rows only as semantic/help/provenance enrichment,
4. do not infer support solely from witness presence,
5. if a row has no witness payload yet, keep reading it from `V1` only.

Current widened artifact contents:
1. `FUNC.GROUPBY`
2. `FUNC.HYPERLINK`
3. `FUNC.IF`
4. `FUNC.IMAGE`
5. `FUNC.LET`
6. `FUNC.OP_IMPLICIT_INTERSECTION`
7. `FUNC.SUM`
8. `FUNC.XLOOKUP`

### 8.5 This Export Is A Stabilization Artifact
This export is the right pinned interchange artifact for bounded integration rounds, test pinning, and mismatch reporting. It is not a final cross-repo ABI. It also serves as the retained structural seed for `W069` witness generation. The preferred long-term direction is the runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` model described in `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`.

## 9. Next Expected Refinements
1. widen operator coverage beyond the current exported operator set plus `OP_IMPLICIT_INTERSECTION`,
2. normalize direct detailed-profile fields for seam-heavy rows like `LET` and `LAMBDA`,
3. improve per-entry semantic/admission profile dereferenceability,
4. refine gating-profile projection beyond the current packet-wide default plus version-marker split,
5. keep the exact `W44` stale-row refresh generation-based so later documented-complete rows do not drift back to false `catalog_only` state,
6. normalize the grouped-row names at the snapshot/catalog-source layer if a future export-generation pass wants the published artifact itself to stop using grouped entries,
7. add explicit export-reading examples if OxFml needs them,
8. adjust the first-pass seam-facing fields if OxFml wants a different split or naming,
9. populate structured help/signature payload fields when available (see `OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` Section 4).

## 10. Preferred Long-Term Runtime Direction
Current OxFunc reading:
1. this CSV export is the right pinned interchange artifact for bounded integration rounds, test pinning, and mismatch reporting,
2. it should also be treated as the retained `V1` seed for `W069` witness generation rather than as the final downstream destination surface,
3. the comprehensive function list and signature source should be the OxFunc-owned runtime registry defined by `W091`,
4. the preferred long-term implementation seam should be a runtime-ingested:
   - `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
5. function registration or removal should produce explicit new snapshot generations rather than mutating downstream state invisibly,
6. capability overlays should project availability over registry entries rather than deleting or rewriting base function truth,
7. OxFml should therefore treat CSV ingestion as a current integration mechanism and witness-generation seed, not as the desired permanent runtime coupling.
