# IN_PROGRESS_FEATURE_WORKLIST.md - OxFunc

Status: `active_feature_map`
Last updated: 2026-04-08

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
3. detailed `V2` schema and witness-framework work belongs to [W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md),
4. the remaining full-surface population work belongs to [W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md).

## Active Feature Map

### IP-01 Parked Function Surface
- Current state: the non-deferred current-version surface remains largely parked, but a concrete ordinary-operator gap has reopened the active current-gap lane for array-involved binary arithmetic.
- Canonical owner: [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md), [W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md), and [W050_DEFERRED_CURRENT_VERSION_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W050_DEFERRED_CURRENT_VERSION_SURFACE.md).

### IP-02 Semantic Witness Snapshot V2 Framework
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

### IP-09 Semantic Witness Full-Surface Population
- Current state: planned successor execution lane for filling the remaining supported non-deferred witness rows.
- Canonical owner: [W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md), [WORKSET_REGISTER.md](C:\Work\DnaCalc\OxFunc\docs\WORKSET_REGISTER.md), and `.beads/`.

### IP-10 Ordinary Operator Value-Surface Expansion
- Current state: active cross-repo follow-up with the local OxFunc broadcast pass validated and handed off; the remaining open lane is landed-ref promotion plus downstream OxFml acknowledgment/removal of the temporary fallback under `HO-FN-005`.
- Canonical owner: [W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md), [W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md), [BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md), [BUG-FUNC-002_ordinary_operator_broadcast_semantics_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-002_ordinary_operator_broadcast_semantics_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-11 Multi-Area Reference Seam
- Current state: active cross-repo seam follow-up with the local OxFunc first-class `MultiArea` identity correction and the Style A value-materialization implementation both now validated locally; the remaining open lane is landed-ref promotion plus downstream OxFml acknowledgment under `HO-FN-006` and `HO-FN-007`.
- Canonical owner: [W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md), [W076_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W076_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md), [BUG-FUNC-003_multi_area_reference_seam_collapses_to_area_string.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-003_multi_area_reference_seam_collapses_to_area_string.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-12 Numeric Comparison Tolerance Follow-On
- Current state: active local follow-up from the corpus replay. `IF` / `IFS` empty-text condition is now pinned as no local bug, while ordinary compare operators, criteria/database selection, and `SWITCH` reopen under one numeric-comparison tolerance packet. The current local fix is validated against both the original `0.1+0.2` rows and the stronger arithmetic-generated 15-digit boundary rows; the remaining open lane is landed-ref promotion plus downstream OxFml acknowledgment under `HO-FN-008`.
- Canonical owner: [W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md), [BUG-FUNC-004_numeric_comparison_tolerance_family_split.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-004_numeric_comparison_tolerance_family_split.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-13 Power Zero-To-Zero Parity
- Current state: active local follow-up from live Excel replay. `POWER(0,0)` and `0^0` are now pinned as `#NUM!` in Excel, reopening the shared power-kernel lane because the pre-fix local runtime/formal path published `1`. The local correction and focused validation are on the working tree; the remaining open lane is landed-ref promotion.
- Canonical owner: [W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md), [BUG-FUNC-005_power_zero_to_zero_diverges_from_excel.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-005_power_zero_to_zero_diverges_from_excel.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-14 Lookup-Family Array Lookup-Value Lifting
- Current state: active local follow-up from live Excel replay. `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP` are now pinned as array-lifting over an array-valued `lookup_value`, reopening the lookup family because the pre-fix local surfaces rejected or mishandled that lane. The working-tree correction now covers those four functions; adjacent `XLOOKUP` risk remains open rather than silently bundled into the same closure claim.
- Canonical owner: [W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md), [BUG-FUNC-006_lookup_selection_array_lookup_value_lifting_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-006_lookup_selection_array_lookup_value_lifting_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-15 Function Array-Support Review
- Current state: active local review seed. `LEFT`, `RIGHT`, and `MID` are pinned as spilling over array-valued count/start inputs, and the first bounded follow-on `W066` batch now also pins spill lanes for `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, `REPT`, `TEXTAFTER`, and `TEXTBEFORE`. The working-tree correction covers those rows, while the next review batch and the broader systematic review remain open.
- Canonical owner: [W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md), [BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md), [BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-16 RATE Default-Guess Convergence
- Current state: active local follow-up from direct replay. Live Excel returns `0.004166644536345589` for `RATE(360,-1073.64,200000)` and the local `W081` correction now matches that lane on the working tree through a bounded bracket-and-bisection fallback around the existing secant path. The remaining open lanes are landed-ref promotion and a broader adjacent omitted-guess scan.
- Canonical owner: [W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md), [BUG-FUNC-009_rate_default_guess_solver_no_convergence.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-009_rate_default_guess_solver_no_convergence.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-17 Locale/Format Seam Ownership Realignment
- Current state: active cross-repo seam follow-up with the immediate leak now inventoried concretely. OxFunc still exports `en_us_context()` / `current_excel_host_context()` plus the local shim implementation, the XLL add-in still constructs `current_excel_host_context()` directly, and OxFml tests still import `en_us_context()` broadly as a convenience bundle. `W082` and `HO-FN-009` now own the migration from that OxFunc-owned shim surface onto caller-supplied capability bundles with no backward-compatible OxFunc fallback.
- Canonical owner: [W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md), [HO-FN-009_locale_format_seam_ownership_realignment.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HO-FN-009_locale_format_seam_ownership_realignment.md), and [LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md](C:\Work\DnaCalc\OxFunc\docs\function-lane\LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md).

### IP-18 Dynamic-Array Sort Omitted-Argument Defaulting
- Current state: active local follow-up from direct OxFunc replay. `SORT({2;3;7;5},,-1)` was pinned as a real local omission/defaulting bug: the pre-fix committed ref surfaced `Preparation(MissingArg)` / `#VALUE!`, while explicit `sort_index=1` succeeded. The local `W083` correction now normalizes omitted sort-family controls for `SORT` and adjacent `SORTBY` on the working tree; the remaining open lanes are landed-ref promotion and bounded adjacent review of other reshape functions with optional defaulted controls.
- Canonical owner: [W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md), [BUG-FUNC-010_dynamic_array_sort_family_omitted_optional_argument_defaulting_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-010_dynamic_array_sort_family_omitted_optional_argument_defaulting_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-19 COUNTBLANK Range-Only Parity
- Current state: local correction and focused validation are now recorded on the working tree. `COUNTBLANK` was pinned as accepting true ranges but rejecting array-valued substitutes with `#VALUE!`, while `COUNT`, `COUNTA`, `ROWS`, and `COLUMNS` remain explicit contrast controls rather than part of the same narrowing. `W084` now rejects direct array-valued substitutes locally while preserving true-range blank counting; landed-ref promotion and the bounded adjacent policy review remain open.
- Canonical owner: [W084_COUNTBLANK_RANGE_ONLY_PARITY.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W084_COUNTBLANK_RANGE_ONLY_PARITY.md), [BUG-FUNC-011_countblank_range_only_parity_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-011_countblank_range_only_parity_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-20 TAKE/DROP Omitted Leading-Count Parity
- Current state: local correction and focused validation are now recorded on the working tree. `TAKE(...,,n)` and `DROP(...,,n)` were pinned as real local omission/defaulting bugs: Excel keeps all rows while slicing columns, while the reported ref still treated the leading row-count as required and surfaced `Preparation(MissingArg)`. `W085` now normalizes omitted leading row-count handling for the bounded `TAKE` / `DROP` lane, adds focused omission-shape coverage plus W39 witness rows, and keeps W39/W51 honest until the correction lands on a committed ref.
- Canonical owner: [W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md), [BUG-FUNC-012_take_drop_omitted_leading_count_parity_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-012_take_drop_omitted_leading_count_parity_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-21 Normal-Distribution Exact-Value Accuracy
- Current state: local correction and focused validation are now recorded on the working tree. `W086` replaced the coarse local error-function helper with `libm::erf`, bringing the bounded `NORM.DIST(0,0,1,TRUE)` and `NORM.INV(0.975,0,1)` witnesses onto the live Excel `Value2` targets `0.5` and `1.9599639845400536`, while also tightening the adjacent `Z.TEST` witness to the current Excel observable. Landed-ref promotion remains open.
- Canonical owner: [W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md), [BUG-FUNC-013_normal_distribution_exact_value_accuracy_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-013_normal_distribution_exact_value_accuracy_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-22 XIRR Solver Precision Reconciliation
- Current state: exact live Excel `Value2` replay is now pinned on a widened adjacent multi-cashflow positive-root `XIRR` guess matrix. That replay confirms Excel publishes exact bisection midpoints rather than the tightest `XNPV` root, and those targets lie on the same bracket path as the current OxFunc implementation. Focused local verification now shows the current working-tree solver matches that bounded matrix, preserves the earlier two-cashflow `W37` special case plus negative-root lanes, and is waiting on landed-ref promotion rather than further bounded `XIRR` semantic repair.
- Canonical owner: [W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md), [BUG-FUNC-014_xirr_solver_precision_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-014_xirr_solver_precision_drift.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

## Status Vocabulary
- `planned`: accepted lane, no active execution claim implied here.
- `active`: live lane with current owner surfaces.
- `parked`: current baseline or parked authority exists; reopen only by explicit future work.

Current reading:
1. `IP-01` is `active`,
2. `IP-02`, `IP-03`, `IP-04`, `IP-05`, and `IP-06` are `active`,
3. `IP-07` is `planned`,
4. `IP-08` is `parked`,
5. `IP-09` is `planned`,
6. `IP-10`, `IP-11`, `IP-12`, `IP-13`, `IP-14`, `IP-15`, `IP-16`, `IP-17`, `IP-18`, `IP-19`, `IP-20`, `IP-21`, and `IP-22` are `active`.
