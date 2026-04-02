# WORKSET - Library Context Snapshot Export Baseline (W44)

## 1. Purpose
Produce the first honest OxFunc-local export for the external library-context snapshot that OxFml can bind against for parse, bind, semantic planning, and replay correlation.

This packet exists because the current OxFml note has moved from abstract agreement on the snapshot idea to a concrete ask:
1. provide a pinned export that preserves the minimum agreed fields, or
2. provide a stable downstream pointer plus export-reading guidance.

## 2. Why This Packet Exists
Current OxFunc already has:
1. a corrected current-baseline catalog in `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`,
2. seam notes narrowing the minimum snapshot fields,
3. growing pressure from:
   - callable/helper binding,
   - availability/gating classification,
   - localized function names,
   - replay correlation.

What it does not yet have is one explicit downstream artifact that honestly serves as the external library-context snapshot.

## 3. In Scope
1. define the first OxFunc-local snapshot export artifact,
2. include the minimum exercised field set needed for OxFml:
   - `surface_stable_id`
   - name-resolution table or equivalent pointer
   - semantic trait/profile reference
   - gating profile reference
   - snapshot identity/version
3. state which current OxFunc artifacts remain authoritative sources for those exported fields,
4. provide export-reading guidance for OxFml,
5. keep runtime capability/session/provider truth out of the snapshot.
6. own the exact refresh inventory for documented-complete rows that the current export still mislabels as `catalog_only`.

## 4. Out Of Scope
1. locking the final cross-repo snapshot ABI,
2. inlining every downstream semantic profile or full function contract,
3. runtime host/provider/capability state,
4. final operator-admission lock,
5. full callable-carrier lock.

## 4A. Current Long-Term Direction
Current OxFunc reading after the latest OxFml round:
1. the CSV export remains the right pinned interchange and debugging artifact for bounded integration rounds,
2. the CSV export is now also the retained structural seed for `W069` witness generation rather than the long-term destination surface,
3. but the preferred long-term implementation seam should be:
   - runtime `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - explicit snapshot generations when registration/removal changes the library context,
4. downstream runtime integration should not stay coupled to build-time CSV ingestion as the normative implementation path.

## 5. Candidate Inputs
1. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
2. `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv`
3. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V1.md`
4. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V2.md`
5. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`

## 6. Gate Criteria
This packet can only be reported `scope_complete` when:
1. one explicit export artifact or stable export pointer exists,
2. snapshot identity/versioning is stated,
3. the minimum exported fields are pinned,
4. the relationship between exported rows and canonical OxFunc-local sources is documented,
5. OxFml-facing reading guidance is written.

## 7. Initial Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - first-pass export exists and now includes the full current `W45` non-`@` operator surface plus one doc-modeled implicit-intersection row
   - seam-heavy rows like `LET` and `LAMBDA` still rely partly on linked contract artifacts rather than fully normalized direct profile fields
   - OxFml has now accepted the current first-pass callable-row split for one bounded round, but no runtime consumer/model example is pinned yet
   - exact per-entry semantic/gating profile dereferenceability is not yet frozen
   - the preferred runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` direction is now explicit and the concrete shared runtime shape now lives in successor `W049`
   - the exact documented-complete snapshot-stale refresh set is now pinned in `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv`
   - that exact refresh set is `114` unique rows, with source split:
     - `W024`: `62`
     - `W022`: `7`
     - `W027`: `13`
     - `W033`: `11`
     - `W037`: `1`
     - `W045`: `19`
   - legacy `RANDARRAY` cleanup: `1`
   - that refresh is now generator-driven in `tools/w44-probe/generate-w44-library-context-snapshot.ps1` and applied in the current export
   - the next honest W44 lane is broader metadata-field refinement and consumer-shape normalization rather than another prose-only stale/backlog recount
   - `W044` now remains as the pinned `V1` export provenance owner and `W069` bridge input, not as the final downstream destination surface

## 8. Current Outputs
1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
3. `docs/function-lane/W44_EXECUTION_RECORD.md`
4. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv`
