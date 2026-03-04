# AGENTS.md - OxFunc Agent Instructions

## Context Loading Order
1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `TUX1000_PLAN.md`
5. `docs/function-lane/README.md`
6. `docs/FOUNDATION_SPEC_INDEX.md`
7. Foundation doctrine docs referenced from the index (`../Foundation/CHARTER.md`, `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`, `../Foundation/OPERATIONS.md`)

## Source-of-Truth Rules
- For OxFunc-local work, treat `CHARTER.md` in this directory as the working charter.
- For OxFunc execution doctrine, treat `OPERATIONS.md` as normative unless it conflicts with charter/Foundation doctrine.
- For cross-program doctrine and architecture constraints, treat Foundation docs as authoritative.
- Treat `TUX1000_PLAN.md` as aspirational planning guidance; it does not override doctrinal docs.
- For mutable function-definition work, use `docs/function-lane/*` in this repo.
- For Excel reference/spec corpus and program-level conformance registry, use links listed in `docs/FOUNDATION_SPEC_INDEX.md`.

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
