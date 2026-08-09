# IN_PROGRESS_FEATURE_WORKLIST.md - OxFunc

Status: `active_feature_map`
Last updated: 2026-07-10

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

### IP-10 Ordinary Operator Value-Surface Expansion — DONE (2026-06-20)
- Current state: closed. The operator broadcast surface is landed on `main` and re-verified bit-exact vs live Excel 16.0 b20026 (21-case sweep). The downstream half is also discharged: OxFml dispatches operators to OxFunc's `OP_*` surface with no temporary array-operator fallback remaining (verified). `BUG-FUNC-001` / `BUG-FUNC-002` closed, `HO-FN-005` resolved, catalog G2 `OP_*` row removed.
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
- Current state: active smart-fuzzer lane. `W088` supplied the pilot substrate; `W089` has now run the first comprehensive manifest-seed exploration against live Excel COM. The 2026-04-30 pass recorded `339` broad seed cases, `139` successor array-support replay cases, a `1,000,000`-case local finance reference run, and an Excel throughput benchmark. It fixed the `ABS` array-lift gap as `BUG-FUNC-022`, corrected stale POWER known-deviation classification in the smart-fuzzer inventory, and promoted residual statistical exactness drift to `BUG-FUNC-021` plus non-statistical/matrix drift to `BUG-FUNC-023`. The `BUG-FUNC-023` follow-up repaired `VDB`; its attempted `MINVERSE(5)` / `MMULT(5,2)` scalar collapse was later reverted when nested `TYPE` probes proved an internal `1x1` array, so final-cell appearance is now Category-1 publication work (`CSC-0024`/`CSC-0025`, `HO-FN-010`). Numeric successors split to `BUG-FUNC-024` (`BESSELY`, subsequently signed off) and `BUG-FUNC-025` (`MINVERSE` matrix numeric exactness). The 2026-05-09 broad scalar exploration extended W092 outside the manifest-seed plateau by walking `~50` single/two-arg numeric scalar functions across per-family numeric bands; aggregate `11.5M` local cases plus `4,200` Excel comparisons surfaced `BUG-FUNC-027` covering 15 recurring mismatch subclasses (GAMMALN tiny pos `+Inf`, GAMMA tiny non-zero false pole, SINH/COSH overflow, POWER overflow, PERMUTATIONA overflow, FISHERINV saturation, MROUND zero-num, MOD `#NUM!` threshold, trig large-arg `#NUM!`, trig precision drift, ATANH near-boundary, ACOTH/ACOSH near-1, ATAN2 magnitude spread).
- Canonical owner: [W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md), [W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md), [COMPREHENSIVE_SMART_FUZZER_RUN_20260430.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\COMPREHENSIVE_SMART_FUZZER_RUN_20260430.md), [BROAD_SCALAR_EXPLORATION_2026-05-09.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\BROAD_SCALAR_EXPLORATION_2026-05-09.md), [BUG-FUNC-021_w090_statistical_numeric_exactness_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-021_w090_statistical_numeric_exactness_drift.md), [BUG-FUNC-022_abs_unary_array_lift_gap.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-022_abs_unary_array_lift_gap.md), [BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md), [BUG-FUNC-024_bessely_current_baseline_exactness_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-024_bessely_current_baseline_exactness_drift.md), [BUG-FUNC-025_minverse_matrix_numeric_exactness_drift.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-025_minverse_matrix_numeric_exactness_drift.md), [BUG-FUNC-027_broad_scalar_invocation_space_findings.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-027_broad_scalar_invocation_space_findings.md), [smart-fuzzer\README.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\README.md), and [SMART_FUZZER_DESIGN.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\SMART_FUZZER_DESIGN.md).

### IP-19 Canonical Runtime Function Registry
- Current state: active intake from DNA OneCalc. OxFunc has acknowledged that the comprehensive function list, arity, signature, parameter descriptor, runtime UDF registration, and capability-overlay truth must be owned by OxFunc rather than by host-local lists or string-filled snapshot fields. The implementation lane is open under `W091`.
- Canonical owner: [W091_CANONICAL_RUNTIME_FUNCTION_REGISTRY.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W091_CANONICAL_RUNTIME_FUNCTION_REGISTRY.md), [OXFUNC_CANONICAL_RUNTIME_FUNCTION_REGISTRY_CONTRACT.md](C:\Work\DnaCalc\OxFunc\docs\function-lane\OXFUNC_CANONICAL_RUNTIME_FUNCTION_REGISTRY_CONTRACT.md), [HANDOFF-OXFUNC-004_canonical_runtime_function_registry.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HANDOFF-OXFUNC-004_canonical_runtime_function_registry.md), and [HO-FN-011_canonical_function_registry_consumption.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HO-FN-011_canonical_function_registry_consumption.md).

### IP-21 Full-Model Compiled Semantic Kernel Dispatch
- Current state: active cross-repo optimization seam with OxFunc local target satisfied and OxFml acknowledgement received. W096 added the OxFunc-local resolved function-call target, FEC provider bundle, and scratch surfaces, resolver-erased ABI, generated catalog-index dispatch backend, compatibility string wrapper, planning metadata, focused parity evidence, and OxFml handoff. OxFml accepts `HO-FN-016`, owns the longer optimizer path under W075, and reports initial consumption of `FunctionCallTarget`, `FunctionExecutionContextBundle`, and `FunctionCallScratch`; remaining optimizer lanes are downstream W075 work rather than an OxFunc semantic gap.
- Canonical owner: [W096_FULL_MODEL_COMPILED_SEMANTIC_KERNEL_DISPATCH.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W096_FULL_MODEL_COMPILED_SEMANTIC_KERNEL_DISPATCH.md), [HO-FN-016_function_call_target_and_index_dispatch.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HO-FN-016_function_call_target_and_index_dispatch.md), and `.beads/`.

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
10. `IP-19` is `active`.
11. `IP-20` is `active`.
12. `IP-21` is `active`.
13. `IP-22` is `active`.
14. `IP-23` is `proposed`.
15. `IP-24` is `active`.
16. `IP-25` is `active`.

## 2026-05-03 W091 OxFunc-local execution update

execution_state: `local_target_satisfied_downstream_ack_pending`

scope_completeness: `scope_partial`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. OxFml downstream acknowledgement and consumer migration remain outside OxFunc-local write authority.
2. DNA OneCalc downstream acknowledgement and UI migration remain outside OxFunc-local write authority.
3. Full workspace integration test remains affected by the pre-existing OxFml dev-test import mismatch for `oxfml_core::format::current_excel_host_context`.

OxFunc-local evidence:
1. `oxfunc_core::registry` exposes canonical built-in iteration, lookup, UDF registration, UDF unregistration, and capability overlays.
2. Built-in signature descriptors are generated from OxFunc seed/catalog truth and normalized against `FunctionMeta.arity`.
3. `tools/xll-addin/oxfunc_xll/export_specs.csv` was regenerated from OxFunc core.
4. `docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md` and `docs/handoffs/HO-FN-012_dnaonecalc_registry_consumption.md` provide sibling migration instructions.
5. `.beads/` W091 execution beads were closed with command evidence.

Validation evidence:
1. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib`: passed, 1266 passed, 1 ignored.
2. `cargo check --manifest-path crates/oxfunc_core/Cargo.toml --target wasm32-unknown-unknown --lib`: passed.
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml`: attempted and blocked by the pre-existing OxFml integration-test import mismatch noted above.

## 2026-05-04 W091 future-shape hardening update

execution_state: `local_target_satisfied_downstream_ack_pending`

scope_completeness: `scope_partial`

target_completeness: `target_complete`

integration_completeness: `partial`

open_lanes:
1. OxFml downstream removal of registry-like function-list surfaces remains outside OxFunc-local write authority.
2. DNA OneCalc downstream consumption of OxFunc registry metadata remains outside OxFunc-local write authority.
3. Parameter and function help descriptions remain a separate future corpus.

Additional OxFunc-local hardening:
1. registry entries now use owned `RegistryFunctionMeta` for runtime/UDF-safe function identifiers,
2. unchecked registry construction was replaced by `try_from_entries`,
3. context metadata seed generation is reproducible and expands combined historical rows to per-function IDs,
4. stale `legacy_arity_shape_note` references were removed from future-facing docs,
5. W44 V1 snapshot generation is guarded as legacy-only.

## 2026-05-04 OxFml W068 landing acknowledgement

OxFunc acknowledged `HANDOFF-OXFUNC-005` from OxFml W068 and marked HO-FN-011 acknowledged. Follow-up `HO-FN-013` was filed to document the owned `RegistryFunctionMeta` shape introduced after OxFml landed canonical registry consumption.

### 2026-05-04 W092 Spark-Guided Smart-Fuzzer Long-Run Update

W092 adds the Spark-guided long-run execution guide at [SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md). The guide is the controlling run document for repeated feedback-guided cycles and ambitious stop gates: catalog-axis saturation, sustained no-new-signal plateau, excessive untriaged finding pressure, all-path blockers, resource-safety stop, or user stop. The `2026-05-04` run reached the current no-new-signal plateau for available nonblocked generators; guide Section 2.3 records the stop-gate and promotion audit.

W092 follow-up repair update: `BUG-FUNC-018` reopened scalar-parameter
array-lift rows are validated in the current working tree by
`w092-bug-func-018-repair-max20x60-all-001`, which replayed `253` successor
cases as `211` exact typed bit matches plus `42` known residual rows in
existing exactness lanes, with `0` unexpected mismatches.

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_complete` for the W092 long-run objective through the current stop gate
3. `integration_completeness`: `integrated`
4. `open_lanes`: broader catalog-axis saturation, richer generator design, provider/context/locale/version fixtures, `BUG-FUNC-018` landed-ref promotion, `BUG-FUNC-021`, `BUG-FUNC-024`, `BUG-FUNC-025`, `BUG-FUNC-015`, and `HO-FN-010`.

### IP-20 UDF Registration And Name-Resolution Seam
- Current state: active design lane with the first OxFunc-local API tranche landed and `HO-FN-014` acknowledged by OxFml. W093 defines the source-neutral UDF registration contract and OxFml invalidation seam so XLL, VBA, JavaScript custom functions, Automation, and worksheet registered-external paths can converge on OxFunc registry truth without moving workbook defined names into OxFunc.
- Canonical owner: [W093_UDF_REGISTRATION_AND_NAME_RESOLUTION_SEAM.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W093_UDF_REGISTRATION_AND_NAME_RESOLUTION_SEAM.md), [OXFUNC_UDF_REGISTRATION_AND_REGISTRY_MUTATION_CONTRACT.md](C:\Work\DnaCalc\OxFunc\docs\function-lane\OXFUNC_UDF_REGISTRATION_AND_REGISTRY_MUTATION_CONTRACT.md), and [HO-FN-014_udf_registry_mutation_and_name_resolution_invalidation.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HO-FN-014_udf_registry_mutation_and_name_resolution_invalidation.md).

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: OxFml W074 formula-call registry lookup/cache invalidation evidence, Excel precedence oracle coverage, source-adapter detail, and broad UDF execution semantics.

### 2026-05-04 W093 Design Review Sweep

W093 design corrections after review:
1. `REGISTER.ID` / `CALL` descriptor-only state is adjacent registered-external seam state, not ordinary UDF function registration by default.
2. Bind-visible UDF registration uses immutable registry-backed snapshot identity/change sets rather than an unrelated second registry epoch.
3. Callable worksheet surface metadata remains separate from source-specific invocation target descriptors.
4. Rejected mutations return typed rejection outcomes without semantic snapshot advancement.
5. Collision/precedence and JavaScript custom-function metadata evidence are blockers before implementation promotion.
6. OxFml migration must cover formula-call binding/evaluation, not only editor help/completion.

### 2026-05-22 W093 OxFunc Repo-Local API Tranche

OxFunc added source-neutral UDF registry mutation packets and tests in
`oxfunc_core::registry`: `UdfRegistrationRequest`, typed registration and
unregistration results, `FunctionRegistrySnapshotIdentity`, `RegistryChangeSet`,
capability-aware execution profile fields, invocation-target descriptors kept
separate from callable surface metadata, and opaque `ReferenceLike`/host
reference carriers.

Validation evidence:
1. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib registry`: passed, `24` passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib`: passed, `1307` passed, `1` ignored.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --test oxfml_registered_external_interface_integration`: passed, `3` passed.
4. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml`: passed.

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: OxFml W074 formula-call registry lookup and invalidation, Excel oracle precedence matrix, source-adapter implementations, and broad UDF execution semantics.

### 2026-05-23 W093 / W056 Structured-Table ReferenceLike Guardrail

OxFunc narrowed the W056 table-carrier seam without taking TreeCalc selector
ownership. `ReferenceKind::Structured` targets may contain bracketed table
selector text without being rejected as external workbook references, and the
structured sparse-reader support now covers the first aggregate group plus
representative shared aggregate/statistical/logical/text, lookup/match,
criteria aggregate, extent/projection, and host-context lanes. The guardrail
test proves structured table data/header/totals/current-row carriers are
consumed through opaque `ReferenceResolver::resolve_reference_values(...)`
sparse readers and `HostInfoProvider` context APIs, without TreeCalc selector
parsing in OxFunc.

Validation evidence:
1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --test structured_table_reference_guardrails`: passed, `8` passed.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib`: passed,
   `1316` passed, `1` ignored.
4. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml`: passed.

Status axes:
1. `scope_completeness`: `scope_complete` for beads `oxf-ypq2.15` and
   `oxf-ypq2.16` as scoped to representative/shared-adapter sparse support and
   typed classification.
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: full sparse-reader metadata/replay promotion, whole-surface
   function-specific sparse parity evidence, dynamic-array/table-context
   semantics, caller-context lanes such as `ROW` / `COLUMN`, A1-only metadata
   lanes such as `CELL("address"|"row"|"col")`, reference operator
   range/intersection semantics, and downstream W056 retained table evidence
   outside OxFunc write authority.

### IP-21 Locale Profile Expansion
- Current state: OxFunc-local target satisfied with follow-up lanes split. OxFunc has acknowledged `BLK-FML-005` and the OxFunc-local W094 profile identity/constants slice now covers the DNA OneCalc ambient language-tag table through canonical profile ids, stable names, `LocaleProfileId::from_bcp47_language_tag(...)`, `LocaleProfileId::from_excel_lcid(...)`, short-date order/pattern facts, currency layout facts, and invariant format-code token-policy facts. Remaining work is tracked separately for downstream OxFml consumption, Excel storage/localized-function research, culture-profile seed mismatch triage, and broader locale semantic parity.
- Canonical owner: [W094_LOCALE_PROFILE_EXPANSION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W094_LOCALE_PROFILE_EXPANSION.md), [HANDOFF-OXFUNC-006_W070_LOCALE_PROFILE_EXPANSION_REQUEST.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HANDOFF-OXFUNC-006_W070_LOCALE_PROFILE_EXPANSION_REQUEST.md), and [locale_format.rs](C:\Work\DnaCalc\OxFunc\crates\oxfunc_core\src\locale_format.rs).

Validation evidence:
1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib profile`: passed, `8` passed, `0` failed, `1264` filtered out.

Status axes:
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: `oxf-swy6`, `oxf-mxwo`, `oxf-2nc0`, and broader locale semantic-parity sweeps.

### IP-23 Bit-Exact Re-Sweep Of Known Mismatches

- Current state: `proposed`. The 2026-05-09 broad-scalar smart-fuzzer cycle uncovered that comparator runs which pass numeric arguments through formula literal text introduce a hidden harness artefact (Excel's formula parser is not always correctly-rounded for long decimal literals). A direct re-replay of one cycle under the new cell-ref plumbing showed about `60%` of the prior `expected_formula_literal_encoding_drift` rows were genuine harness artefact and about `40%` were genuine kernel drift hidden under the loose tolerance; some `BUG-FUNC-*` bands have OxFunc *more* accurate than Excel under bit-exact comparison. W097 re-replays every existing OxFunc-vs-Excel exactness mismatch surface under cell-ref plumbing so each open and closed exactness stream's ULP magnitude is re-measured.
- Canonical owner: [W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md), [EXCEL_RUNNER_PLUMBING_NOTE.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\EXCEL_RUNNER_PLUMBING_NOTE.md), [KNOWN_MISMATCH_RESWEEP_PLAN.md](C:\Work\DnaCalc\OxFunc\smart-fuzzer\planning\KNOWN_MISMATCH_RESWEEP_PLAN.md), [BUG-FUNC-027_broad_scalar_invocation_space_findings.md](C:\Work\DnaCalc\OxFunc\docs\bugs\streams\BUG-FUNC-027_broad_scalar_invocation_space_findings.md), and the affected open/closed `BUG-FUNC-005`, `BUG-FUNC-013`, `BUG-FUNC-014`, `BUG-FUNC-015`, `BUG-FUNC-021`, `BUG-FUNC-024`, `BUG-FUNC-025` streams.

### IP-24 Kernel Metadata And Rich/Sparse Admission Profiles
- Current state: current-scope Rust bridge work landed locally. OxFunc accepts ownership of reduction-sensitive and error-collapse-sensitive kernel metadata, exact `NumericalReductionPolicy` semantics, exact `ErrorAlgebra` precedence semantics, an admission profile equivalent to `RichArgAccepted(required_capability_set)`, sparse-reader admission as a successor metadata lane, and prepared-package invalidation signals `semantic_kernel_metadata_version` and `arg_admission_metadata_version`. `RegistryFunctionMeta` now exposes semantic-kernel metadata/version, arg-admission metadata/version, and `IMAGE` `_webimage` producer capability keys; `render_registry_metadata_csv(...)` exports those fields for downstream consumers. `SUM` prepared aggregate evaluation exercises `SequentialLeftFold`; `ErrorAlgebra::CanonicalExcelLegacy` has a tested helper boundary; rich-admission capability mismatch has deterministic required/missing-key reporting; and `IMAGE` exposes adjacent runtime `producer_capability_set_keys` / `exercised_capability_keys` for successful `_webimage` results. Pairwise/Kahan replay fields, broad per-family error-collapse wiring, generic rich-argument consumers, generic rich producer protocol, and sparse support remain open.
- Canonical owner: [OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md](C:\Work\DnaCalc\OxFunc\docs\function-lane\OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md), [HANDOFF-CALC-003_OXFUNC_RECEIPT.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HANDOFF-CALC-003_OXFUNC_RECEIPT.md), [HANDOFF-CALC-004_OXFUNC_RECEIPT.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HANDOFF-CALC-004_OXFUNC_RECEIPT.md), and [NOTES_FOR_OXCALC.md](C:\Work\DnaCalc\OxFunc\docs\upstream\NOTES_FOR_OXCALC.md).

### IP-25 CalcValue End-To-End Migration
- Current state: active W099 execution lane is reopened at W099-015 corrective work. The initial W099-001 inventory mapped legacy `EvalValue`, `CallArgValue`, `PreparedArgValue`, `EvalArray`, `ArrayCellValue`, `ExtendedValue`, `LambdaValue`, old reference-provider, and `.target` surfaces to deletion batches; W099-002 through W099-014 landed the typed `ReferenceLike`, `ReferenceSystemProvider`, CalcValue, callable, cleanup, and OxFml/OxCalc W060 provider foundations. The first W099-015 pass incorrectly renamed several public value-like carriers to `FunctionValue`, `FunctionArg`, `PreparedValue`, `FunctionArray`, and `FunctionArrayCell` instead of deleting the carrier family; W099-015 remains open until those wrappers/adapters are removed from active code or reduced to non-value domain facts with native `CalcValue` / `CalcArray` payloads.
- Canonical owner: [W099_CALCVALUE_END_TO_END_MIGRATION.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W099_CALCVALUE_END_TO_END_MIGRATION.md), [W099_CALCVALUE_INVENTORY_AND_DELETION_LEDGER.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W099_CALCVALUE_INVENTORY_AND_DELETION_LEDGER.md), [W099_CALCVALUE_OCCURRENCE_LEDGER.csv](C:\Work\DnaCalc\OxFunc\docs\worksets\W099_CALCVALUE_OCCURRENCE_LEDGER.csv), and `.beads/` epic `oxf-im4m`.

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: W099-016 final cross-repo validation and any final naming/adapter audit items surfaced by that sweep.

### IP-22 REDUCE Lambda-Helper Hot-Loop Performance
- Current state: stopped at the OxFunc-local gate. W095 has local helper iteration changes so `MAP`, `REDUCE`, and `SCAN` consume iterable items lazily instead of materializing a full `Vec<PreparedArgValue>` upfront; `BYROW`/`BYCOL` avoid cloning an already-array source; `REDUCE` has a numeric-array fast path; `HSTACK`/`VSTACK` use borrowed/inline argument sources; `EvalArray` stores arrays with up to `8` cells inline; and `CallableInvoker::invoke_many(...)` exposes the OxFml-facing batching seam.
- Canonical owner: [W095_REDUCE_LAMBDA_HELPER_HOTLOOP_PERF.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W095_REDUCE_LAMBDA_HELPER_HOTLOOP_PERF.md), [HANDOFF-OXFUNC-007_reduce_hotloop_perf.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HANDOFF-OXFUNC-007_reduce_hotloop_perf.md), [HO-FN-015_callable_batching_invocation_seam.md](C:\Work\DnaCalc\OxFunc\docs\handoffs\HO-FN-015_callable_batching_invocation_seam.md), and [callable_helpers.rs](C:\Work\DnaCalc\OxFunc\crates\oxfunc_core\src\functions\callable_helpers.rs).

Validation evidence:
1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib callable_helpers`: passed, `29` passed, `0` failed, `1246` filtered out.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib hstack`: passed, `3` passed, `0` failed, `1269` filtered out.
4. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib vstack`: passed, `1` passed, `0` failed, `1271` filtered out.
5. `cargo test --release -p dnaonecalc-host --test mandelbrot_perf_probe -- --ignored --nocapture`: passed, `2` passed, `0` failed; after inline `EvalArray` storage the host-visible full-size print was `5.7998668s`.

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: OxFml `invoke_many(...)` specialization through `HO-FN-015`, post-OxFml-specialization DnaOneCalc perf replay, and landed-ref promotion.

### IP-26 Excel Numeric Core And Financial Exactness

- Current state: W109 is the active continuation of W108's exact-numeric work.
  `EXP`, `LN`, `LOG10`, `LOG`, `POWER`, the primary trigonometric family,
  XNPV, and the earlier W109 quick-win functions are signed off on the declared
  x86-64 reference baseline. The 2026-08-09 reconciliation identified distinct
  legacy x87 calculation graphs for `EFFECT`, `RRI`, and `NOMINAL`. EFFECT's
  post-truncation `u32::MAX` loop-to-raw-power dispatch is now live-pinned
  `160/160`; RRI's MIN_NORMAL/DAZ/equality/sign/zero-base/period-one composite
  is live-pinned and replays `5536/5536`. Their production repairs and
  discriminator pins pass the post-repair full suite (`1518` passed, `4`
  ignored) and Lean build (`492` jobs). The repairs and evidence landed in
  `876635e`; BUG-FUNC-043/044/045 and beads `oxf-jwh5.1/.2/.4` are closed
  signed off, and G6-12/G6-13/G6-14 are retired from the open catalog. This
  three-bug landing does not close the wider financial family or W109. A
  separate ACCRINT publication repair landed in `cd1f9fe`: the stored
  ordinary-f64 coupon and stored identified accrual fraction are multiplied
  only through `excel_x87_mul`. Retained b39/b40/b42 plus recaptured b43 and a
  fresh frozen held-out replay `146850/146850`; BUG-FUNC-030 is closed signed
  off, its already-closed bead carries successor evidence, and G6-02 is retired.
  This likewise does not close other bond/financial lanes. The shared
  worksheet-COS graph and dependent `BESSELJ` composition landed in `ed9f222`:
  COS retains the tiny guard/FPREM1/FCOS structure and uses the signed
  tangent-square reconstruction on odd quadrants (`2561/2561` selected
  evidence), while BESSELJ routes both J0/J1 cosine sites through it and stages
  only J0 `cosine*p` (`794/794` production). BUG-FUNC-046/047 and beads
  `oxf-jwh5.5/.5.1` are closed signed off; G4-06/G4-07 are retired. The full
  core and 492-job Lean gates pass; this scoped closure does not close W109.
  The PMT/IPMT/PPMT/CUM family, ATANH,
  GAMMALN/GAMMA, distributions, regression, MINVERSE, CONVERT, and the other
  catalog rows remain active model-identification work. The July-24 PMT
  "irreducible / needs provenance" framing is superseded: only a bounded
  negative result over the stated search grammar and size limits is established,
  and larger graphs plus coefficient recovery remain actionable.
- Canonical owners: [W109_ACTIVE_MODEL_DISCOVERY_CALC_GRAPH_SEARCH.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W109_ACTIVE_MODEL_DISCOVERY_CALC_GRAPH_SEARCH.md), [W108_EXCEL_NUMERIC_CORE_AND_FINANCIAL_POWER_EXACTNESS.md](C:\Work\DnaCalc\OxFunc\docs\worksets\W108_EXCEL_NUMERIC_CORE_AND_FINANCIAL_POWER_EXACTNESS.md), [OXFUNC_EXCEL_DISCREPANCY_CATALOG.md](C:\Work\DnaCalc\OxFunc\docs\OXFUNC_EXCEL_DISCREPANCY_CATALOG.md), and `.beads/` epic `oxf-jwh5`.

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: PMT/IPMT/PPMT/CUM `log1p`/`expm1` and composition graphs;
   ATANH and ACOTH shared substrate;
   GAMMALN/GAMMA, distributions, regression, MINVERSE, CONVERT, and every other
   open catalog row; fresh broad discovery; alternate CPU/application-version/
   Compatibility-Version validation; global Sections 12 and 14 closure audit.
