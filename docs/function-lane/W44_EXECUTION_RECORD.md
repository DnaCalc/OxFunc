# W44 Execution Record

## 1. Packet
1. workset: `W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE`
2. execution_state: `in_progress`

## 2. Objective
Convert the library-context snapshot discussion from a note-only seam topic into one explicit downstream export artifact or stable export pointer that OxFml can consume.

## 3. Outputs Produced
1. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
3. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
4. `docs/upstream/NOTES_FOR_OXFML.md`
5. `docs/function-lane/XLCALL_CODE_CATALOG.csv`

## 4. Current Export Produced
Current first-pass export:
1. `snapshot_id`: `oxfunc-libctx-v1`
2. `snapshot_generation`: current generation date emitted by the export script
3. `source_commit_short`: current repo commit that generated the export
4. `source_commit_full`: full current repo commit hash that generated the export
5. `source_tree_state`: `clean` or `dirty` local tree state for the export run
6. scope:
   - built-in worksheet functions
   - current exported evaluable operator surface
   - one explicitly modeled special operator row for implicit intersection
7. current row count: `534`
   - `511` functions
   - `23` operators

Current row fields:
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

## 5. Why This Is Better Than The Old Pointer
Before this packet, the best downstream pointer was only:
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`

That was useful but insufficient because it did not by itself pin:
1. snapshot identity/version,
2. the intended stable function id projection,
3. the explicit localization-table pointer,
4. a declared semantic/gating reference surface,
5. export-reading guidance for OxFml.
6. source commit identity for reproducible test pinning.
7. full commit identity for stronger downstream fixture pinning.
8. current local tree cleanliness for honest downstream fixture pinning.
9. built-in `XLCALL.H` identity interop for host C API routing.

## 6. Current Honest Limits
1. operator coverage now includes the full current `W45` non-`@` operator surface plus one doc-modeled implicit-intersection row, but this is still not a claim about every future operator the architecture may eventually expose
2. some seam-heavy rows such as `LET` and `LAMBDA` still rely on `interface_contract_ref` rather than fully normalized detailed profile columns
3. the current export may be generated from a dirty local tree; that is now explicit in the artifact, but a dirty snapshot still is not the same thing as a clean release snapshot
4. `semantic_trait_profile_ref` and `gating_profile_ref` remain first-pass OxFunc-local references, not fully dereferenceable downstream profile bundles
5. localized names remain pointed-to rather than inlined
6. runtime capability/provider state is intentionally excluded
7. the new seam-facing guidance fields are still first-pass OxFunc vocabulary rather than locked shared names
8. final cross-repo field names and ABI remain open
9. `xlcall_builtin_*` currently covers matched built-in `xlf*` rows only, not commands, auxiliary callbacks, or future registered-external rows

## 7. Verification
1. generated from `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
2. detailed profile columns extracted mechanically from current `FunctionMeta` definitions where present, including the `reshape_meta!` family used by the completed `W039` dynamic-array reshaping batch
3. operator rows derived from the current exported operator surface in `tools/xll-addin/oxfunc_xll/export_specs.csv`, plus one doc-modeled `FUNC.OP_IMPLICIT_INTERSECTION` row
4. row count matches current combined export:
   - `511` function rows
   - `23` operator rows
5. current export inspected manually for header shape and special-row stability
6. key special-row fields inspected manually for:
   - `CALL`
   - `REGISTER.ID`
   - `LET`
   - `LAMBDA`
   - `ASC`
   - `DBCS`
   - `JIS`
   - `NUMBERVALUE`
   - `NOW`
   - `TODAY`
   - `HYPERLINK`
   - `MAP`
   - `RTD`
   - `TRANSLATE`
   - `OP_IMPLICIT_INTERSECTION`
   - representative `W039` rows:
     - `CHOOSECOLS`
     - `FILTER`
     - `UNIQUE`
     - `VSTACK`
7. export provenance fields inspected manually for:
   - `source_commit_short`
   - `source_commit_full`
   - `source_tree_state`
   - `snapshot_generation`
8. `XLCALL.H` ingest generated and inspected manually for representative rows:
   - `SUM`
   - `CALL`
   - `REGISTER.ID`
   - `RTD`

## 8. Status
1. scope_completeness: `scope_partial`
2. target_completeness: `target_partial`
3. integration_completeness: `partial`
4. open_lanes:
   - the export is materially better after `W45`, but it is still a first-pass stabilization artifact rather than a locked final ABI
   - seam-heavy rows like `LET` and `LAMBDA` still need more normalized direct profile fields
   - the export is now more honest about local source state, but OxFml still has no pinned clean-release consumer example
   - per-entry semantic/gating profile dereferenceability is still coarse
   - no formal consumer example for OxFml exists yet
