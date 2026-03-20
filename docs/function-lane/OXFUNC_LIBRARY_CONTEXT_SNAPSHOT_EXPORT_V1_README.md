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
3. `lane_id`
4. `entry_kind`
5. `surface_stable_id`
6. `canonical_surface_name`
7. `name_resolution_table_ref`
8. `semantic_trait_profile_ref`
9. `gating_profile_ref`
10. `version_marker`
11. `category`
12. `interesting`
13. `arity_min`
14. `arity_max`
15. `arg_preparation_profile`
16. `coercion_lift_profile`
17. `kernel_signature_class`
18. `determinism_class`
19. `volatility_class`
20. `host_interaction_class`
21. `thread_safety_class`
22. `fec_dependency_profile`
23. `surface_fec_dependency_profile`
24. `metadata_status`
25. `special_interface_kind`
26. `admission_interface_kind`
27. `preparation_owner`
28. `runtime_boundary_kind`
29. `arity_shape_note`
30. `interface_contract_ref`
31. `source_catalog_ref`

## 4. Field Meaning
1. `snapshot_id`
   - stable id for this exported snapshot family
2. `snapshot_generation`
   - generation date for this emitted export
3. `lane_id`
   - current lane owner; fixed here as `oxfunc`
4. `entry_kind`
   - currently `built_in_function` or `built_in_operator`
5. `surface_stable_id`
   - current OxFunc-local stable function id candidate, emitted as `FUNC.<CANONICAL_NAME>`
6. `canonical_surface_name`
   - canonical English surface name from the current local catalog, or current operator canonical name
7. `name_resolution_table_ref`
   - pointer to the current multilingual name table seed used for localized function resolution work, or the current operator-name placeholder ref
8. `semantic_trait_profile_ref`
   - current OxFunc-local profile-family ref for function-surface semantics/admission
9. `gating_profile_ref`
   - current OxFunc-local static gating family ref
10. `version_marker`
   - current support-harvest version marker when present
11. `category`
   - support-page category carried through from the canonical catalog
12. `interesting`
   - current planning-interest flag from the canonical catalog
13. `arity_min` / `arity_max`
   - first-pass arity exposure for OxFml parse/bind work
14. `arg_preparation_profile`
   - first-pass statement of whether arguments are expected values-only or refs-visible at the adapter seam
15. `coercion_lift_profile`
   - current OxFunc-local coercion/admission family indicator
16. `kernel_signature_class`
   - coarse kernel-shape classification
17. `determinism_class`
   - deterministic, time-dependent, pseudo-random, or external-event dependent
18. `volatility_class`
   - current recalc/invalidation posture
19. `host_interaction_class`
   - current host/session interaction class
20. `thread_safety_class`
   - current runtime thread-safety posture
21. `fec_dependency_profile`
   - current adapter-level dependency summary
22. `surface_fec_dependency_profile`
   - current surface pipeline dependency summary
23. `metadata_status`
   - current extraction status for the detailed profile columns:
     - `function_meta_extracted`
     - `catalog_only`
     - `doc_modeled`
24. `special_interface_kind`
   - first-pass signal that a row is seam-heavy rather than ordinary
25. `admission_interface_kind`
   - first-pass indication of whether the row is an ordinary call, helper-formation form, higher-order call, operator form, or host-subscription call
26. `preparation_owner`
   - first-pass indication of where preparation/formation responsibility mainly sits
27. `runtime_boundary_kind`
   - first-pass indication of the runtime seam OxFml should expect after preparation
28. `arity_shape_note`
   - free-form first-pass note for special argument-shape or helper/operator admission details
29. `interface_contract_ref`
   - current best contract/workset artifact to follow for seam-heavy rows
30. `source_catalog_ref`
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
   - preserve `surface_stable_id`

## 6. Current Honest Limits
1. This export includes the full current `W45` non-`@` operator surface plus one explicitly modeled `FUNC.OP_IMPLICIT_INTERSECTION` row, not the full future operator universe.
2. Some seam-heavy rows such as `LET` and `LAMBDA` still have blank detailed profile columns; that currently means "follow `interface_contract_ref`", not "treat as ordinary default semantics".
3. `semantic_trait_profile_ref` and `gating_profile_ref` are currently family refs, not fully dereferenceable per-row downstream contracts.
4. `admission_interface_kind`, `preparation_owner`, `runtime_boundary_kind`, and `arity_shape_note` are first-pass OxFunc guidance fields, not yet locked shared vocabulary.
5. This export does not itself inline localized names; it points to the current multilingual seed table.
6. This export does not carry runtime capability, provider availability, caller-context, or host-query payload facts.
7. The exact final shared field set and field names are still not locked cross-repo.

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
