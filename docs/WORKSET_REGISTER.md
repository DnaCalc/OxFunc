# OxFunc Workset Register

Status: `active_register`
Date: 2026-04-02

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
1. OxFunc is currently in `W070` Phase E-prep state.
2. `.beads/` is now bootstrapped and owns live execution-state truth.
3. this register remains authoritative for workset order and [W070](worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md) remains the migration authority while the active-tree reduction waves continue.

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
   the first real V2 witness surface exists with explicit schema, stable
   generation path, and at least one exercised function-family slice.
6. initial_epic_lanes:
   schema and stability tiers, witness-generation pipeline, seeded function-family rollout
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
   or rolled into a future honest execution packet with active evidence.
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
   remain the canonical non-deferred current-version tracker, currently parked at
   zero open rows.
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
