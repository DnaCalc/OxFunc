# AGENTS.md - OxFunc Agent Instructions

## Context Loading Order
1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `docs/WORKSET_REGISTER.md`
5. `docs/BEADS.md`
6. `docs/worksets/README.md`
7. `docs/function-lane/README.md`
8. `docs/FOUNDATION_SPEC_INDEX.md`
9. Foundation doctrine docs referenced from the index (`../Foundation/CHARTER.md`, `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`, `../Foundation/OPERATIONS.md`)
10. `CURRENT_BLOCKERS.md`
11. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
12. Inbound observation ledgers from upstream repos (see OPERATIONS.md Section 16.3):
   - `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`

## Source-of-Truth Rules
- For OxFunc-local work, treat `CHARTER.md` in this directory as the working charter.
- For OxFunc execution doctrine, treat `OPERATIONS.md` as normative unless it conflicts with charter/Foundation doctrine.
- Treat `docs/WORKSET_REGISTER.md` as the canonical ordered workset surface.
- Treat `docs/BEADS.md` as the canonical local bead-method surface.
- For cross-program doctrine and architecture constraints, treat Foundation docs as authoritative.
- For mutable function-definition work, use `docs/function-lane/*` in this repo.
- For Excel reference/spec corpus and program-level conformance registry, use links listed in `docs/FOUNDATION_SPEC_INDEX.md`.

## No-Compromise Function Doctrine
- OxFunc targets full semantic identity with Excel for each implemented function over the declared version axes.
- Partial or bounded semantic coverage is never an "implemented function" claim; it is work-in-progress scaffolding only.
- If public documentation and empirical Excel behavior differ, record the discrepancy explicitly and implement the empirically observed behavior.
- In the current implementation phase, a function may be reported as `function-phase-complete` when its semantics are characterized with high confidence for the current reference Excel baseline, the function/evaluation seam is understood and documented, the Rust implementation is thorough and tested, and the Lean/formal work required by `docs/function-lane/FORMALIZATION_STRATEGY_EXECUTABLE_SEMANTIC_MODEL.md` for the function's primary semantic substrate and admitted slice has been attended to and aligned; this may be a substrate-level executable model, binding, and alignment layer rather than a full duplicate Lean implementation for the function. No known function-semantic gap may remain in current-phase scope.
- Locale and alternate Excel-version sweeps are separate orthogonal validation phases unless a workset explicitly declares them in scope; they do not by themselves prevent a function from being `function-phase-complete`.
- The only allowed compromise is in the XLL test/verification seam, where harness limits may prevent recreating all host behavior even though the core OxFunc semantics must still target full Excel parity.
- When any known semantic lane remains open for a function, report that function and any packet containing it as `scope_partial`.
- XLL test/verification seam limitations must be documented in the seam-project records and repeated in per-function or per-packet verification records when those limits materially qualify a function claim.

## Clean-room Rule
Use only:
- public specifications/documentation,
- published research,
- reproducible black-box Excel behavior observations.

Do not use proprietary or restricted sources.

## Versioning Reminder
Function behavior must be tracked across two axes:
1. Excel application version/channel.
2. Workbook Compatibility Version.

## Anti-Premature-Completion Doctrine

Extends the No-Compromise Function Doctrine above with mechanical rules. This section is binding. Violations are doctrine failures, not style preferences.

### Rule 1: Restricted Completion Language
The words "implemented", "closed", "done", and "complete" are forbidden when describing:
- partial subsets of declared scope,
- scaffolding, stubs, or compile-only code,
- merely enabled paths without exercised evidence,
- contract text without replay/test evidence.

Use "in-progress", "partial", or "scaffolded" instead.

### Rule 2: Self-Audit Required Before Completion Claims
Before ANY completion claim, the agent must:
1. Run the Pre-Closure Verification Checklist from OPERATIONS.md Section 12.
2. Run the Completion Claim Self-Audit from OPERATIONS.md Section 14.
3. Include the checklist and self-audit results in the completion report.

### Rule 3: Three-Axis Reporting Mandatory
Every status report must include:
- `scope_completeness` (`scope_complete` | `scope_partial`)
- `target_completeness` (`target_complete` | `target_partial`)
- `integration_completeness` (`integrated` | `partial`)
- explicit `open_lanes` list when any axis is partial

### Rule 4: Scaffolding Is Not Implementation
Stubs, empty traits, compile-only code, and placeholder implementations are scaffolding.
Scaffolding is never reported as implementation. Report it as `scaffolded`.

### Rule 5: Spec Text Without Evidence Is Not Done
Contract or spec text without at least one deterministic replay artifact or exercised test proving intended behavior is not done. Report it as `spec_drafted`.

### Rule 6: Cross-Repo Handoff Is Not Completion
Filing a handoff packet to OxFml opens a dependency — it does not close work.
The originating item remains `in_progress` until the receiving repo acknowledges and integrates.

### Rule 7: Default to In-Progress
When uncertain whether work meets completion criteria, report `in_progress`.

## Continuation Behavior

Mode: **checkpoint-at-gates** with mature-repo calibration.

1. Agent must pause and report status at each workset gate boundary.
2. AutoRun is disabled by default.
3. AutoRun may only be enabled when explicitly requested by the user for a specific scope.
4. Between gates, the agent may proceed autonomously within the declared workset scope (scope-bounded autonomous execution).
5. When AutoRun is explicitly enabled and the user sets a named exit gate, the agent must remain silent until one of only two conditions holds:
   - the declared AutoRun exit gate has been reached, or
   - all remaining in-scope paths are blocked and `CURRENT_BLOCKERS.md` has been updated.
6. User messages such as `continue`, `go on`, or equivalent resume nudges do not create a checkpoint and do not justify an interim status reply while an AutoRun gate remains open.
7. During AutoRun, partial progress must be recorded in repository artifacts, checklist rows, and blocker entries rather than emitted as chat status.
8. If a response is required because all remaining paths are blocked, the response must be a blocker summary only; it must not be framed as a checkpoint or partial completion report.

Transition note:
1. OxFunc is currently in `W070` Phase E-prep state.
2. `.beads/` is now bootstrapped and is the ordinary blocker surface.
3. `CURRENT_BLOCKERS.md` remains only as a transitional prose ledger pending narrower retention or retirement during later `W070` archive waves.
4. do not create new ordinary blocker entries in `CURRENT_BLOCKERS.md` unless the blocker cannot be represented cleanly in the bead graph.

Mature-repo note: OxFunc has 13+ worksets and 38 function-phase-complete functions, exceeding the 5-workset threshold for conservative gate-pausing. Gate discipline remains mandatory but the agent has demonstrated execution history to support scope-bounded autonomy between gates.

## Blocker Handling

When a blocker is encountered:

1. Record the blocker in `.beads/` through the ordinary bead graph.
2. Continue with other non-blocked work within scope.
3. If all paths are blocked, emit a structured summary:
   - blocked items with `BLK-*` identifiers,
   - current state of each,
   - exact unblock steps required,
   - recommendation (wait / escalate / workaround).

Post-bootstrap rule:
1. ordinary blockers belong in the bead graph rather than in new prose blocker notes.

## Change Discipline

1. Keep changes minimal, explicit, and testable.
2. Changes to FEC/F3E boundary semantics (function-facing declarations, coercion policies, admission contracts) require cross-repo impact assessment before promotion.
3. When proposing changes that affect OxFml evaluator-facing clauses, file a handoff packet and register it in `docs/handoffs/HANDOFF_REGISTER.csv`.
4. Neither repo marks a seam change as "complete" until both sides acknowledge.
