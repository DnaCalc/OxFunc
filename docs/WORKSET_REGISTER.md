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
   own the local parity correction for the shared power-kernel lane where Excel
   surfaces `#NUM!` for both `0^0` and `POWER(0,0)` but the current local power
   path still published `1`.
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
   `POWER(0,0)` and `0^0` both surface `#NUM!` locally, the shared Rust and
   Lean power lanes agree, focused validation is recorded, and the current-gap
   surfaces no longer overclaim `POWER`.
6. initial_epic_lanes:
   bug intake, shared power-kernel correction, Lean alignment, focused
   validation, truth-surface reconciliation
7. rollout_mode:
   `execution_target`

### W079 Lookup-Family Array Lookup-Value Lifting
1. purpose:
   own the local lookup-family correction where Excel spills array-valued
   `lookup_value` inputs for `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP` but
   the current local surface rejects or mishandles them.
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
   `XMATCH`, `MATCH`, `VLOOKUP`, and `HLOOKUP` all spill array-valued
   `lookup_value` lanes locally, focused validation is recorded, current-gap
   truth is reconciled honestly, and any unresolved adjacent `XLOOKUP`
   follow-on remains explicit rather than hidden behind stale closure language.
6. initial_epic_lanes:
   bug intake, live Excel replay, `XMATCH` / `MATCH` / `VLOOKUP` / `HLOOKUP`
   surface correction,
   focused validation, truth-surface reconciliation
7. rollout_mode:
   `execution_target`

### W080 Function Array Support Review
1. purpose:
   own the bounded seed for systematic function array-support review, starting
   with the immediate `LEFT` / `RIGHT` / `MID` spill correction and carrying
   that learning into explicit next-batch review packets for ordinary function
   array-expansion semantics, beginning with the first bounded `W066`
   text-family batch.
2. depends_on:
   `W070`, `W072`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`,
   `docs/bugs/streams/BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md`,
   `docs/bugs/streams/BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md`,
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
   current-gap reconciliation, broader review framing
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
