# OxFunc Workset Register

Status: `active_register`
Date: 2026-04-06

## 1. Purpose
This is the live ordered workset register for post-parked-baseline OxFunc.

It defines the current workset set, dependency order, and intended rollout shape
for the repo after the non-deferred baseline parking cut.

This file is not an execution-status board.
It owns workset truth, not bead state.

## 2. Planning-Surface Clarification
Planning and execution truth in OxFunc is split as follows:
1. [CHARTER.md](../CHARTER.md) owns mission, scope, and completion doctrine.
2. [OPERATIONS.md](../OPERATIONS.md) owns the operating model and execution rules.
3. this register owns the ordered workset set and dependency shape.
4. `.beads/` owns epics, beads, readiness, blockers, in-progress state, and closure.
5. current function-lane and workset evidence artifacts remain the supporting truth surfaces for supported claims.

Transition note:
1. OxFunc is now operating under the bead graph for live execution state.
2. `.beads/` owns ready, blocked, in-progress, and closed execution truth.
3. this register remains authoritative for workset order, while [W070](worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) survives only as closed migration provenance.

## 3. Use Rule
Use this document as:
1. the repo-local workset authority,
2. the source for future `workset -> epic -> bead` rollout,
3. the current ordered implementation map for post-park baseline work.

Do not use this document as:
1. a second blocker tracker,
2. a substitute for the future bead graph,
3. a reason to keep one document per work item forever,
4. a duplicate of current downstream or function-definition truth.

## 4. Register Contract
Each workset in this register carries:
1. stable workset id,
2. title,
3. purpose,
4. depends_on,
5. parent doctrine/spec surfaces,
6. primary upstream repo dependencies,
7. closure condition,
8. initial epic lanes,
9. rollout mode:
   - `execution_target`: expected to roll into epics/beads,
   - `tracking_anchor`: current authority/tracker that normally stays narrow unless reopened.

## 5. Sequencing Rule
The sequence below is the default expansion order for the live repo.

It does mean:
1. migration and doctrine reset come before new broad execution,
2. semantic-witness/export/runtime-model work comes before reopening large deferred families,
3. parked tracking authorities remain live but are not the default source of new bead rollout unless a mismatch reopens them.

## 6. Active Workset Sequence

### W070 Beads Migration And Active Tree Reduction
1. purpose:
   migrate OxFunc from workset-plus-ad-hoc execution to `workset -> epic -> bead`,
   reduce the active tree, and define the archive policy for historical packet
   surfaces.
2. depends_on: none
3. parent_doctrine_and_spec_surfaces:
   `CHARTER.md`, `OPERATIONS.md`, `docs/worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md`
4. upstream_dependencies:
   none
5. closure_condition:
   bead doctrine is live, `.beads/` exists and owns execution state, the active
   tree is materially reduced, and one real post-migration workset has executed
   under the new model.
6. initial_epic_lanes:
   doctrine rewrite, bead bootstrap, active-tree reduction, archive-wave execution
7. rollout_mode:
   `execution_target`

### W044 Library-Context Snapshot Export Baseline
1. purpose:
   keep the current V1 library-context export honest while bridging toward richer
   runtime-consumer and semantic-witness surfaces.
2. depends_on:
   `W070`
3. parent_doctrine_and_spec_surfaces:
   `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`,
   `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`,
   `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the active V1 export surfaces remain coherent and any retained V1 work is
   either absorbed into W069/W049 or explicitly narrowed.
6. initial_epic_lanes:
   export integrity, downstream contract coherence, V1-to-V2 bridge
7. rollout_mode:
   `execution_target`

### W049 Runtime Library-Context Provider Consumer Model
1. purpose:
   own the runtime provider/snapshot model that bridges the current export
   artifact into a real runtime consumer shape.
2. depends_on:
   `W070`, `W044`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`,
   `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the runtime provider/snapshot model is narrowed to the active post-park
   direction and aligned with the current witness/export plan.
6. initial_epic_lanes:
   runtime model narrowing, consumer mapping, W069 integration
7. rollout_mode:
   `execution_target`

### W069 Semantic Witness Snapshot V2 Plan
1. purpose:
   turn the parked library-context export into a semantic witness surface with
   structured help, signature, evidence, and formal-reference payloads.
2. depends_on:
   `W070`, `W044`, `W049`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md`,
   `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`
4. upstream_dependencies:
   `OxFml`, downstream consumers such as `DnaOneCalc`
5. closure_condition:
   the `V2` witness framework and seeded surfaces remain coherent as closed
   provenance for later execution work.
6. initial_epic_lanes:
   schema and stability tiers, witness-generation pipeline, seeded function-family rollout
7. rollout_mode:
   `tracking_anchor`

### W071 Semantic Witness Full-Surface Population
1. purpose:
   populate the remaining supported non-deferred surface with actual semantic
   witness rows keyed by the frozen `W069` tranche and gap ledgers.
2. depends_on:
   `W069`, `W044`, `W049`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W071_SEMANTIC_WITNESS_FULL_SURFACE_POPULATION.md`,
   `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`
4. upstream_dependencies:
   `OxFml`, downstream consumers such as `DnaOneCalc`
5. closure_condition:
   the remaining supported rows in the parked baseline have actual witness
   rows or explicit dependency-gated witness records, and the final
   reconciliation shows no leftover supported-row witness gap.
6. initial_epic_lanes:
   tranche controls, ordinary extracted rollout, ordinary curated rollout,
   seam-heavy rollout, operator/model rollout, final reconciliation
7. rollout_mode:
   `execution_target`

### W072 Bug Intake Root-Cause And Regression Stream Protocol
1. purpose:
   establish a canonical OxFunc bug-intake and regression-stream mechanism so
   defects are recorded against exact refs, grouped into canonical streams, and
   carried through root-cause and similar-risk analysis without reintroducing a
   second blocker tracker.
2. depends_on:
   `W070`
3. parent_doctrine_and_spec_surfaces:
   `OPERATIONS.md`,
   `docs/bugs/README.md`,
   `docs/worksets/W072_BUG_INTAKE_ROOT_CAUSE_AND_REGRESSION_STREAM_PROTOCOL.md`
4. upstream_dependencies:
   `OxFml` reference shape only
5. closure_condition:
   the bug protocol is live in `OPERATIONS.md`, `docs/bugs/` contains the
   canonical registers/templates/directories, duplicate and exact-ref rules are
   explicit, and the ordered workset surfaces expose the packet.
6. initial_epic_lanes:
   doctrine text, bug-register scaffolding, workset/index integration
7. rollout_mode:
   `execution_target`

### W073 Operator Value Surface And Array-Lift Expansion
1. purpose:
   own the confirmed ordinary-operator seam gap where array-involved binary
   arithmetic cannot yet travel through the current OxFunc value surface without
   downstream fallback, and reconcile the related current-gap and closure
   surfaces honestly.
2. depends_on:
   `W070`, `W072`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`,
   `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`,
   `docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md`,
   `docs/bugs/streams/BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the admitted binary operator array-lift lane is implemented or honestly
   reclassified, local validation is recorded, the downstream fallback can be
   removed or explicitly reclassified, and the current-gap plus closure surfaces
   no longer overclaim the ordinary-operator state.
6. initial_epic_lanes:
   current-surface intake, binary value-surface widening, adjacent-family review,
   downstream seam reconciliation, current-gap truth correction
7. rollout_mode:
   `execution_target`

### W074 Ordinary Operator Broadcast Reconciliation
1. purpose:
   reconcile the broader ordinary-operator broadcast rule now proven by local
   Excel comparison, widen arithmetic plus compare/concat onto that rule, and
   refresh the downstream seam truth needed to remove temporary OxFml
   compatibility fallbacks honestly.
2. depends_on:
   `W070`, `W072`, `W073`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md`,
   `docs/function-lane/W45_EXECUTION_RECORD.md`,
   `docs/bugs/streams/BUG-FUNC-002_ordinary_operator_broadcast_semantics_gap.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the current ordinary arithmetic and compare/concat surfaces either follow the
   observed broadcast rule or are honestly reclassified, refreshed native probe
   evidence is recorded, the required downstream handoff is filed if the seam
   changed materially, and the active truth surfaces no longer overclaim the
   ordinary-operator state.
6. initial_epic_lanes:
   empirical broadcast characterization, runtime widening, probe refresh and
   validation, downstream handoff, truth-surface reconciliation
7. rollout_mode:
   `execution_target`

### W075 Multi-Area Reference Seam Correction
1. purpose:
   reconcile the OxFunc-side same-sheet multi-area reference seam so union
   formation, `AREAS`, and `INDEX(..., area_num)` use first-class
   `ReferenceKind::MultiArea` rather than an `Area` plus parenthesized-string
   convention.
2. depends_on:
   `W070`, `W072`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md`,
   `docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`,
   `docs/function-lane/FUNCTION_SLICE_INDEX_CONTRACT_PRELIM.md`,
   `docs/function-lane/FUNCTION_SLICE_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_CONTRACT_PRELIM.md`,
   `docs/bugs/streams/BUG-FUNC-003_multi_area_reference_seam_collapses_to_area_string.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   `OP_UNION_REF` returns first-class `MultiArea`, named consumers consume that
   shape and reject the old non-`MultiArea` wrapper carrier, local validation is
   recorded, the downstream handoff is filed, and the active truth surfaces no
   longer overclaim the old area-string seam.
6. initial_epic_lanes:
   inbound note intake, runtime correction, consumer alignment, truth-surface
   reconciliation, downstream handoff
7. rollout_mode:
   `execution_target`

### W076 Multi-Area Value Materialization Style A
1. purpose:
   move same-sheet `ReferenceKind::MultiArea` value materialization into
   OxFunc-owned resolver-driven combination semantics for current value-required
   lanes, matching `HANDOFF-OXFUNC-002`.
2. depends_on:
   `W070`, `W075`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W076_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md`,
   `docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`,
   `docs/handoffs/HO-FN-007_multiarea_value_materialization_style_a.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   same-sheet `MultiArea` materializes through OxFunc-owned resolver-driven
   combination semantics for the admitted value-required callers, mixed-sheet
   multi-area remains an explicit rejection path, focused local validation is
   recorded, and the downstream reply handoff is filed.
6. initial_epic_lanes:
   handoff intake, materialization helper, caller alignment, focused validation,
   downstream reply handoff
7. rollout_mode:
   `execution_target`

### W077 Corpus IF Condition And Numeric Comparison Tolerance
1. purpose:
   own the current corpus follow-on around `IF` empty-text condition triage and
   the broader numeric comparison tolerance family split across ordinary
   operators, criteria/database selection, and `SWITCH`.
2. depends_on:
   `W070`, `W072`, `W045`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W077_CORPUS_IF_CONDITION_AND_NUMERIC_COMPARISON_TOLERANCE.md`,
   `docs/bugs/streams/BUG-FUNC-004_numeric_comparison_tolerance_family_split.md`,
   `docs/function-lane/FLOATING_POINT_EXECUTION_RECORD.md`,
   `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`,
   `docs/function-lane/FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md`,
   `docs/function-lane/FUNCTION_SLICE_DATABASE_FAMILY_CONTRACT_PRELIM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the inbound `IF` report is either fixed or honestly closed as no local bug,
   the tolerant numeric comparison family split is pinned with replayable Excel
   evidence, the local tolerant families are corrected without changing the
   exact-match contrast families, focused validation is recorded, and the reply
   handoff is filed.
6. initial_epic_lanes:
   handoff intake, empirical replay, tolerant-family runtime correction, truth
   surface reconciliation, downstream reply
7. rollout_mode:
   `execution_target`

### W078 Power Zero-To-Zero Num Error Parity
1. purpose:
   own the local review/correction lane for the prior shared power-kernel claim
   where an April 8, 2026 local Excel replay reported `#NUM!` for both `0^0`
   and `POWER(0,0)`, but the then-current local power path still published `1`.
   Fresh Excel replay on 2026-04-29 confirmed the Excel rule, and the local
   Rust/Lean correction is landed, so this workset now records closed
   historical bug work rather than a current open gap.
2. depends_on:
   `W070`, `W072`, `W045`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W078_POWER_ZERO_TO_ZERO_NUM_ERROR_PARITY.md`,
   `docs/bugs/streams/BUG-FUNC-005_power_zero_to_zero_diverges_from_excel.md`,
   `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`,
   `docs/function-lane/W53_NUMERIC_FORENSICS_20260326.md`,
   `formal/lean/OxFunc/Functions/PowerFn.lean`
4. upstream_dependencies:
   none
5. closure_condition:
   fresh Excel replay either confirms the prior `POWER(0,0)` / `0^0` claim or
   invalidates it as stale; the shared Rust and Lean power lanes agree with the
   freshly confirmed rule; focused validation is recorded where the bug remains
   real; and the current-gap surfaces no longer overclaim `POWER`.
6. current_status:
   closed on 2026-04-29 after fresh Excel COM replay confirmed `#NUM!` for both
   `=0^0` and `=POWER(0,0)` on Excel 16.0 build 19929, with the local Rust and
   Lean correction landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`.
7. initial_epic_lanes:
   bug intake, shared power-kernel correction, Lean alignment, focused
   validation, truth-surface reconciliation
8. rollout_mode:
   `execution_target`

### W079 Lookup-Family Array Lookup-Value Lifting
1. purpose:
   own the local lookup-family correction where Excel spills array-valued
   `lookup_value` inputs for `XMATCH`, `MATCH`, `VLOOKUP`, `HLOOKUP`, and
   adjacent `XLOOKUP` but the current local surface rejects or mishandles them.
2. depends_on:
   `W070`, `W072`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md`,
   `docs/bugs/streams/BUG-FUNC-006_lookup_selection_array_lookup_value_lifting_gap.md`,
   `docs/function-lane/XMATCH_EXECUTION_RECORD.md`,
   `docs/function-lane/W10_EXECUTION_RECORD.md`
4. upstream_dependencies:
   none
5. closure_condition:
   `XMATCH`, `MATCH`, `VLOOKUP`, `HLOOKUP`, and adjacent `XLOOKUP` all spill
   array-valued `lookup_value` lanes locally, focused validation is recorded,
   current-gap truth is reconciled honestly, and landed-ref promotion state is
   recorded explicitly.
6. initial_epic_lanes:
   bug intake, live Excel replay, `XMATCH` / `MATCH` / `VLOOKUP` / `HLOOKUP`
   / `XLOOKUP` surface correction,
   focused validation, truth-surface reconciliation
7. rollout_mode:
   `execution_target`

### W080 Function Array Support Review
1. purpose:
   own the bounded seed for systematic function array-support review: the
   immediate `LEFT` / `RIGHT` / `MID` spill correction plus two adjacent
   text-family follow-on batches, with successor `W090` owning the broader
   supported-surface sweep.
2. depends_on:
   `W070`, `W072`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`,
   `docs/bugs/streams/BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md`,
   `docs/bugs/streams/BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md`,
   `docs/bugs/streams/BUG-FUNC-016_text_search_replace_array_support_gap.md`,
   `docs/worksets/W090_FUNCTION_ARRAY_SUPPORT_SYSTEMATIC_SWEEP.md`,
   `docs/function-lane/FUNCTION_SLICE_TEXT_CORE_AND_COMPATIBILITY_FAMILY_CONTRACT_PRELIM.md`,
   `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
4. upstream_dependencies:
   none
5. closure_condition:
   the immediate text-slice seed correction is validated locally, current-gap
   truth is reconciled honestly, at least one bounded post-text-slice batch is
   replayed and reconciled honestly, and the broader function-array-support
   review has an explicit bounded owner plus next-batch sequencing without
   claiming the full supported surface has already been reviewed.
6. initial_epic_lanes:
   text-slice bug intake, immediate seed correction, focused validation,
   current-gap reconciliation, successor sweep framing
7. rollout_mode:
   `execution_target`

### W081 RATE Default-Guess Convergence Repair
1. purpose:
   own the bounded local `RATE` repair where Excel returns a small positive
   rate for a mortgage-style omitted-guess row but the current local default-
   guess path fails with `#NUM!`.
2. depends_on:
   `W070`, `W072`, `W024`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md`,
   `docs/bugs/streams/BUG-FUNC-009_rate_default_guess_solver_no_convergence.md`,
   `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`,
   `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`
4. upstream_dependencies:
   none
5. closure_condition:
   the reopened mortgage-style omitted-guess `RATE` lane matches Excel locally,
   the earlier admitted seed inversion row remains aligned, focused validation
   is recorded, and the current-gap surfaces no longer overclaim `RATE`.
6. initial_epic_lanes:
   bug intake, omitted-guess replay characterization, local solver/default-
   guess repair, focused validation, current-gap reconciliation
7. rollout_mode:
   `execution_target`

### W082 Locale/Format Seam Ownership Realignment
1. purpose:
   move OxFunc to the exact intended locale/format decomposition where OxFunc
   keeps function semantics plus the typed seam contract, while OxFml/FEC owns
   and supplies the concrete parser/formatter implementation without any
   backward-compatible OxFunc runtime fallback.
2. depends_on:
   `W070`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W082_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT.md`,
   `docs/function-lane/LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md`,
   `docs/function-lane/LOCALE_AND_FORMAT_INTERFACE_OPTIONS.md`,
   `docs/handoffs/HO-FN-009_locale_format_seam_ownership_realignment.md`,
   `../OxFml/docs/handoffs/HO-FN-009_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT_ACK.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   OxFunc no longer ships an OxFunc-owned production parser/formatter runtime
   path for locale-sensitive function evaluation, the affected functions
   consume caller-supplied `LocaleFormatContext` only, no backward-compatible
   runtime fallback remains, and the downstream handoff is filed and
   acknowledged.
6. initial_epic_lanes:
   seam inventory, OxFunc runtime ownership removal, caller alignment,
   downstream handoff, truth-surface reconciliation
7. rollout_mode:
   `execution_target`

### W083 Dynamic-Array Sort Omitted Optional-Argument Defaulting
1. purpose:
   own the bounded local repair where `SORT` and adjacent `SORTBY` fail to
   default optional controls when the argument is syntactically omitted and
   therefore arrives on the prepared surface as `MissingArg`.
2. depends_on:
   `W070`, `W072`, `W039`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W083_DYNAMIC_ARRAY_SORT_OMITTED_OPTIONAL_ARGUMENT_DEFAULTING.md`,
   `docs/bugs/streams/BUG-FUNC-010_dynamic_array_sort_family_omitted_optional_argument_defaulting_gap.md`,
   `docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`,
   `docs/function-lane/W39_EXECUTION_RECORD.md`
4. upstream_dependencies:
   none
5. closure_condition:
   `SORT({2;3;7;5},,-1)` and adjacent `SORTBY(..., by_array,)` omission lanes
   default correctly locally, focused validation is recorded, and the
   current-gap surfaces no longer overclaim `SORT` / `SORTBY`.
6. initial_epic_lanes:
   bug intake, omission/default repair, focused validation, W39/W51 truth
   reconciliation, adjacent optional-default review framing
7. rollout_mode:
   `execution_target`

### W084 COUNTBLANK Range-Only Parity
1. purpose:
   own the bounded local repair where `COUNTBLANK` currently over-admits
   array-valued substitutes even though live Excel accepts true ranges and
   rejects array-valued substitutes with `#VALUE!`.
2. depends_on:
   `W070`, `W072`, `W016`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W084_COUNTBLANK_RANGE_ONLY_PARITY.md`,
   `docs/bugs/streams/BUG-FUNC-011_countblank_range_only_parity_gap.md`,
   `docs/function-lane/W16_EXECUTION_RECORD.md`,
   `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
4. upstream_dependencies:
   none
5. closure_condition:
   direct array-valued `COUNTBLANK` inputs reject locally with `#VALUE!`, true
   ranges still count empty cells and `""`, focused validation is recorded, and
   the current-gap surfaces no longer overclaim `COUNTBLANK`.
6. initial_epic_lanes:
   bug intake, countblank range-only repair, focused validation, W51 truth
   reconciliation, bounded adjacent policy review
7. rollout_mode:
   `execution_target`

### W085 TAKE/DROP Omitted Leading-Count Parity
1. purpose:
   own the bounded local repair where `TAKE` and `DROP` currently fail when the
   leading row-count argument is syntactically omitted and the third argument
   is used to slice columns.
2. depends_on:
   `W070`, `W072`, `W039`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W085_TAKE_DROP_OMITTED_LEADING_COUNT_PARITY.md`,
   `docs/bugs/streams/BUG-FUNC-012_take_drop_omitted_leading_count_parity_gap.md`,
   `docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md`,
   `docs/function-lane/W39_EXECUTION_RECORD.md`
4. upstream_dependencies:
   none
5. closure_condition:
   omitted-leading-count `TAKE` / `DROP` lanes default correctly locally,
   focused validation is recorded, and the current-gap surfaces no longer
   overclaim `TAKE` / `DROP`.
6. initial_epic_lanes:
   handoff intake, omission/default repair, focused validation, W39/W51 truth
   reconciliation, bounded adjacent reshape review
7. rollout_mode:
   `execution_target`

### W086 Normal-Distribution Exact-Value Accuracy
1. purpose:
   own the bounded local reconciliation where current `NORM.DIST` and
   `NORM.INV` approximations drift from live Excel `Value2` on bounded
   current-baseline exact-value witnesses.
2. depends_on:
   `W070`, `W072`, `W062`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W086_NORMAL_DISTRIBUTION_EXACT_VALUE_ACCURACY.md`,
   `docs/bugs/streams/BUG-FUNC-013_normal_distribution_exact_value_accuracy_gap.md`,
   `docs/function-lane/FUNCTION_SLICE_STATISTICAL_DISTRIBUTIONS_AND_COMPAT_B_CONTRACT_PRELIM.md`,
   `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
4. upstream_dependencies:
   none
5. closure_condition:
   the bounded `NORM.DIST` and `NORM.INV` exact-value witnesses match Excel
   locally, focused validation is recorded, and the current-gap surfaces no
   longer overclaim the reopened rows.
6. initial_epic_lanes:
   bug intake, exact-value replay characterization, local approximation
   reconciliation, focused validation, W51 truth reconciliation
7. rollout_mode:
   `execution_target`

### W087 XIRR Solver Precision Reconciliation
1. purpose:
   own the bounded local reconciliation where current `XIRR` solver output
   drifts from live Excel `Value2` on a pinned current-baseline cashflow/date
   witness.
2. depends_on:
   `W070`, `W072`, `W037`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W087_XIRR_SOLVER_PRECISION_RECONCILIATION.md`,
   `docs/bugs/streams/BUG-FUNC-014_xirr_solver_precision_drift.md`,
   `docs/function-lane/FUNCTION_SLICE_CASHFLOW_RATE_FAMILY_CONTRACT_PRELIM.md`,
   `docs/function-lane/W37_EXECUTION_RECORD.md`
4. upstream_dependencies:
   none
5. closure_condition:
   the bounded `XIRR` precision witness matches Excel locally, focused
   validation is recorded, and the current-gap surfaces no longer overclaim the
   reopened row.
6. initial_epic_lanes:
   bug intake, solver-precision replay characterization, local iterative-path
   reconciliation, focused validation, W51 truth reconciliation
7. rollout_mode:
   `execution_target`

### W088 Smart-Fuzzer Differential Exploration Pilot
1. purpose:
   establish the first smart-fuzzer execution lane for high-throughput,
   metadata-aware differential exploration against Excel while keeping ordinary
   passing cases as compact telemetry rather than heavyweight prose artifacts.
2. depends_on:
   `W070`, `W072`, `W044`, `W049`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`,
   `smart-fuzzer/README.md`,
   `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md`,
   `smart-fuzzer/planning/CASE_SCHEMA_V0.md`,
   `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`,
   `docs/bugs/README.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the pilot throughput benchmark, compact telemetry/failure-packet formats,
   static metadata/risk index, and at least one bounded local-vs-Excel
   comparison family are exercised with typed comparison output; any confirmed
   mismatch is routed through the ordinary bug stream or classified as a
   seam/harness blocker, and pass rows remain summarized as coverage telemetry
   rather than completion evidence.
6. initial_epic_lanes:
   artifact economy and schema discipline, Excel throughput benchmark, static
   metadata/risk index, bounded pilot generator, local evaluator and Excel
   comparator, mismatch minimization and promotion path
7. rollout_mode:
   `execution_target`

### W089 Smart-Fuzzer Sweeping Invocation-Space Exploration
1. purpose:
   plan the next broad smart-fuzzer sweep over the OxFunc invocation space by
   inventorying tweakable dimensions, coverage telemetry, sampling levers,
   known-deviation handling, and execution gates before running any tests.
2. depends_on:
   `W088`, `W070`, `W072`, `W044`, `W049`, `W051`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`,
   `smart-fuzzer/planning/SWEEPING_INVOCATION_SPACE_RUN_PLAN.md`,
   `smart-fuzzer/README.md`,
   `smart-fuzzer/planning/RUN_ARTIFACT_CONTRACT.md`,
   `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`,
   `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`,
   `docs/bugs/README.md`
4. upstream_dependencies:
   `OxFml`, live Excel comparison harness availability
5. closure_condition:
   the sweep has a generated dimension inventory, compact coverage taxonomy,
   metadata-driven generator matrix, local-evaluation budget, Excel candidate
   budget, blocked/deferred seam classification, and explicit user-approved
   execution gate; any later unexpected mismatch is minimized or routed through
   the ordinary bug stream, while passing rows remain coverage telemetry rather
   than completion evidence.
6. initial_epic_lanes:
   dimension inventory and coverage taxonomy, generator matrix and typed
   mutator plan, local evaluator expansion and dry-run budget, Excel candidate
   selection and batching budget, blocked/deferred seam classification, roadmap
   trace and compact reporting artifacts, unexpected mismatch triage and
   minimization protocol, explicit execution gate
7. rollout_mode:
   `planning_target`

### W090 Function Array Support Systematic Sweep
1. purpose:
   own the broader family-by-family sweep for array-valued scalar-parameter
   behavior across supported OxFunc functions, using compact inventory,
   coverage telemetry, bounded Excel comparison batches, and ordinary bug
   promotion for confirmed divergences.
2. depends_on:
   `W080`, `W088`, `W089`, `W070`, `W072`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W090_FUNCTION_ARRAY_SUPPORT_SYSTEMATIC_SWEEP.md`,
   `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`,
   `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`,
   `docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`,
   `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`,
   `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`,
   `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
4. upstream_dependencies:
   live Excel comparison harness availability
5. closure_condition:
   the supported-function and argument-role inventory is stable, at least one
   non-text successor tranche is executed or explicitly replaced with rationale,
   confirmed divergences are minimized and promoted through the ordinary bug
   stream, passing cases remain compact coverage telemetry, truth surfaces are
   reconciled for examined regions, and remaining unswept regions have an
   explicit next-owner or next-tranche plan without claiming the full supported
   surface has already been reviewed.
6. initial_epic_lanes:
   function/argument inventory, static risk classification, replay matrix and
   batch sizing, first post-`W080` tranche selection, local-vs-Excel comparison,
   mismatch minimization, bug promotion, coverage-roadmap trace
7. rollout_mode:
   `execution_target`

### W041 External Data Provider And Cube Functions
1. purpose:
   remain the current deferred/open authority for provider-bound and cube-context
   functions that were intentionally excluded from the parked current-version set.
2. depends_on:
   `W070`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md`,
   `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   either the deferred provider/cube surface is later re-admitted honestly or it
   remains clearly bounded as deferred without stale local ambiguity.
6. initial_epic_lanes:
   deferred-surface narrowing, provider capability modeling, evidence guardrails
7. rollout_mode:
   `execution_target`

### W043 RTD COM Activation And Topic Lifecycle Seam
1. purpose:
   remain the live authority for the RTD seam and the minimal COM/topic-lifecycle
   packet around RTD behavior.
2. depends_on:
   `W070`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md`,
   `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the RTD seam is either explicitly narrowed and retained as deferred/open truth
   or rolled into a future honest workset-led execution lane with active evidence.
6. initial_epic_lanes:
   seam authority cleanup, RTD evidence preservation, future-rollout preparation
7. rollout_mode:
   `execution_target`

### W050 Deferred Current-Version Surface
1. purpose:
   remain the canonical deferred-current-version tracker.
2. depends_on:
   none
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md`,
   `docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   none under the current parked baseline; this workset remains the authority
   until the deferred surface is intentionally reopened.
6. initial_epic_lanes:
   none unless reopened
7. rollout_mode:
   `tracking_anchor`

### W051 In-Scope Current-Version Not-Complete Surface
1. purpose:
   remain the canonical non-deferred current-version tracker for any reopened or
   still-open current-version rows.
2. depends_on:
   none
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
4. upstream_dependencies:
   none
5. closure_condition:
   remain narrow and empty unless a concrete non-deferred gap is reopened.
6. initial_epic_lanes:
   none unless reopened
7. rollout_mode:
   `tracking_anchor`

### W054 Lean Formalization Gap Reconciliation
1. purpose:
   remain the parked authority for non-deferred Rust-vs-Lean id reconciliation.
2. depends_on:
   none
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`,
   `docs/function-lane/W54_LEAN_FORMALIZATION_GAP_INVENTORY.csv`
4. upstream_dependencies:
   none
5. closure_condition:
   remain parked and coherent unless future proof-deepening or reopen work
   creates a new reconciliation obligation.
6. initial_epic_lanes:
   none unless reopened
7. rollout_mode:
   `tracking_anchor`
