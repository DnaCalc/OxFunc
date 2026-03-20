# OxFunc Library Context Snapshot Export V1

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
10. `canonical_surface_name`
11. `name_resolution_table_ref`
12. `semantic_trait_profile_ref`
13. `gating_profile_ref`
14. `version_marker`
15. `category`
16. `interesting`
17. `arity_min`
18. `arity_max`
19. `arg_preparation_profile`
20. `coercion_lift_profile`
21. `kernel_signature_class`
22. `determinism_class`
23. `volatility_class`
24. `host_interaction_class`
25. `thread_safety_class`
26. `fec_dependency_profile`
27. `surface_fec_dependency_profile`
28. `metadata_status`
29. `special_interface_kind`
30. `admission_interface_kind`
31. `preparation_owner`
32. `runtime_boundary_kind`
33. `arity_shape_note`
34. `interface_contract_ref`
35. `source_catalog_ref`

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
10. `canonical_surface_name`
   - canonical English surface name from the current local catalog, or current operator canonical name
11. `name_resolution_table_ref`
   - pointer to the current multilingual name table seed used for localized function resolution work, or the current operator-name placeholder ref
12. `semantic_trait_profile_ref`
   - current OxFunc-local profile-family ref for function-surface semantics/admission
13. `gating_profile_ref`
   - current OxFunc-local static gating family ref
14. `version_marker`
   - current support-harvest version marker when present
15. `category`
   - support-page category carried through from the canonical catalog
16. `interesting`
   - current planning-interest flag from the canonical catalog
17. `arity_min` / `arity_max`
   - first-pass arity exposure for OxFml parse/bind work
18. `arg_preparation_profile`
   - first-pass statement of whether arguments are expected values-only or refs-visible at the adapter seam
19. `coercion_lift_profile`
   - current OxFunc-local coercion/admission family indicator
20. `kernel_signature_class`
   - coarse kernel-shape classification
21. `determinism_class`
   - deterministic, time-dependent, pseudo-random, or external-event dependent
22. `volatility_class`
   - current recalc/invalidation posture
23. `host_interaction_class`
   - current host/session interaction class
24. `thread_safety_class`
   - current runtime thread-safety posture
25. `fec_dependency_profile`
   - current adapter-level dependency summary
26. `surface_fec_dependency_profile`
   - current surface pipeline dependency summary
27. `metadata_status`
   - current extraction status for the detailed profile columns:
     - `function_meta_extracted`
     - `catalog_only`
     - `doc_modeled`
28. `special_interface_kind`
   - first-pass signal that a row is seam-heavy rather than ordinary
29. `admission_interface_kind`
   - first-pass indication of whether the row is an ordinary call, helper-formation form, higher-order call, operator form, or host-subscription call
30. `preparation_owner`
   - first-pass indication of where preparation/formation responsibility mainly sits
31. `runtime_boundary_kind`
   - first-pass indication of the runtime seam OxFml should expect after preparation
32. `arity_shape_note`
   - free-form first-pass note for special argument-shape or helper/operator admission details
33. `interface_contract_ref`
   - current best contract/workset artifact to follow for seam-heavy rows
34. `source_catalog_ref`
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
   - use the detailed profile columns when `metadata_status = function_meta_extracted` or `doc_modeled`
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

## 6. Current Honest Limits
1. This export includes the full current `W45` non-`@` operator surface plus one explicitly modeled `FUNC.OP_IMPLICIT_INTERSECTION` row, not the full future operator universe.
2. Some seam-heavy rows such as `LET` and `LAMBDA` still have blank detailed profile columns; that currently means "follow `interface_contract_ref`", not "treat as ordinary default semantics".
3. The current export is generated from the current local tree and now states that explicitly via `source_commit_short`, `source_commit_full`, and `source_tree_state`; a `dirty` row set is still useful for bounded integration rounds, but it is not the same thing as a clean committed release artifact.
4. `semantic_trait_profile_ref` and `gating_profile_ref` are currently family refs, not fully dereferenceable per-row downstream contracts.
5. `admission_interface_kind`, `preparation_owner`, `runtime_boundary_kind`, and `arity_shape_note` are first-pass OxFunc guidance fields, not yet locked shared vocabulary.
6. This export does not itself inline localized names; it points to the current multilingual seed table.
7. This export does not carry runtime capability, provider availability, caller-context, or host-query payload facts.
8. The exact final shared field set and field names are still not locked cross-repo.

## 7. Authoritative Sources
Current authoritative source surfaces:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
2. `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv`
3. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V1.md`
4. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V2.md`
5. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`

## 8. Next Expected Refinements
1. widen operator coverage beyond the current exported operator set plus `OP_IMPLICIT_INTERSECTION`,
2. normalize direct detailed-profile fields for seam-heavy rows like `LET` and `LAMBDA`,
3. improve per-entry semantic/admission profile dereferenceability,
4. refine gating-profile projection beyond the current packet-wide default plus version-marker split,
5. add explicit export-reading examples if OxFml needs them,
6. adjust the first-pass seam-facing fields if OxFml wants a different split or naming.
