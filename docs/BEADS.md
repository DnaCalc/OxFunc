# OxFunc Beads Working Method

## 1. Purpose
This file defines the local bead method for OxFunc.

It covers:
1. the local execution model,
2. the `br` / `bv` tool split,
3. the bead mutation rule,
4. OxFunc-specific bead quality expectations,
5. the `workset -> epic -> bead` rollout pattern,
6. transition and archive rules during the `W070` migration.

## 2. Core Model
Execution in OxFunc is moving to:
1. [docs/WORKSET_REGISTER.md](WORKSET_REGISTER.md)
2. `workset -> epic -> bead`
3. `.beads/` as the detailed execution truth

Interpretation rule:
1. worksets are high-level planning and scope-partition units,
2. epics are the main execution lanes under a chosen workset,
3. beads are the unit of executable progress,
4. worksets do not carry ready/in-progress/blocked/closed execution state,
5. `.beads/` will become the sole owner of execution-state truth,
6. function-lane evidence artifacts remain evidence/provenance surfaces and are
   not replaced by beads.

## 3. Transition Status
Current migration status:
1. `W070` Phase D is complete.
2. `.beads/` is now bootstrapped and is the live execution-state surface.
3. current migration pressure has moved to Phase E active-tree reduction and
   the later first real post-migration execution work.

Transition rule:
1. use this file and [docs/WORKSET_REGISTER.md](WORKSET_REGISTER.md) to shape
   the live execution model now,
2. use `W070` to drive the remaining migration and archive waves,
3. do not reintroduce ad hoc execution-state notes now that `.beads/` exists.

## 4. Tool Split
`br` is the mutation tool.

Use it to:
1. inspect ready work,
2. create beads,
3. update bead status,
4. add dependencies,
5. close completed beads.

Typical future commands:

```powershell
br ready
br show <id>
br create --title "..." --type task --priority 2
br update <id> --status in_progress
br close <id> --reason "Completed"
br dep add <issue> <depends-on>
```

`bv` is the graph-aware triage and analysis tool.

Use it to:
1. inspect the ready path,
2. identify blockers,
3. inspect graph shape and pressure.

Agent rule:
1. use only non-interactive robot-style inspection calls from agent sessions,
2. prefer machine-readable or robot output modes where available,
3. do not launch blocking interactive views from unattended sessions.

## 5. Mutation Rule
Do not edit `.beads/` files directly.

When `.beads/` exists:
1. use `br` for issue creation and mutation,
2. use `bv` and read-only `br` for graph inspection,
3. keep execution-state truth out of ad hoc notes.

## 6. OxFunc Bead Quality Bar
Every executable OxFunc bead should state:
1. one reviewable implementation outcome,
2. the evidence needed for closure,
3. its parent epic,
4. real dependency relationships,
5. the truth surfaces touched when that matters.

For OxFunc, closure evidence normally means some combination of:
1. implementation code,
2. test code,
3. native replay or differential evidence,
4. Lean/formal alignment where in scope,
5. current truth-surface updates,
6. required downstream or seam handoffs when boundary change is involved.

Bad beads:
1. vague activity without a reviewable outcome,
2. ongoing themes disguised as one issue,
3. mini-worksets hidden inside one bead,
4. local-document-only output unless the bead is making a narrow spec correction,
   active truth correction, or required handoff,
5. claims of closure without replay/test/formal evidence where doctrine requires them.

## 7. OxFunc Epic Shapes
Typical OxFunc epics:
1. runtime/kernel implementation lane,
2. native replay/evidence lane,
3. Lean/formal lane,
4. export/catalog/admission lane,
5. seam/handoff lane,
6. archive/cleanup lane when the workset itself is migration or reduction work.

Not every workset needs every epic.
But any real OxFunc rollout should make the intended lane split explicit.

## 8. Rollout Rule
Any workset chosen for execution should be rolled out into one or more epics.

Rollout pattern:
1. some epics should be expanded into child beads immediately,
2. some epics may begin with a rollout bead when the child set still needs to be
   created or refreshed,
3. obvious implementation work should be expanded directly instead of hidden
   behind narrative placeholders,
4. both patterns are normal as long as the graph stays explicit.

A rollout bead is complete only when:
1. the epic has a believable ready path,
2. the next child beads exist explicitly,
3. the work no longer depends on narrative memory alone.

## 9. Closure Rule
A bead closes only when:
1. the stated outcome exists,
2. the stated evidence exists,
3. any newly discovered required work has already been added back into the graph,
4. the current truth surfaces touched by the bead are updated.

Do not close a bead because "enough progress happened."

Capability-bearing OxFunc beads must normally close on meaningful code plus
verification.
Stub code, placeholder artifacts, and descriptive notes are not sufficient.

## 10. Documentation Rule
After planning is in place, default bead outputs should be:
1. implementation code,
2. test code,
3. narrowly-scoped spec or contract corrections,
4. narrowly-scoped upstream seam handoffs,
5. necessary evidence or reference notes for behavior that now exists in code.

Do not use beads as a reason to multiply local status documents.

## 11. Compact Rollout Template
When a workset is chosen for rollout, capture:

1. Workset:
   - id
   - title
   - scope
   - terminal condition
2. Execution epics:
   - implementation lane
   - evidence lane
   - formal lane
   - seam/integration lane where needed
   - cleanup/archive lane where needed
3. First rollout bead per epic:
   - title
   - one reviewable outcome
   - completion evidence
4. First execution child beads:
   - one clear outcome each
   - explicit dependencies
   - explicit evidence

## 12. Validator
Validator:
1. `scripts/check-worksets.ps1`

Planned minimum checks:
1. the workset register exists,
2. workset ids are unique,
3. the register exposes a coherent active sequence,
4. once `.beads/` exists, the bead workspace exists and can report summary state.

Current note:
1. the minimal validator landed with Phase D.
2. deeper bead-quality or rollout-shape checks may be added later without turning the validator into a second execution-state surface.
