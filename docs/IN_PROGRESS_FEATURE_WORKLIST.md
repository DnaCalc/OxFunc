# IN_PROGRESS_FEATURE_WORKLIST.md - OxFunc

Status: `active_feature_map`
Last updated: 2026-04-02

Purpose:
1. provide a compact repo-level map of the major OxFunc lanes that remain live after the parked non-deferred baseline,
2. point readers at the current owning workset or contract surface,
3. avoid duplicating execution-state, blocker, or archive-wave detail now owned by `.beads/`.

Use rule:
1. use this file as a high-level feature map only,
2. use [WORKSET_REGISTER.md](C:\Work\DnaCalc\OxFunc\docs\WORKSET_REGISTER.md) for ordered workset truth,
3. use `.beads/` for ready, blocked, in-progress, and closed execution state,
4. use [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md) and [W050_DEFERRED_CURRENT_VERSION_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W050_DEFERRED_CURRENT_VERSION_SURFACE.md) for current parked-surface counts and deferred/non-deferred scope truth.

Supersession note:
1. this file no longer acts as a second execution tracker,
2. detailed migration progress for the doctrine reset belongs to [W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) plus `.beads/`,
3. detailed `V2` schema and witness work belongs to [W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md).

## Active Feature Map

### IP-01 Parked Function Surface
- Current state: the non-deferred current-version surface is parked and complete; the remaining excluded rows are the intentional deferred `W050` set.
- Canonical owner: [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md) and [W050_DEFERRED_CURRENT_VERSION_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W050_DEFERRED_CURRENT_VERSION_SURFACE.md).

### IP-02 Semantic Witness Snapshot V2
- Current state: planned and partially scaffolded through schema and bridge-definition work.
- Canonical owner: [W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md), [OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md](C:\Work\DnaCalc\OxFunc\docs\function-lane\OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md), and [W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md).

### IP-03 Library-Context Export And Runtime Provider
- Current state: `V1` export is parked and live; remaining work is the bridge from `V1` export into witness-bearing runtime consumption.
- Canonical owner: [W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md) and [W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md).

### IP-04 Deferred Provider And Host-Bound Surface
- Current state: intentionally excluded from the parked current-version baseline.
- Canonical owner: [W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md), [W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md), and [W050_DEFERRED_CURRENT_VERSION_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W050_DEFERRED_CURRENT_VERSION_SURFACE.md).

### IP-05 Formalization Deepening
- Current state: non-deferred Rust-vs-Lean id reconciliation is parked complete; deeper proof work remains a distinct long-run lane.
- Canonical owner: [W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md).

### IP-06 XLL Verification Seam
- Current state: retained as a live seam/evidence lane, not a closed historical packet.
- Canonical owner: [XLL_VERIFICATION_SEAM_LIMITATIONS.md](C:\Work\DnaCalc\OxFunc\docs\function-lane\XLL_VERIFICATION_SEAM_LIMITATIONS.md) and the surviving XLL bridge notes in `docs/function-lane/`.

### IP-07 Locale And Version Sweeps
- Current state: still a planned orthogonal validation phase, not part of the parked current-baseline completion claim.
- Canonical owner: future workset; no live execution bead unless explicitly reopened.

### IP-08 Execution Doctrine Migration And Active-Tree Reduction
- Current state: parked complete; doctrine rewrite, bead bootstrap, archive reduction, reconciliation, and closure proof are complete.
- Canonical owner: [W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) as migration provenance, with ordinary live execution now owned by [WORKSET_REGISTER.md](C:\Work\DnaCalc\OxFunc\docs\WORKSET_REGISTER.md) plus `.beads/`.

## Status Vocabulary
- `planned`: accepted lane, no active execution claim implied here.
- `active`: live lane with current owner surfaces.
- `parked`: current baseline or parked authority exists; reopen only by explicit future work.

Current reading:
1. `IP-01` is `parked`,
2. `IP-02`, `IP-03`, `IP-04`, `IP-05`, and `IP-06` are `active`,
3. `IP-07` is `planned`,
4. `IP-08` is `parked`.
