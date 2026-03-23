# Function Slice - Runtime Library Context Provider Consumer Model (Prelim)

Status: `active`
Packet: `W049`

## 1. Purpose
Pin the first runtime-only `LibraryContextProvider` / immutable `LibraryContextSnapshot` model for the already-covered scope, while keeping the CSV snapshot export as the bounded interchange and debugging artifact.

## 2. Current Freeze Candidate
The current first-freeze runtime model is:
1. `LibraryContextProvider`
2. immutable `LibraryContextSnapshot`
3. explicit generation changes when registration/removal changes library-context truth
4. explicit mapping from the runtime model to the current CSV export artifact

## 3. Runtime Model Shape
Current runtime-only shape should not mirror the CSV column-for-column. It should group fields by runtime responsibility.

### 3.1 Provider
`LibraryContextProvider` should expose:
1. `current_snapshot() -> LibraryContextSnapshot`
2. optionally `snapshot_by_generation(generation_id) -> Option<LibraryContextSnapshot>` for debugging/replay support

Current reading:
1. consumers should treat the provider as the authority for the latest snapshot,
2. consumers should not depend on mutable in-place catalog updates,
3. registration/removal should publish a new snapshot generation rather than mutating the currently observed snapshot.

### 3.2 Snapshot
`LibraryContextSnapshot` should carry:
1. snapshot identity and provenance
2. immutable entry set
3. resolution indexes
4. stable generation identity

Current runtime snapshot fields:
1. `snapshot_family_id`
2. `snapshot_generation`
3. `source_commit_short`
4. `source_commit_full`
5. `source_tree_state`
6. `entries_by_stable_id`
7. `entries_by_canonical_name`
8. `entries_by_xlcall_code` where applicable
9. `name_resolution_source`

### 3.3 Entry
`LibraryContextEntry` should group fields into:
1. identity
2. surface naming
3. planner-visible semantics
4. seam guidance
5. provenance

Current runtime entry fields:
1. identity:
   - `surface_stable_id`
   - `entry_kind`
   - `registration_source_kind`
2. surface naming:
   - `canonical_surface_name`
   - `name_resolution_table_ref`
3. planner-visible semantics:
   - `semantic_trait_profile_ref`
   - `gating_profile_ref`
   - `version_marker`
   - `category`
   - `interesting`
   - `arity`
   - `arg_preparation_profile`
   - `coercion_lift_profile`
   - `kernel_signature_class`
   - `determinism_class`
   - `volatility_class`
   - `host_interaction_class`
   - `thread_safety_class`
   - `fec_dependency_profile`
   - `surface_fec_dependency_profile`
4. seam guidance:
   - `metadata_status`
   - `special_interface_kind`
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `arity_shape_note`
   - `interface_contract_ref`
5. provenance:
   - `source_catalog_ref`
   - `xlcall_builtin_symbol`
   - `xlcall_builtin_code`

## 4. Why Runtime Should Not Mirror CSV Directly
The CSV export is useful because it is:
1. stable
2. inspectable
3. easy to diff
4. easy to pin in cross-repo mismatch reports

But the runtime model should differ because:
1. runtime consumers want grouped semantics rather than flat stringly columns,
2. lookup indexes matter at runtime but are flattened awkwardly in CSV,
3. immutable snapshot semantics are clearer in object form than in tabular form,
4. later registered-external entries should be addable without redefining the CSV as the normative runtime ABI.

## 5. Generation Behavior
Current generation rule:
1. built-in-only steady state may reuse the committed generation from the currently pinned export artifact,
2. any future registered-external addition or removal should produce a new `LibraryContextSnapshot`,
3. downstream consumers should compare `snapshot_generation` rather than infer change from incidental row ordering,
4. `W046` owns the worksheet registration seam, but the generation behavior belongs here.

## 6. Consumer Walkthrough Summary
First-pass consumer model:
1. OxFml gets `current_snapshot()` from `LibraryContextProvider`.
2. Name resolution binds formulas against `canonical_surface_name` plus `name_resolution_table_ref`.
3. Binder preserves `surface_stable_id` and planner-facing profile fields on the bound call/operator node.
4. For seam-heavy rows, consumer also preserves:
   - `special_interface_kind`
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `arity_shape_note`
   - `interface_contract_ref`
5. Evaluation then combines:
   - runtime snapshot entry
   - `W047` typed context/query bundle
   - `W048` return-surface split
6. If registration/removal changes library-context truth, a fresh immutable snapshot generation is requested instead of mutating the bound meaning of the current snapshot silently.

## 7. Covered Current-Scope Pressure Cases
This runtime model is already sufficient for the currently covered seam-heavy scope:
1. `LET` / `LAMBDA` planning rows from `W044`
2. `RTD`
3. `TRANSLATE`
4. `CALL`
5. `REGISTER.ID`
6. `OP_IMPLICIT_INTERSECTION`
7. presentation-aware functions such as `NOW`, `TODAY`, and `HYPERLINK`

## 8. Boundaries
This packet does not freeze:
1. final cross-repo ABI naming
2. final registered-external descriptor shape
3. host/provider runtime capability state
4. callable minimum carrier beyond already-converged first-freeze needs

## 9. Artifacts
This packet currently binds:
1. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
2. `docs/function-lane/W49_RUNTIME_LIBRARY_CONTEXT_CSV_TO_RUNTIME_MAPPING.csv`
3. `docs/function-lane/W49_RUNTIME_LIBRARY_CONTEXT_CONSUMER_WALKTHROUGH.md`
4. `docs/function-lane/W49_EXECUTION_RECORD.md`
5. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
