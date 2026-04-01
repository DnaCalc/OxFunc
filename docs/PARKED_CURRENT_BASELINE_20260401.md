# OxFunc Parked Current Baseline 2026-04-01

This note records the intended parked baseline for the current non-deferred OxFunc surface as of `2026-04-01`.

Parking scope:
1. Full current published catalog is present in the library-context export.
2. All non-deferred current-version rows are drained from active backlog.
3. The remaining deferred set stays owned by `W050`.
4. The current-phase Rust/Lean alignment gap for non-deferred rows is closed.

Current parked reading:
1. Published rows: `534`
2. Supported rows: `517`
3. Preview rows: `0`
4. Deferred rows: `17`
5. Non-deferred backlog rows in `W051`: `0`
6. Non-deferred active Lean gap rows in `W054`: `0`

Canonical parked-state artifacts:
1. Current non-deferred backlog truth: `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
2. Current non-deferred formalization reconciliation truth: `docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`
3. Current published catalog/profile artifact and consumer guidance:
   - `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
   - `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`

Deferred-surface note:
1. `W050` remains the canonical deferred-current-version tracker.
2. The deferred inventory is `17` rows even though only `14` rows currently remain `catalog_only` in the export; `ENCODEURL`, `FILTERXML`, and `TRANSLATE` already carry profile metadata.

Snapshot provenance note:
1. The exported snapshot has been regenerated from the parked baseline commit and should carry the current clean-tree provenance fields.

Next-direction note:
1. The next planned forward packet after parking is `W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md`.

Status:
- `scope_completeness: scope_complete`
- `target_completeness: target_complete`
- `integration_completeness: integrated`
