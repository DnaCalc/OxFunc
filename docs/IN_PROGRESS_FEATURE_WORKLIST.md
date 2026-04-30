# IN_PROGRESS_FEATURE_WORKLIST.md - OxFunc

Status: `active_feature_map`
Last updated: 2026-04-30

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
- Current state: closed historical follow-up. Fresh Excel COM replay on 2026-04-29 confirmed `=0^0` and `=POWER(0,0)` still return `#NUM!` on Excel 16.0 build 19929, and the local Rust/Lean correction is landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`; POWER is no longer a current open gap.
- Canonical owner: [W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md), [BUG-FUNC-005_power_zero_to_zero_diverges_from_excel.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-005_power_zero_to_zero_diverges_from_excel.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-14 Lookup-Family Array Lookup-Value Lifting
- Current state: closed historical follow-up. `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP` array-valued `lookup_value` corrections are landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`; fresh 2026-04-29 replay pinned adjacent `XLOOKUP` shape-preserving multi-needle behavior, and the local `XLOOKUP` correction is landed on `b1faa5e8f08cd534601dc57bf79a9fed3ff26972`.
- Canonical owner: [W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md), [BUG-FUNC-006_lookup_selection_array_lookup_value_lifting_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-006_lookup_selection_array_lookup_value_lifting_gap.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-15 Function Array-Support Review
- Current state: successor sweep executed with target still partial. The `W080` seed packet is closed for its declared scope: `LEFT` / `LEFTB`, `RIGHT` / `RIGHTB`, and `MID` / `MIDB` are landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`; the first bounded follow-on `W066` batch for `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, `REPT`, `TEXTAFTER`, and `TEXTBEFORE` is landed on `2e818f03a71ba393690275a7fb437ddd9a6bf760`; and the second bounded follow-on batch for `FIND` / `FINDB`, `SEARCH` / `SEARCHB`, `REPLACE` / `REPLACEB`, `PROPER`, and `SUBSTITUTE` is landed on `b1faa5e8f08cd534601dc57bf79a9fed3ff26972`. W090 tranche A then found and repaired the math scalar-numeric array-lift gap as `BUG-FUNC-017`; ref `0b966d0ee7c8ce4a327b0b3090f9a108248c37fd` reran `34/34` exact typed bit matches against Excel `16.0` build `19929`. The generated 2026-04-30 successor pass ran `139` cases across `8` category tranches and promoted `BUG-FUNC-018`, `BUG-FUNC-019`, and `BUG-FUNC-020`; the W090 repair replay now closes those array-admission/aggregate/panic streams with `98/139` exact typed bit matches and `0` local harness blockers. The remaining `41` mismatches are no-tolerance numeric exactness drift in statistical kernels and are tracked as `BUG-FUNC-021` / `oxf-simj`. The remaining supported array-support space is not claimed reviewed.
- Canonical owner: [W090_FUNCTION_ARRAY_SUPPORT_SYSTEMATIC_SWEEP.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W090_FUNCTION_ARRAY_SUPPORT_SYSTEMATIC_SWEEP.md), [W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md), [BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md), [BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md), [BUG-FUNC-016_text_search_replace_array_support_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-016_text_search_replace_array_support_gap.md), [BUG-FUNC-018_successor_scalar_parameter_array_lift_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-018_successor_scalar_parameter_array_lift_gap.md), [BUG-FUNC-019_complex_aggregate_array_literal_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-019_complex_aggregate_array_literal_gap.md), [BUG-FUNC-020_expand_array_pad_with_panic.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-020_expand_array_pad_with_panic.md), [BUG-FUNC-021_w090_statistical_numeric_exactness_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-021_w090_statistical_numeric_exactness_drift.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-16 RATE Default-Guess Convergence
- Current state: active local follow-up from direct replay. Live Excel returns `0.004166644536345589` for `RATE(360,-1073.64,200000)` and the local `W081` correction now matches that lane on the working tree through a bounded bracket-and-bisection fallback around the existing secant path. The remaining open lanes are landed-ref promotion and a broader adjacent omitted-guess scan.
- Canonical owner: [W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md), [BUG-FUNC-009_rate_default_guess_solver_no_convergence.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-009_rate_default_guess_solver_no_convergence.md), and [W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md).

### IP-17 Locale/Format Seam Ownership Realignment
- Current state: closed historical follow-up. OxFunc removed the old ordinary locale-context convenience constructors from `oxfunc_core`, leaving the local parser/formatter only as explicit `#[cfg(test)]` support; the XLL add-in now supplies a caller-owned host context and delegates parse/render behavior to Excel through `xlfEvaluate`; and OxFml acknowledged `HO-FN-009` on 2026-04-29 with no OxFunc-side seam change requested.
- Canonical owner: [W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md), [HO-FN-009_locale_format_seam_ownership_realignment.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HO-FN-009_locale_format_seam_ownership_realignment.md), and [LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md](C:\Work\DnaCalc\OxFunc\docs\function-lane\LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md).

### IP-18 Smart-Fuzzer Differential Exploration
- Current state: active smart-fuzzer lane. `W088` supplied the pilot substrate; `W089` has now run the first comprehensive manifest-seed exploration against live Excel COM. The 2026-04-30 pass recorded `339` broad seed cases, `139` successor array-support replay cases, a `1,000,000`-case local finance reference run, and an Excel throughput benchmark. It fixed the `ABS` array-lift gap as `BUG-FUNC-022`, corrected stale POWER known-deviation classification in the smart-fuzzer inventory, and promoted residual statistical exactness drift to `BUG-FUNC-021` plus non-statistical/matrix drift to `BUG-FUNC-023`.
- Canonical owner: [W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md), [W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md), [COMPREHENSIVE_SMART_FUZZER_RUN_20260430.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\COMPREHENSIVE_SMART_FUZZER_RUN_20260430.md), [BUG-FUNC-021_w090_statistical_numeric_exactness_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-021_w090_statistical_numeric_exactness_drift.md), [BUG-FUNC-022_abs_unary_array_lift_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-022_abs_unary_array_lift_gap.md), [BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md), [smart-fuzzer\README.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\README.md), and [SMART_FUZZER_DESIGN.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\SMART_FUZZER_DESIGN.md).

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
6. `IP-10`, `IP-11`, `IP-12`, `IP-13`, and `IP-16` are `active`.
7. `IP-14` and `IP-17` are `parked`.
8. `IP-15` is `planned` for successor tranches after the closed first W090 cycle.
9. `IP-18` is `active`.
