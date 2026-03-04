# OxFunc

OxFunc is the function-semantics and implementation lane for DNA Calc worksheet compatibility.

OxFunc is the canonical owner for mutable value-type/function-definition working docs.
Foundation remains the canonical owner for external Excel reference/spec corpus artifacts.

## F3E Position
OxFunc is positioned as the value/function slice of `F3E`:
1. owns worksheet value types and coercion semantics,
2. owns function/operator semantics and contracts,
3. references FEC capability dependencies needed by function evaluation.
4. defines cross-cutting function tags (`deterministic`, `volatile`, `host-interaction`) that FEC policy consumes.

Out of slice:
1. formula language grammar/parse/bind (OxFml lane),
2. FEC host protocol/scheduling/state-machine design (FEC/F3E model lane).

## Ownership Split
1. OxFunc-owned mutable docs:
   - `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md`
   - `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - `docs/function-lane/EXCEL_FUNCTION_DEFINITION_DISCUSSION.md`
   - `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv`
   - `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.md`
2. Foundation-owned reference/spec docs consumed by OxFunc:
   - `../Foundation/reference/conformance/excel-worksheet-engine/functions/XLL_SDK_REGISTRATION_AND_TYPES_REFERENCE.md`
   - `../Foundation/reference/conformance/excel-worksheet-engine/EXCEL_CONFORMANCE_SPEC.md`
   - `../Foundation/reference/conformance/excel-worksheet-engine/SOURCE_BINDINGS.csv`
   - `../Foundation/reference/downloads/*` and `../Foundation/reference/index.*`

## Core Files Here
- `CHARTER.md` - OxFunc charter (canonical for OxFunc lane).
- `OPERATIONS.md` - OxFunc execution doctrine (lane-level operations).
- `TUX1000_PLAN.md` - aspirational execution adjunct to the charter.
- `docs/worksets/` - sequence-based execution worksets for cross-cutting slices.
- `docs/function-lane/` - mutable function/value working artifacts.
- `docs/FOUNDATION_SPEC_INDEX.md` - indexed read links into Foundation doctrine and reference corpus.
- `crates/` - Rust runtime/function scaffolding for executable slices.
- `formal/lean/` - Lean formalization scaffolding for function/value proofs.

## Notes
- Function behavior now has a dual version axis (Excel app version/channel plus workbook Compatibility Version), reflected in the OxFunc charter.
- OxFunc assumes read access to Foundation artifacts but does not assume direct-write workflow to Foundation during routine iteration.
