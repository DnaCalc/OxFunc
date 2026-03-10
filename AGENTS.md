# AGENTS.md - OxFunc Agent Instructions

## Context Loading Order
1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `TUX1000_PLAN.md`
5. `docs/worksets/README.md`
6. `docs/function-lane/README.md`
7. `docs/FOUNDATION_SPEC_INDEX.md`
8. Foundation doctrine docs referenced from the index (`../Foundation/CHARTER.md`, `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`, `../Foundation/OPERATIONS.md`)

## Source-of-Truth Rules
- For OxFunc-local work, treat `CHARTER.md` in this directory as the working charter.
- For OxFunc execution doctrine, treat `OPERATIONS.md` as normative unless it conflicts with charter/Foundation doctrine.
- For cross-program doctrine and architecture constraints, treat Foundation docs as authoritative.
- Treat `TUX1000_PLAN.md` as aspirational planning guidance; it does not override doctrinal docs.
- For mutable function-definition work, use `docs/function-lane/*` in this repo.
- For Excel reference/spec corpus and program-level conformance registry, use links listed in `docs/FOUNDATION_SPEC_INDEX.md`.

## No-Compromise Function Doctrine
- OxFunc targets full semantic identity with Excel for each implemented function over the declared version axes.
- Partial or bounded semantic coverage is never an "implemented function" claim; it is work-in-progress scaffolding only.
- If public documentation and empirical Excel behavior differ, record the discrepancy explicitly and implement the empirically observed behavior.
- In the current implementation phase, a function may be reported as `function-phase-complete` when its semantics are characterized with high confidence for the current reference Excel baseline, the function/evaluation seam is understood and documented, the Rust implementation is thorough and tested, the Lean description covers the intended slice, and no known function-semantic gap remains in current-phase scope.
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
