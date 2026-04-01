# WORKSET - In-Scope Current-Version Not-Complete Surface (W51)

## 1. Purpose
Centralize the Excel function and operator rows that are still in scope for the current OxFunc version target and are not yet honestly reportable as supported in the published OxFunc library catalog.

This packet now also acts as the first-pass reconciliation hub between:
1. the published library-context snapshot surface seen by downstream consumers,
2. the deferred-current-version list in `W050`,
3. the explicit packet-local residual rows already known to remain open,
4. hidden non-deferred `catalog_only` rows that were still outside legacy `W051`.

This packet also records the current shared-interface acknowledgement outcome for OxFunc ↔ OxFml work:
1. the shared interaction model is now acknowledged for the previously seam-heavy non-deferred surface that was carried through `W023`, `W038`, `W042`, `W046`, `W047`, `W048`, and `W049`,
2. the hidden `185`-row ordinary backlog remains excluded from that acknowledgement scope unless a concrete later mismatch shows that one of those rows needs more than the ordinary built-in interaction path,
3. the `17` deferred rows in `W050` remain excluded from the current acknowledgement scope and may reopen only on the same concrete-mismatch basis in a later round.

## 2. Provenance
This packet consolidates active current-version backlog ownership from:
1. `W014_IMPLICIT_INTERSECTION_OPERATOR.md`
2. `W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
3. `W025_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_OUTLIERS.md`
4. `W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md`
5. `W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`
6. `W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM.md`
7. latent catalog gaps visible through `W044`
8. later packet-complete evidence checked during the first-pass catalog reconciliation:
   - `W022_CRITERIA_FAMILY_SHAPE_HARDENING.md`
   - `W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
   - `W027_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_HARDENING.md`
   - `W033_INFORMATION_PREDICATES_AND_FORECAST_COMPATIBILITY_CLOSURE.md`
   - `W037_REOPENED_XIRR_LARGE_ROOT_SOLVER_PRECISION.md`
   - `W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`

## 3. Scope
Machine-readable working inputs:
1. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
2. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv`
3. `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv`
4. `docs/function-lane/W58_GROUPED_ROW_NORMALIZATION_MAP.csv`
5. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
6. `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`
7. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv`

Current working total:
1. `192` function rows.
2. `0` operator rows.
3. `192` total rows.
4. current working backlog split:
   - `0` explicitly tracked residual rows with real runtime/formal/evidence floors or packet-specific open lanes,
   - `192` normalized ordinary execution rows produced from the hidden first-pass backlog.
5. current shared-interface-acknowledgement split:
   - the prior seam-heavy non-deferred surface is now acknowledged and promoted out of `W051`,
   - no explicit residual seam-heavy row remains in current `W051`,
   - `185` hidden non-interesting rows are excluded from the shared-interface pass and treated as ordinary backlog unless a later concrete mismatch proves otherwise.
6. current hidden-backlog identity split:
   - `185` hidden snapshot entries remain the authoritative published-catalog reading,
   - `W058` splits `7` grouped snapshot entries into `14` explicit function members,
   - the ordinary execution program therefore now operates on `192` machine-clean function rows.

First-pass published-catalog reading:
1. `534` published rows in `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`:
   - `511` functions,
   - `23` operators.
2. `332` rows are currently usable on a first-pass consumer read:
   - `332` supported/complete,
   - `0` preview.
3. `17` rows are deferred in the current version target through `W050`.
4. `185` rows are non-deferred but not yet supported and are now centralized here as real first-pass backlog.
5. the exact `114` documented-complete snapshot-stale rows from the first-pass reconciliation have now been refreshed out of false `catalog_only` state in the published snapshot export and are not included in the W51 backlog totals.

First-pass documented-complete snapshot-stale groups now refreshed in the current export:
1. `W024` ordinary-done rows: `62`.
2. `W022` criteria-family closures: `7`.
3. `W027` advanced-bond/odd-bond closures: `13`.
4. `W033` information-predicate/forecast closures: `11`.
5. `W037` reopened `XIRR` closure: `1`.
6. `W045` non-`@` operator closures: `19`.
7. `RANDARRAY` legacy cleanup: `1`.

Exact-inventory note:
1. the earlier first-pass packet-group prose had one overlap on `XIRR` between `W024` and later `W037`,
2. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv` resolves that overlap and pins the exact unique refresh set at `114`,
3. the exact source split is now:
   - `W024`: `62`
   - `W022`: `7`
   - `W027`: `13`
   - `W033`: `11`
   - `W037`: `1`
   - `W045`: `19`
   - legacy `RANDARRAY` cleanup: `1`
4. `W058` preserves the original hidden-backlog `185` snapshot entries as provenance while creating the machine-clean `192`-row execution inventory used for ordinary backlog closure.

Completed and removed from this inventory (moved to function-phase-complete):
- `BYCOL`, `BYROW`, `CALL`, `IMAGE`, `ISOMITTED`, `LAMBDA`, `LET`, `MAKEARRAY`, `MAP`, `REDUCE`, `REGISTER.ID`, `SCAN` (12 rows after packet-local closure and shared-freeze promotion)
- `GROUPBY`, `PIVOTBY` (2 rows after grouped-aggregation current-baseline promotion in `W055`)
- `OP_IMPLICIT_INTERSECTION` (1 operator after current-baseline implicit-intersection and legacy-single closure in `W014`)
- `COLUMNS`, `RANDARRAY`, `RANDBETWEEN`, `ROWS`, `TRIMRANGE`, `VALUETOTEXT` (6 functions)
- `OP_TRIM_REF_LEADING`, `OP_TRIM_REF_TRAILING`, `OP_TRIM_REF_BOTH` (3 operators, verified against W045 structural slice)

Important current reading:
- the exact packet-local work split for any future explicit residual rows is pinned through `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`; it is currently empty because no explicit preview-cluster row remains.
- there is no remaining explicit preview-cluster row; the only residual `W051` membership is now the hidden ordinary backlog.
- the new hidden `185`-row appendix is a different class of backlog:
  - these rows were still `catalog_only` in the published snapshot,
  - they were not in `W050`,
  - they were not in legacy `W051`,
  - and this first pass did not find a later packet already closing them.
- `W058` now supplies the authoritative machine-clean execution view of that same backlog:
  - `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv` is the execution inventory,
  - `docs/function-lane/W58_GROUPED_ROW_NORMALIZATION_MAP.csv` pins the seven grouped-row splits,
  - `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv` freezes the exact `W059` through `W068` ownership counts.
- for shared-interface acknowledgement purposes, the hidden `185` should currently be read as ordinary built-in backlog rather than open OxFml-shape backlog.
- frozen successor execution split after `W058`:
  - `W059`: `16`
  - `W060`: `26`
  - `W061`: `29`
  - `W062`: `35`
  - `W063`: `18`
  - `W064`: `15`
  - `W065`: `12`
  - `W066`: `23`
  - `W067`: `15`
  - `W068`: `3`

Explicit preview-cluster functions:
None.

Explicit preview-cluster operators:
None.

## 4. Current-Version Rule
For the current version target:
1. every non-deferred row that is not honestly supported in the consumer-facing catalog must appear here, either in the explicit inventory or in the hidden-backlog appendix,
2. rows that are only `catalog_only` in the snapshot export are not exempt from W51 just because they were not yet named in a narrower packet,
3. `GROUPBY` and `PIVOTBY` are now complete for declared current-phase scope under `W055` and removed from `W051`,
4. `IMAGE`, the callable-helper family rows from `W038`, and `CALL` / `REGISTER.ID` are now complete for declared current-phase scope and removed from `W051`,
5. `HYPERLINK` is now treated as complete on the OxFunc side and therefore removed from `W051`; host publication application remains above OxFunc rather than an OxFunc function gap,
6. `ROWS`, `COLUMNS`, `RANDBETWEEN`, `VALUETOTEXT`, `RANDARRAY`, `TRIMRANGE` are now function-phase-complete and removed,
7. trim-reference operators (`OP_TRIM_REF_*`) are verified against W045 structural slice and removed,
8. rows with later complete packet evidence should be kept out of W51 and instead recorded as snapshot/export drift until metadata refresh catches up.
9. the current shared-interface freeze/promotion pass is owned by:
   - `W042` for the callable minimum carrier,
   - `W046` for registered-external packet shape,
   - `W047` for typed context/query bundle,
   - `W048` for return-surface split,
   - `W049` for runtime library-context provider/snapshot model,
   - `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` as the consolidated outbound model.

## 5. Ownership Rule
1. `W51` is the canonical current-version not-complete working backlog for all non-deferred outstanding rows.
2. Older packets remain provenance/evidence owners and, where applicable, execution owners for the explicit family-specific residual rows.
3. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv` remains provenance for the original hidden snapshot-entry discovery.
4. `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv` is the authoritative execution-owner inventory for the ordinary backlog and freezes successor ownership through `W059` to `W068`.
5. New latent gaps or reconciliation findings should be added here immediately, then extracted into narrower execution packets as needed.

## 6. Cleanup And Completion Sequence
1. Finalize the remaining shared-model owner packets for the interesting backlog:
   - `W042`
   - `W046`
   - `W047`
   - `W048`
   - `W049`
2. Consolidate those owner-packet readings into one outbound current-phase model note:
   - `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
3. Completed in the current pass: the OxFunc ↔ OxFml notes interchange against that single consolidated model no longer shows a material interface mismatch for the interesting non-deferred surface.
4. Completed in the current pass: the shared-interface model is now acknowledged across OxFunc and OxFml and promoted into the current owner-packet reading across `W042`, `W046`, `W047`, `W048`, `W049`, and `W051`.
5. Completed in the current pass: close the packet-local promotion/doc lanes for `W023`, `W038`, and `W046`, removing `IMAGE`, the nine callable-helper rows, and `CALL` / `REGISTER.ID` from `W051`.
6. Completed in the current pass: refresh the exact `114` documented-complete snapshot-stale rows listed in `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv` out of false `catalog_only` state in the published library-context snapshot and downstream labeling guidance.
7. Completed in the current pass: promote `GROUPBY` and `PIVOTBY` through `W055`, removing them from the residual preview cluster and narrowing the explicit `W051` surface to `OP_IMPLICIT_INTERSECTION` only.
8. Completed in the current pass: close `W014` for declared current-phase scope after pinning current-baseline `_xlfn.SINGLE(...)` normalization and removing `OP_IMPLICIT_INTERSECTION` from `W051`.
9. Completed in the current pass: `W058` normalized grouped-name rows in the hidden backlog appendix (`FIND, FINDB`; `LEFT, LEFTB`; `LEN, LENB`; `MID, MIDB`; `REPLACE, REPLACEB`; `RIGHT, RIGHTB`; `SEARCH, SEARCHB`) into machine-clean row identities.
10. Completed in the current pass: `W058` froze the normalized ordinary backlog into narrower successor execution packets by family rather than leaving it as one mega-list.
11. Keep `W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md` aligned with each closure wave so the catalog does not get ahead of the executable/formal doctrine again.
12. Regenerate the published consumer surface after each closure wave:
   - snapshot export
   - labeling policy
   - consumer-facing counts
   - `W051` membership

Exit condition:
- `W051` contains every non-deferred row that is still not honestly supported.
- the shared-interface model for the interesting non-deferred surface is explicitly frozen across OxFunc and OxFml.
- documented-complete rows are no longer misreported as `catalog_only` in the published snapshot.
- downstream consumer docs can report one aligned set of counts without relying on stale side packets.

## 7. Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - `192` normalized hidden ordinary execution rows still lack semantic closure packets
   - successor execution work `W059` through `W068` has not started yet
