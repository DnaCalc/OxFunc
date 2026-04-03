# OxFunc Worksets

This folder is the compact active map for workset-level OxFunc planning and
provenance.

It is not a live execution tracker.
For ordered workset truth use [WORKSET_REGISTER.md](/C:/Work/DnaCalc/OxFunc/docs/WORKSET_REGISTER.md).
For live execution state use [.beads/issues.jsonl](/C:/Work/DnaCalc/OxFunc/.beads/issues.jsonl) through `br`.

## Current Rules
1. Worksets are planning and provenance packets, not the owner of ready or blocked state.
2. `.beads/` owns live execution truth.
3. Closed historical worksets removed from `main` are preserved through [HISTORY.md](/C:/Work/DnaCalc/OxFunc/docs/HISTORY.md) and tag `OxFunc_V1`.

## Active Workset Set
1. [W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md)
2. [W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md)
3. [W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md)
4. [W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md)
5. [W050_DEFERRED_CURRENT_VERSION_SURFACE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md)
6. [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md)
7. [W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md)
8. [W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md)

## Active Role Split
1. [W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md) remains the live deferred/provider-family authority.
2. [W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md) remains the live RTD seam authority.
3. [W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md) remains the live V1 export provenance owner.
4. [W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md) remains the retained runtime carrier model.
5. [W050_DEFERRED_CURRENT_VERSION_SURFACE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md) remains the canonical deferred tracker.
6. [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md) remains the parked non-deferred tracker and currently records zero open rows.
7. [W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md) remains the parked Lean reconciliation authority.
8. [W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md) is the next substantive execution target.

## Use These Instead
1. Use [PARKED_CURRENT_BASELINE_20260401.md](/C:/Work/DnaCalc/OxFunc/docs/PARKED_CURRENT_BASELINE_20260401.md) for the parked non-deferred baseline summary.
2. Use [WORKSET_REGISTER.md](/C:/Work/DnaCalc/OxFunc/docs/WORKSET_REGISTER.md) for ordered workset truth.
3. Use [BEADS.md](/C:/Work/DnaCalc/OxFunc/docs/BEADS.md) for the local bead method.
4. Use [IN_PROGRESS_FEATURE_WORKLIST.md](/C:/Work/DnaCalc/OxFunc/docs/IN_PROGRESS_FEATURE_WORKLIST.md) only as a high-level feature map.
5. Use [HISTORY.md](/C:/Work/DnaCalc/OxFunc/docs/HISTORY.md) and tag `OxFunc_V1` for removed workset packets and retired migration aids.
6. Use [W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md) only as closed framework provenance, not as active execution authority.
7. Use [W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) only as closed migration provenance, not as active workset authority.
