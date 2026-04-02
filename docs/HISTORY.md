# OxFunc History Pointer

Status: `active_history_pointer`
Last updated: 2026-04-02
Tag anchor: `OxFunc_V1`

## Purpose
This file is the compact live pointer for historical OxFunc execution packets
and migration-era notes that have been removed from the active tree under
`W070`.

Use git history and the `OxFunc_V1` tag for the removed packet contents.
Do not treat this file as a live execution tracker.

## Historical Blocker Ledger
`CURRENT_BLOCKERS.md` has been removed from the active tree.

Historical blocker provenance now survives through:
1. git history,
2. tag `OxFunc_V1`,
3. blocker identifiers such as `BLK-FN-*` that still appear in retained historical records.

Interpretation rule:
1. `BLK-FN-*` identifiers in retained historical execution records and runtime notes are provenance only,
2. live blocker truth belongs in `.beads/`,
3. no active repo workflow should recreate `CURRENT_BLOCKERS.md` as a live surface.

## Historical Migration Register
The temporary migration triage register used during `W070` has been removed
from the active tree.

Historical migration-triage provenance now survives through:
1. this history pointer,
2. [docs/worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md](worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) as a closed migration packet,
3. git history,
4. tag `OxFunc_V1`.

Interpretation rule:
1. no active workflow should recreate `docs/W070_ACTIVE_TREE_TRIAGE_REGISTER.csv` as a live tracker,
2. active execution, blocker, and ready-state truth belongs in `.beads/`,
3. active document ownership and work ordering belong in the surviving doctrine and register surfaces.

## Phase E Wave 1 Removed Surfaces
The following historical packets and migration-era helper notes were removed
from `main` in `W070` Phase E wave 1:

1. `docs/DOC_OWNERSHIP_RECOMMENDATION.md`
2. `docs/FOUNDATION_EDITOR_PROMPTS_FROM_OXFUNC.md`
3. `docs/function-lane/NOTES_FOR_OXFML.md`
4. `docs/worksets/W000_KICKOFF_PROGRAM_W001_W006.md`
5. `docs/worksets/W001_PI_END_TO_END_SLICE.md`
6. `docs/worksets/W002_FLOATING_POINT_CHARACTERIZATION.md`
7. `docs/worksets/W003_VALUE_UNIVERSE_AND_EXTENDED_TYPES.md`
8. `docs/worksets/W004_COERCION_AND_REFERENCE_RESOLUTION_PRIMITIVES.md`
9. `docs/worksets/W005_ABS_FULL_FORMALITY.md`
10. `docs/worksets/W006_XMATCH_DETERMINISTIC_QUIRKS.md`
11. `docs/worksets/W007_STRING_CHARACTERIZATION.md`
12. `docs/worksets/W008_STRING_W8_1_CHECKLIST.md`
13. `docs/worksets/W009_XLL_ADDIN_BRIDGE.md`
14. `docs/worksets/W010_TEN_FUNCTION_MIXED_SEAMS.md`
15. `docs/worksets/W011_XLL_REGISTRATION_FLAGS_EVIDENCE.md`
16. `docs/worksets/W012_MODERATE_FUNCTION_EXPANSION.md`
17. `docs/worksets/W013_DECEPTIVELY_SIMPLE_BOUNDARY_FUNCTIONS.md`
18. `docs/worksets/W014_IMPLICIT_INTERSECTION_OPERATOR.md`
19. `docs/worksets/W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md`
20. `docs/worksets/W016_BULK_NON_INTERESTING_FUNCTIONS_AND_OPERATORS.md`
21. `docs/worksets/W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md`
22. `docs/worksets/W018_REPLAY_APPLIANCE_PACKET_ADAPTER_BASELINE.md`
23. `docs/worksets/W019_PACKET_WITNESS_DISTILLATION_AND_RETENTION_BASELINE.md`
24. `docs/worksets/W020_OXFUNC_REPLAY_BUNDLE_LAYOUT_AND_INDEX_BASELINE.md`
25. `docs/worksets/W021_W15_FIRST_LIVE_REPLAY_ADAPTER_RUN_BASELINE.md`

## Active Replacements
Use these live surfaces instead of the removed wave-1 packets:

1. [docs/WORKSET_REGISTER.md](WORKSET_REGISTER.md) for current ordered workset truth.
2. [docs/worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md](worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) for migration policy and archive-wave rationale.
3. [docs/PARKED_CURRENT_BASELINE_20260401.md](PARKED_CURRENT_BASELINE_20260401.md) for the parked non-deferred baseline summary.
4. Current function-lane contract, execution-record, scenario, and evidence-registry surfaces for surviving semantic and replay claims.

## Phase E Wave 2 Removed Surfaces
The following historical provenance surfaces were removed from `main` in `W070`
Phase E wave 2:

1. `docs/worksets/W022_*.md` through `docs/worksets/W040_*.md`
2. `docs/worksets/W042_DEFERRED_CALLABLE_SEAM_FIELD_LOCK_AND_HIGHER_ORDER_EVIDENCE.md`
3. `docs/handoffs/HANDOFF_SHARED_INTERFACE_FREEZE_PROMOTION_TO_OXFML_V1.md`
4. `docs/handoffs/HANDOFF_W014_IMPLICIT_INTERSECTION_TO_OXFML.md`
5. `docs/handoffs/HANDOFF_W015_CELL_INFO_HOST_QUERY_TO_OXFML.md`
6. `docs/handoffs/HANDOFF_W052_UNARY_NEGATIVE_AND_BLANK_SINGLE_CELL_TO_OXFML.md`
7. `docs/upstream/OXFUNC_OXFML_SEAM_REQUIREMENTS_CONSOLIDATED.md`
8. `docs/function-lane/W16_BATCH*.md`
9. `docs/function-lane/W16_NON_INTERESTING_REMAINING_*.csv`

## Wave 2 Active Replacements
Use these live surfaces instead of the removed wave-2 provenance packets:

1. [docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md](function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md) for the parked shared-interface model.
2. [docs/upstream/NOTES_FOR_OXFML.md](upstream/NOTES_FOR_OXFML.md) for the live OxFml observation ledger.
3. [docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md](worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md), [docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md](worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md), [docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md](worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md), [docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md](worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md), [docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md](worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md), [docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md), and [docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md](worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md) for the surviving current authorities.
4. `OxFunc_V1` for the archived contents of the removed worksets, handoffs, and helper inventories.

## Phase E Wave 3 Removed Surfaces
The following late closed packets and bridge-contract residue were removed from
`main` in `W070` Phase E wave 3:

1. `docs/worksets/W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`
2. `docs/worksets/W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM.md`
3. `docs/worksets/W047_TYPED_CONTEXT_AND_QUERY_BUNDLE_FREEZE.md`
4. `docs/worksets/W048_RETURN_SURFACE_AND_PUBLICATION_HINT_FREEZE.md`
5. `docs/worksets/W052_SUMIF_CRITERIA_FAMILY_COMPLETION.md`
6. `docs/worksets/W053_LOW_ORDER_PUBLICATION_DRIFT_ASINH_PMT.md`
7. `docs/function-lane/XLL_ADDIN_BRIDGE_SHIM_CONTRACT_PRELIM.md`
8. `docs/function-lane/FUNCTION_SLICE_TYPED_CONTEXT_AND_QUERY_BUNDLE_CONTRACT_PRELIM.md`
9. `docs/function-lane/FUNCTION_SLICE_RETURN_SURFACE_AND_PUBLICATION_HINT_CONTRACT_PRELIM.md`

## Wave 3 Active Replacements
Use these live surfaces instead of the removed wave-3 packets:

1. [docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md](worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md) for the retained runtime carrier and the summarized shared-freeze reading.
2. [docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md](function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md) for the parked shared-interface model text.
3. The surviving family contract and evidence surfaces:
   - [docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_OP_SPILL_REF_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_OP_SPILL_REF_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md](function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md)
4. [docs/function-lane/XLL_ADDIN_BRIDGE_EXECUTION_RECORD.md](function-lane/XLL_ADDIN_BRIDGE_EXECUTION_RECORD.md), [docs/function-lane/XLL_ADDIN_BRIDGE_REGISTRATION_NOTES.md](function-lane/XLL_ADDIN_BRIDGE_REGISTRATION_NOTES.md), and [docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md](function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md) for the surviving XLL bridge truth.
5. `OxFunc_V1` for the archived contents of the removed worksets and bridge prelims.

## Phase E Wave 4 Removed Surfaces
The following late closed packet-local evidence tails and residual historical
families were removed from `main` in `W070` Phase E wave 4:

1. `docs/worksets/W055_*.md` through `docs/worksets/W068_*.md`
2. `docs/function-lane/W55_EXECUTION_RECORD.md`
3. `docs/function-lane/W56_EXECUTION_RECORD.md`
4. `docs/function-lane/W56_GROUPED_AGGREGATION_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W56_GROUPED_AGGREGATION_SCENARIO_MANIFEST_SEED.csv`
6. `docs/function-lane/W57_PACKET_REGISTER.csv`
7. `docs/function-lane/W58_EXECUTION_RECORD.md`
8. `docs/function-lane/W58_GROUPED_ROW_NORMALIZATION_MAP.csv`
9. `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv`
10. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
11. `docs/function-lane/W59_*` through `docs/function-lane/W68_*` packet-local execution, runtime-requirement, and scope-reconciliation artifacts

## Wave 4 Active Replacements
Use these live surfaces instead of the removed wave-4 packet locals:

1. [docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md) for the parked non-deferred completion summary and surviving zero-backlog authority.
2. [docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md](function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md) for the surviving evidence-id anchors over the removed packet chain.
3. The surviving function-family contract surfaces, including:
   - [docs/function-lane/FUNCTION_SLICE_GROUPBY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_GROUPBY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_PIVOTBY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_PIVOTBY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_ENGINEERING_CONVERSIONS_AND_BESSEL_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_ENGINEERING_CONVERSIONS_AND_BESSEL_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_COMPLEX_NUMBER_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_COMPLEX_NUMBER_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_A_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_A_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_DATE_TIME_AND_BUSINESS_DAY_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_DATE_TIME_AND_BUSINESS_DAY_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_FINANCIAL_CORE_MISC_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_FINANCIAL_CORE_MISC_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CURRENT_BASELINE_PROMOTION_PRELIM.md](function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CURRENT_BASELINE_PROMOTION_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_MATH_MATRIX_AND_ROUNDING_FAMILY_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_MATH_MATRIX_AND_ROUNDING_FAMILY_CONTRACT_PRELIM.md)
   - [docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md](function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md)
4. The retained active native-evidence subset:
   - `docs/function-lane/W56_GROUPED_AGGREGATION_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W59_SCENARIO_MANIFEST_SEED.csv` through `docs/function-lane/W68_SCENARIO_MANIFEST_SEED.csv`
   - `tools/w56-probe/` through `tools/w68-probe/`
5. The surviving runtime, formal, and export surfaces under `crates/oxfunc_core/`, `formal/lean/OxFunc/Functions/`, and [docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv](function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv).
6. `OxFunc_V1` for the archived contents of the removed packet-local evidence tails.
