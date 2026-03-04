# OxFunc Doc Ownership Recommendation

## Status
Adopted.

Active function-lane working docs are now owned in `OxFunc/docs/function-lane/`.
Foundation remains upstream doctrine/architecture owner and Excel reference/spec corpus owner.

## Why
- Function semantics work will iterate rapidly and benefits from local cohesion with implementation/proof/test assets.
- Keeping rapid-edit docs in Foundation risks split focus with broader program docs.
- Foundation should retain stable references and integration summaries, not high-churn function-lane drafting.

## Active Split
1. Keep in `Foundation` (authoritative program-level):
- `CHARTER.md`
- `ARCHITECTURE_AND_REQUIREMENTS.md`
- `OPERATIONS.md`
- `reference/conformance/excel-worksheet-engine/EXCEL_CONFORMANCE_SPEC.md` (as cross-lane integration index)

2. Owned in `OxFunc` (active ownership):
- function-lane specs and conformance tables
- function classification and signature inventories
- function formal contracts and proof plans
- function-oriented empirical scenario packs and reports

3. Linked from both sides:
- Foundation keeps pointers to OxFunc-owned artifacts.
- OxFunc keeps pointers to Foundation doctrine and program-level constraints.

## Migration Outcome
- Coherent batch migration completed for charter + function spec + conformance CSV + discussion + interesting-function classification docs.
- Foundation files at prior function-lane paths are now redirect stubs to OxFunc canonical locations.
- Foundation retains `XLL_SDK_REGISTRATION_AND_TYPES_REFERENCE.md` as reference-owned artifact.
