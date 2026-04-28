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
9. [W072_BUG_INTAKE_ROOT_CAUSE_AND_REGRESSION_STREAM_PROTOCOL.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W072_BUG_INTAKE_ROOT_CAUSE_AND_REGRESSION_STREAM_PROTOCOL.md)
10. [W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md)
11. [W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md)
12. [W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md)
13. [W076_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W076_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md)
14. [W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md)
15. [W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md)
16. [W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md)
17. [W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md)
18. [W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md)
19. [W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md)
20. [W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md)
21. [W084_COUNTBLANK_RANGE_ONLY_PARITY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W084_COUNTBLANK_RANGE_ONLY_PARITY.md)
22. [W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md)
23. [W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md)
24. [W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md)
25. [W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md)

## Active Role Split
1. [W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md) remains the live deferred/provider-family authority.
2. [W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md) remains the live RTD seam authority.
3. [W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md) remains the live V1 export provenance owner.
4. [W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md) remains the retained runtime carrier model.
5. [W050_DEFERRED_CURRENT_VERSION_SURFACE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md) remains the canonical deferred tracker.
6. [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md) remains the parked non-deferred tracker and currently records reopened ordinary-operator, power, numeric-comparison, and multi-area reference rows.
7. [W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md) remains the parked Lean reconciliation authority.
8. [W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md) is the next substantive execution target.
9. [W072_BUG_INTAKE_ROOT_CAUSE_AND_REGRESSION_STREAM_PROTOCOL.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W072_BUG_INTAKE_ROOT_CAUSE_AND_REGRESSION_STREAM_PROTOCOL.md) is the local process authority for bug intake, duplicate linkage, canonical bug streams, and regression-family tracking.
10. [W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md) owns the current operator value-surface and array-lift seam follow-up opened from `HANDOFF-OXFUNC-001` and `BUG-FUNC-001`.
11. [W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md) owns the broadened ordinary-operator broadcast reconciliation opened from local Excel comparison and `BUG-FUNC-002`.
12. [W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md) owns the first-class same-sheet multi-area reference seam correction opened from the current OxFml upstream note and `BUG-FUNC-003`.
13. [W076_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W076_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md) owns the OxFunc-side Style A follow-up from `HANDOFF-OXFUNC-002`, moving same-sheet multi-area value materialization out of OxFml-local helper logic and into OxFunc resolver-driven semantics.
14. [W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md) owns the corpus follow-on from `HANDOFF-OXFUNC-003`, including the no-action IF correction and the live numeric comparison tolerance family split captured in `BUG-FUNC-004`.
15. [W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md) owns the shared `POWER` / `OP_POWER` zero-to-zero parity correction opened from live Excel replay and tracked in `BUG-FUNC-005`.
16. [W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md) owns the reopened lookup-family lane where live Excel spills array-valued lookup needles for `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP`, while adjacent `XLOOKUP` risk remains an explicit follow-on under `BUG-FUNC-006`.
17. [W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md) owns the immediate text-slice spill correction for `LEFT` / `RIGHT` / `MID` plus the bounded systematic review seed for broader function-array-support investigation under `BUG-FUNC-007`.
18. [W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md) owns the reopened `RATE` mortgage-style omitted-guess lane where Excel returns a small positive periodic rate but the local default-guess solver path currently fails with `#NUM!` under `BUG-FUNC-009`.
19. [W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md) owns the exact-shape decomposition change where OxFunc stops shipping a production locale-format parser/formatter path and instead requires OxFml/FEC to supply the concrete capability bundle through the typed seam.
20. [W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md) owns the reopened sort-family omission/defaulting lane where `SORT({2;3;7;5},,-1)` and adjacent `SORTBY(..., by_array,)` must treat syntactic omission the same as absent optional controls rather than surfacing `#VALUE!`.
21. [W084_COUNTBLANK_RANGE_ONLY_PARITY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W084_COUNTBLANK_RANGE_ONLY_PARITY.md) owns the reopened `COUNTBLANK` parity lane where live Excel accepts true ranges but rejects array-valued substitutes with `#VALUE!`, while `COUNT` / `COUNTA` / `ROWS` / `COLUMNS` remain explicit contrast controls rather than part of the same narrowing.
22. [W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md) owns the reopened reshape-family lane where omitted leading row-counts in `TAKE(...,,n)` and `DROP(...,,n)` must default to all rows rather than surfacing `MissingArg` / `#VALUE!`.
23. [W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md) owns the reopened exact-value accuracy lane where current local `NORM.DIST` and `NORM.INV` approximations drift from live Excel `Value2` on bounded current-baseline witnesses.
24. [W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md) owns the reopened `XIRR` precision lane where the current local iterative solve drifts from live Excel `Value2` on a bounded current-baseline cashflow/date witness.
25. [W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md) owns the smart-fuzzer pilot lane for compact telemetry, Excel throughput measurement, static metadata/risk indexing, bounded candidate generation, local-vs-Excel typed comparison, and mismatch promotion through the ordinary bug stream.

## Use These Instead
1. Use [PARKED_CURRENT_BASELINE_20260401.md](/C:/Work/DnaCalc/OxFunc/docs/PARKED_CURRENT_BASELINE_20260401.md) for the parked non-deferred baseline summary.
2. Use [WORKSET_REGISTER.md](/C:/Work/DnaCalc/OxFunc/docs/WORKSET_REGISTER.md) for ordered workset truth.
3. Use [BEADS.md](/C:/Work/DnaCalc/OxFunc/docs/BEADS.md) for the local bead method.
4. Use [IN_PROGRESS_FEATURE_WORKLIST.md](/C:/Work/DnaCalc/OxFunc/docs/IN_PROGRESS_FEATURE_WORKLIST.md) only as a high-level feature map.
5. Use [HISTORY.md](/C:/Work/DnaCalc/OxFunc/docs/HISTORY.md) and tag `OxFunc_V1` for removed workset packets and retired migration aids.
6. Use [W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md) only as closed framework provenance, not as active execution authority.
7. Use [W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md](/C:/Work/DnaCalc/OxFunc/docs/worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) only as closed migration provenance, not as active workset authority.
