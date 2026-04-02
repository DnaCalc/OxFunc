# W070 OxFunc Beads Migration And Active Tree Reduction

Status: `in_progress`
Date: 2026-04-01
Tag anchor: `OxFunc_V1`
Parked baseline commit: `ac9d6be`

## 1. Purpose
This workset defines the migration from OxFunc's current workset-plus-ad-hoc
execution model to a structured `workset -> epic -> bead` execution model,
while also reducing the active documentation surface to a small, current,
truth-bearing set.

This packet is the authoritative migration plan for:
1. execution-doctrine reorientation,
2. active-tree cleanup and archive policy,
3. document triage and supersession,
4. bead bootstrap and rollout,
5. the first post-migration execution lane.

All later cleanup, archive, or doctrine-rewrite rounds should anchor back to
this plan and report progress against its phases rather than introducing new
ad hoc migration notes.

## 2. Why Now
OxFunc has reached a parked non-deferred baseline:
1. current non-deferred backlog is drained,
2. current-phase seam freeze is parked,
3. current downstream snapshot is published,
4. the remaining excluded surface is the intentional `W050` deferred set,
5. the next major lane (`W069`) is a new product-direction and not a backlog
   rescue packet.

This is the right point to:
1. preserve the current shape behind a stable tag,
2. reduce active-file clutter,
3. move execution-state truth into a bead graph before the next wave begins.

## 3. Migration Goal
OxFunc should end this migration with:
1. current doctrine and scope truth in a small live document set,
2. one explicit workset register that owns ordered workset truth,
3. `.beads/` as the sole owner of execution-state truth,
4. active work executed through `workset -> epic -> bead`,
5. historical execution artifacts preserved by git history and tag rather than
   remaining in the active tree by default,
6. existing evidence, replay, formal, and downstream-contract artifacts kept
   live only where they still support current claims.

## 4. Non-Goals
This migration does not:
1. erase provenance from git history,
2. weaken OxFunc's no-compromise function doctrine,
3. remove reproducible evidence required for current supported claims,
4. rewrite historical worksets only for style,
5. collapse function-lane evidence discipline into a generic tracker.

## 5. Governing Principles
### 5.1 Active Tree Rule
The active tree should describe the present.
Git history and tagged baselines should preserve the past.

### 5.2 Truth-Surface Rule
After migration, truth should be partitioned like this:
1. scope and doctrine truth:
   - `README.md`
   - `CHARTER.md`
   - `OPERATIONS.md`
   - current foundation/index/contract docs
2. workset-sequencing truth:
   - `docs/WORKSET_REGISTER.md`
3. execution-state truth:
   - `.beads/`
4. evidence/provenance truth:
   - current function-lane/evidence docs still needed for live claims

### 5.3 Bead Mutation Rule
Once bead execution is adopted:
1. do not edit `.beads/` directly,
2. use `br` for mutations,
3. use robot-style `br`/`bv` inspection only from agent sessions.

### 5.4 Historical-Surface Rule
Historical execution docs should not remain in `main` merely because they were
once useful.

Default posture:
1. preserve in git and tag,
2. remove from active file set,
3. leave only a compact history pointer where needed.

## 6. Document Classification Policy
Every current document touched by this migration should be classified into one
of four buckets.

### 6.1 `active_canonical`
Keep in the active tree as current truth.

Examples:
1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `docs/FOUNDATION_SPEC_INDEX.md`
5. current downstream metadata/help contracts
6. current surface admission policy
7. current snapshot export and readme
8. current formalization strategy
9. `docs/WORKSET_REGISTER.md`
10. `docs/BEADS.md`
11. current parked-baseline note

### 6.2 `active_evidence`
Keep live only if still required to support current supported claims.

Examples:
1. current function-lane evidence registry,
2. current live export inputs and outputs,
3. current formal gap/reconciliation ledgers if still normative,
4. still-live seam freeze and downstream-consumer model docs.

### 6.3 `historical_provenance`
Preserve in git/tag, but remove from the active tree unless explicitly still
needed.

Examples:
1. closed workset docs that are no longer current planning surfaces,
2. closed execution records,
3. old blocker ledgers,
4. old handoff packets with no active effect,
5. old family-specific backlog lists after central truth has moved elsewhere.

### 6.4 `disposable_transitional`
Delete from the active tree once migration has consumed their content.

Examples:
1. stale temporary reconciliation CSVs whose truth is fully re-expressed in a
   current register,
2. superseded interim status notes,
3. temporary classification lists that no longer feed a live artifact.

## 7. Archive Policy
Archive does not mean "move everything into `docs/archive/` on `main`".

Default archive policy:
1. preserve the pre-migration state with tag `OxFunc_V1`,
2. use git history as the primary historical store,
3. keep `main` lean,
4. leave at most one compact live history pointer document if needed.

Preferred approach:
1. tag the parked baseline,
2. complete migration commits on `main`,
3. remove superseded historical docs from the active tree,
4. retain only a compact `docs/HISTORY.md`-style pointer if discoverability is
   needed.

## 8. Current OxFunc Surfaces To Reorient
The following current surfaces need doctrinal or structural change.

### 8.1 `docs/worksets/README.md`
Current role:
1. workset index,
2. live execution summary,
3. current status/provenance blend.

Target role:
1. minimal index only, or retire in favor of `docs/WORKSET_REGISTER.md`,
2. no live execution truth,
3. no backlog counts or state narratives.

### 8.2 `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
Current role:
1. feature register,
2. partial status tracker,
3. current-state narrative.

Target role:
1. high-level feature map only,
2. no execution-state truth,
3. no duplicate backlog tracking where the bead graph or workset register owns
   it.

### 8.3 `CURRENT_BLOCKERS.md`
Current role:
1. structured blocker ledger,
2. partial execution-state surface.

Target role:
1. remove from the active tree,
2. move ordinary blockers into `.beads/`,
3. preserve any historical blocker narrative only through `docs/HISTORY.md`,
   git history, and tag `OxFunc_V1`.

### 8.4 Workset Docs
Current role:
1. scope packet,
2. execution packet,
3. partial status board,
4. historical provenance.

Target role:
1. either current scope packet,
2. or current evidence packet,
3. or removed from active tree as historical provenance.

### 8.5 Function-Lane Sub-Lists And Reconciliation Tables
Current role:
1. mixed:
   - active contract/evidence inputs,
   - historical backlog helpers,
   - old packet-local rollout aids.

Target role:
1. keep current contract/evidence inputs,
2. remove stale execution helper lists from active tree,
3. avoid keeping packet-local tracking CSVs live once their truth has moved.

## 9. OxFunc Bead Model
OxFunc should adopt a bead model tailored to its assurance stack rather than a
copy of DnaOneCalc.

### 9.1 Worksets
Worksets become high-level planning units only.

Each workset should carry:
1. id,
2. title,
3. purpose,
4. depends_on,
5. parent doctrine/spec sections,
6. closure condition,
7. initial epic lanes.

### 9.2 Epics
Epics become the main execution lanes under a workset.

Typical OxFunc epic lanes:
1. runtime/kernel implementation lane,
2. native replay/evidence lane,
3. Lean/formal lane,
4. export/catalog/admission lane,
5. seam/handoff lane when boundary pressure exists,
6. cleanup or archive lane when migration is the work itself.

### 9.3 Beads
Beads are the unit of executable progress.

Every OxFunc bead should state:
1. one reviewable outcome,
2. the evidence needed for closure,
3. the parent epic,
4. any real dependency,
5. any truth surfaces touched.

Capability-bearing OxFunc beads should normally close on:
1. code,
2. tests or replay artifacts,
3. formal/evidence alignment where in scope,
4. updated current truth surfaces.

### 9.4 What Does Not Move Into Beads
Beads do not replace:
1. function-lane evidence artifacts,
2. execution records needed to justify function-phase-complete claims,
3. current downstream contracts,
4. current formalization doctrine.

Beads own execution state, not the whole assurance stack.

## 10. Migration Phases
### Phase A: Anchor The Historical Baseline
Deliverables:
1. create and publish tag `OxFunc_V1`,
2. keep a compact parked-baseline note pointing to the current supported surface,
3. record the baseline commit in this workset.

Exit condition:
1. pre-migration OxFunc state is recoverable by tag without narrative memory.

Current state:
1. complete, with tag `OxFunc_V1` anchored at parked-baseline commit `ac9d6be`.

### Phase B: Inventory And Triage
Deliverables:
1. classify current docs as `active_canonical`, `active_evidence`,
   `historical_provenance`, or `disposable_transitional`,
2. produce an explicit migration triage register,
3. identify which current docs remain active inputs for current support claims.

Exit condition:
1. every live doc class has an owner decision,
2. there is no ambiguous "leave it for now" pile.

Current state:
1. complete; the migration triage pass has been executed and now survives only as closed provenance through this packet, [docs/HISTORY.md](/C:/Work/DnaCalc/OxFunc/docs/HISTORY.md), git history, and tag `OxFunc_V1`.
2. current inventory baseline:
   - `624` docs under `docs/`
   - `72` workset docs under `docs/worksets/`
   - `533` docs under `docs/function-lane/`
3. currently quantified function-lane families include:
   - `98` contract-prelim docs
   - `75` execution records
   - `80` scenario manifests
   - `56` runtime-requirement notes
   - `21` scope-reconciliation ledgers
4. active doctrine no longer treats `TUX1000_PLAN.md` as a live authority; that plan has been removed from the active tree and reclassified as `historical_provenance`.
5. current Phase B split now distinguishes:
   - live later workset anchors that remain active (`W041`, `W043`, `W044`, `W049`, `W050`, `W051`, `W054`, `W069`, `W070`)
   - closed later packets queued for archive waves
   - research/investigation notes that still anchor current empirical or seam understanding (`FLOATING_POINT_BEHAVIOR_RESEARCH_NOTES.md`, `STRING_BEHAVIOR_RESEARCH_NOTES.md`, `IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md`)
   - function contract prelim docs as a conservative keep-for-now family pending a narrower subwave review
6. packet-local evidence tails are now split into:
   - `7` active packet-local artifacts attached to live worksets (`W041`, `W043`, `W044`, `W049`, `W054`)
   - roughly `225` historical packet-local artifacts (`EXECUTION_RECORD`, `SCENARIO_MANIFEST`, `RUNTIME_REQUIREMENTS`, `SCOPE_RECONCILIATION`) queued for archive waves unless a later review promotes them into the active subset
7. the remaining uncategorized function-lane tail is now grouped into:
   - semantic-core support surfaces to keep for now,
   - XLL/replay-support surfaces to keep for now,
   - live deferred/runtime/export inventories tied to `W041` / `W043` / `W044` / `W049` / `W050` / `W051`,
   - large historical note/inventory families (`W16_BATCH*`, old localization and deferred inventories, closed seam-review notes, packet registers and normalization maps) queued for archive waves
8. the migration triage pass gave every major current doc family an explicit bucket and target action; there is no remaining large "unclassified" document pile blocking doctrine rewrite.

### Phase C: Doctrinal Reorientation
Deliverables:
1. new `docs/WORKSET_REGISTER.md`,
2. new `docs/BEADS.md`,
3. rewritten `OPERATIONS.md`,
4. rewritten `AGENTS.md`,
5. rewritten `README.md` references where needed.

Required doctrinal changes:
1. worksets are planning units, not execution-state objects,
2. execution runs through `workset -> epic -> bead`,
3. `.beads/` owns execution state,
4. `br` owns mutation,
5. live blockers move into the bead graph by default.

Exit condition:
1. local doctrine consistently describes the bead model,
2. no active doctrine file still presents worksets as the live execution-state
   surface.

Current state:
1. complete:
   - `docs/WORKSET_REGISTER.md` created,
   - `docs/BEADS.md` created,
   - `OPERATIONS.md`, `AGENTS.md`, and `README.md` rewritten around the new model,
   - `docs/worksets/README.md` now explicitly defers workset truth to `docs/WORKSET_REGISTER.md`.

### Phase D: Bead Workspace Bootstrap
Deliverables:
1. initialize `.beads/`,
2. install or validate `br` workflow expectations,
3. add a minimal OxFunc `check-worksets` validator,
4. optionally add a seed script that rolls worksets into initial epics.

Bootstrap rule:
1. seed only currently live worksets,
2. do not import every historical packet as an open bead graph by default.

Exit condition:
1. OxFunc has a live bead workspace,
2. current open execution can be inspected via `br`/`bv`.

Current state:
1. complete:
   - `.beads/` is now bootstrapped under the OxFunc root,
   - `scripts/check-worksets.ps1` now validates the active workset register and live bead workspace shape,
   - `scripts/seed-beads-from-worksets.ps1` now seeds the first live OxFunc graph from the active execution-target worksets,
   - the current bead graph now contains `16` seeded issues, with the initial ready path on the first Phase E `W070` archive-wave bead and blocked follow-on rollout for `W069` / `W044` / `W049`.

### Phase E: Active-Tree Reduction
Deliverables:
1. remove superseded historical execution docs from `main`,
2. demote workset and function-lane indexes to active-only surfaces,
3. shrink `docs/worksets/README.md` or retire it,
4. remove `CURRENT_BLOCKERS.md`,
5. remove stale packet-local helper lists not needed for current truth.

Default removal set:
1. closed historical worksets no longer needed as current truth,
2. old execution records not required for current support claims,
3. stale backlog and reconciliation trackers,
4. superseded seam or promotion notes.

Exit condition:
1. active tree is materially smaller,
2. active docs describe present truth rather than historical path.

Current execution encoding:
1. active-tree reduction epic:
   - `oxf-7xi`
2. archive-wave rollout and execution beads:
   - `oxf-7xi.1` rollout the first explicit archive/removal wave,
   - `oxf-7xi.2` execute wave 1 on early historical worksets and redirect stubs,
   - `oxf-7xi.3` execute wave 2 on closed seam packets, handoffs, and historical helper inventories,
   - `oxf-7xi.4` execute wave 3 on late closed packets and bridge-contract residue,
   - `oxf-7xi.5` execute wave 4 on packet-local evidence tails and residual historical families
3. tracker/index cleanup epic:
   - `oxf-o62`
4. tracker/index cleanup beads:
   - `oxf-o62.1` bridge-posture narrowing after bead bootstrap,
   - `oxf-o62.2` remove `CURRENT_BLOCKERS.md` from active operation,
   - `oxf-o62.3` shrink workset and function-lane indexes to active-surface maps only,
   - `oxf-o62.4` reduce the feature register to a high-level map after archive-wave execution

Phase note:
1. live progress for Phase E is now owned by `.beads/`; this packet names the intended wave structure but is no longer the execution tracker.

### Phase F: Reconcile Current Truth Surfaces
Deliverables:
1. ensure current downstream-facing docs still agree after cleanup,
2. ensure current support/evidence claims remain reproducible,
3. ensure surviving function-lane artifacts still form a coherent live set.

Exit condition:
1. no current supported claim depends on a removed active-tree file whose truth
   was not re-homed.

Current execution encoding:
1. truth-surface reconciliation epic:
   - `oxf-pyf`
2. reconciliation beads:
   - `oxf-pyf.1` reconcile surviving doctrine, export, and parked-baseline surfaces after archive reduction,
   - `oxf-pyf.2` prove that surviving support claims no longer depend on removed active-tree files

Phase note:
1. live progress for Phase F is now owned by `.beads/`; this packet names the intended proof and reconciliation surfaces but is no longer the execution tracker.

Current proof reading after archive reduction:
1. the active parked-baseline and downstream guidance surfaces now agree on the same current parked reading:
   - `534` published rows,
   - `517` supported rows,
   - `17` deferred rows,
   - `0` non-deferred backlog rows.
2. those current counts and ownership rules are now aligned across:
   - `docs/PARKED_CURRENT_BASELINE_20260401.md`,
   - `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`,
   - `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`,
   - `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`,
   - `docs/WORKSET_REGISTER.md`,
   - `docs/worksets/README.md`,
   - `docs/IN_PROGRESS_FEATURE_WORKLIST.md`.
3. the active tree no longer depends on the removed wave-3 and wave-4 packet-local artifacts as live truth surfaces; retained references to those removed files now survive only in:
   - `docs/HISTORY.md`,
   - git history and tag `OxFunc_V1`.
4. current support and evidence claims remain reproducible from the surviving live set:
   - active workset authorities,
   - retained function-slice contract prelims,
   - `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`,
   - retained probe scripts under `tools/`,
   - source implementations under `crates/oxfunc_core/src/functions/`,
   - surviving Lean artifacts under `formal/lean/`,
   - the parked `V1` export and readme surfaces.

### Phase G: First Post-Migration Execution
Deliverables:
1. choose the first workset to execute under beads,
2. roll it into epics and initial beads,
3. execute and report against the new model.

Planned first candidate:
1. `W069` Semantic Witness Snapshot V2.

Exit condition:
1. one real OxFunc workset has executed under the new bead doctrine,
2. the repo no longer relies on the old ad hoc model for active work.

Current execution encoding:
1. post-migration rollout epic:
   - `oxf-xw6`
2. rollout and closure beads:
   - `oxf-xw6.1` roll out `W069` child epics and first execution beads,
   - `oxf-xw6.2` execute the first real post-migration `W069` bead and record proof-of-use,
   - `oxf-xw6.3` record `W070` closure evidence and demote this packet to plan-only status

Phase note:
1. live progress for Phase G is now owned by `.beads/`; this packet names the intended closure path but is no longer the execution tracker.

Current proof-of-use note:
1. the first real post-migration `W069` execution output is now the bounded seed witness artifact:
   - `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json`
2. that artifact proves the repo has moved from migration-only cleanup into ordinary post-migration work under beads because it:
   - was added through a live `W069` bead after the graph migration completed,
   - projects retained `V1` export facts into the new witness shape,
   - ties the first seeded family (`HLOOKUP` / `VLOOKUP`) to surviving contract, replay, runtime, and Lean surfaces,
   - lands as an ordinary live truth artifact rather than a migration-only note.

## 11. Recommended First Live Sequencing
Recommended order:
1. Phase A
2. Phase B
3. Phase C
4. Phase D
5. Phase E
6. Phase F
7. Phase G

Reason:
1. OxFunc should not seed beads before doctrine names the bead model,
2. it should not delete active docs before classifying them,
3. it should not start `W069` under beads until execution truth has a real home.

## 12. Deliverables By File
Expected new or rewritten active files:
1. `docs/WORKSET_REGISTER.md`
2. `docs/BEADS.md`
3. `OPERATIONS.md`
4. `AGENTS.md`
5. `README.md`
6. maybe `docs/HISTORY.md`
7. minimal `scripts/check-worksets.ps1`
8. optional bead seed/bootstrap script

Expected active files to shrink or leave the active tree:
1. `docs/worksets/README.md`
2. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
3. `CURRENT_BLOCKERS.md`
4. large sets of closed workset docs
5. large sets of closed packet-local execution aids

## 13. Reporting Rule During Migration
Every migration round should report against this workset using:
1. current phase,
2. files added/rewritten/removed,
3. active-tree reduction achieved,
4. remaining live doctrinal contradictions,
5. any evidence-risk created by removals,
6. next migration phase.

Do not report migration progress as a generic cleanup note with no phase anchor.

Execution-state note:
1. live issue state for the remaining `W070` phases now lives in `.beads/`,
2. the remaining Phase E, Phase F, and Phase G work is fully encoded there through the `oxf-7xi*`, `oxf-o62*`, `oxf-pyf*`, and `oxf-xw6*` issue families,
3. this packet remains the migration plan and closure-criteria surface,
4. it should no longer be used as a checklist substitute for the bead graph.

Closed reading after the final closure-proof bead:
1. all Phase A through Phase G conditions are now satisfied for the migration itself,
2. `.beads/` owns live execution truth,
3. `docs/WORKSET_REGISTER.md` owns ordered workset truth,
4. this packet now remains as migration plan and provenance only,
5. ordinary follow-on execution proceeds through the live workset register and bead graph rather than through `W070`.

## 14. Closure Condition
`W070` is complete only when all hold:
1. OxFunc doctrine consistently describes the bead model,
2. `.beads/` exists and owns live execution state,
3. a workset register owns ordered workset truth,
4. active-tree execution clutter has been materially reduced,
5. historical surfaces are preserved by git/tag rather than left active by
   default,
6. one real post-migration workset has executed under the new model.

## 15. Immediate Next Step
The next correct move under this plan is:
1. continue ordinary post-migration execution under the live workset register and bead graph,
2. widen `W069` from the first `HLOOKUP` / `VLOOKUP` seed artifact into a broader witness-generation path,
3. keep `W070` as closure provenance only unless a later migration mismatch explicitly reopens it.

## 16. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none
