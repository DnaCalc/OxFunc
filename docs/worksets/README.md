# OxFunc Worksets

Current workset-truth note:
1. `docs/WORKSET_REGISTER.md` is now the canonical ordered workset surface.
2. `.beads/` now owns live execution-state truth.
3. This file is now a workset index and historical packet map, not a live execution-state surface.

Worksets are sequence-based planning and provenance packets for cross-cutting OxFunc slices.

Supersession note:
1. Any older reading that `W051` contains only the explicit `15`-row residual preview cluster is superseded by `W051` itself.
2. For current non-deferred outstanding-row truth, use `W051` rather than older family packets or snapshot-only readings.
3. For the current OxFunc-owned shared-interface freeze candidate, use `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.
4. For the parked current non-deferred baseline, use `docs/PARKED_CURRENT_BASELINE_20260401.md`.

Primary kickoff orchestration:
1. historical kickoff packets through `W021` now live behind `docs/HISTORY.md` and the `OxFunc_V1` tag.

Current active worksets:
1. `W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md` (W41)
2. `W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md` (W43)
3. `W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md` (W44)
4. `W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md` (W49)
5. `W050_DEFERRED_CURRENT_VERSION_SURFACE.md` (W50)
6. `W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md` (W51)
7. `W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md` (W54)
8. `W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md` (W69)
9. `W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md` (W70)

Historical packet note:
1. closed historical worksets removed from `main` are indexed in `docs/HISTORY.md` and preserved at tag `OxFunc_V1`.

Common rules:
1. Worksets are sequence/gate driven, never date driven.
2. Each workset must declare dependencies, deliverables, and gate criteria.
3. Completion is recorded by gate closure and explicit status updates.
4. Claim confidence (`draft/provisional/validated`) and assurance maturity (`exercised/green-validated`) must be stated separately.

Mega-batch planning note:
1. If native replay shows that a row is host/profile-sensitive, provider-bound, or absent on the current host surface, extract it early to a successor packet instead of carrying it as an ordinary mega-batch member.
2. For advanced finance families, add at least one direct Excel-valued parity row early; internally consistent local tests are not enough by themselves.
3. Mega-batch packets should define the reconciliation rule up front: every row must end as either `done` or `extracted`, with no silent residual state.

Process references:
- Pre-closure checklist: `OPERATIONS.md` Section 12.
- Completion claim self-audit: `OPERATIONS.md` Section 14.
- Live execution blockers: `.beads/`.
- Exceptional prose blocker ledger only: `CURRENT_BLOCKERS.md`.
- In-progress feature register: `docs/IN_PROGRESS_FEATURE_WORKLIST.md`.

Replay rollout sequence after `W016`:
1. Historical replay rollout packets `W017` through `W021` are preserved behind `docs/HISTORY.md` and `OxFunc_V1`.
2. The surviving live replay surfaces are the replay-support docs in `docs/function-lane/` and the active feature/register lanes in `docs/IN_PROGRESS_FEATURE_WORKLIST.md` and `docs/WORKSET_REGISTER.md`.
3. `W047`, `W048`, and `W049` are complete for declared current-phase scope and remain as shared-freeze provenance for typed context/query, return-surface publication, and runtime provider/snapshot consumer modeling.
4. The previously seam-heavy packets `W023`, `W038`, `W046`, `W055`, and `W014` are complete for declared current-phase scope, and the hidden ordinary backlog was fully drained through `W068`.
5. `W050` and `W051` now centralize current-version backlog truth:
   - `W050` is the canonical deferred-current-version list.
   - `W051` is the canonical in-scope current-version backlog surface and currently records no remaining non-deferred backlog.
   - Older family packets remain provenance/evidence owners rather than the active central tracker.
6. Current `W051` totals are:
   - `0` normalized non-deferred outstanding execution rows (`0` functions, `0` operators).
   - `0` hidden non-deferred backlog snapshot entries after `W068`.
   - `0` explicit preview-cluster rows plus `0` hidden ordinary execution rows.
   - The first-pass `114` documented-complete snapshot-stale rows have been refreshed into the published export.
7. Current interface-finalization reading:
   - the prior seam-heavy non-deferred surface is acknowledged across OxFunc and OxFml and promoted out of `W051`,
   - no explicit residual preview row remains after `W014` current-phase closure,
   - the hidden ordinary backlog is fully drained after `W068`,
   - the current OxFunc-owned consolidated candidate lives in `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.


