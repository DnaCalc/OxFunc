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
- `docs/WORKSET_REGISTER.md` - current ordered workset truth for post-park OxFunc.
- `docs/BEADS.md` - local `workset -> epic -> bead` execution method.
- `.beads/` - live execution-state surface for open OxFunc work.
- `docs/worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md` - current migration plan for bead-based execution and active-tree reduction.
- `docs/worksets/` - workset packets and historical provenance; not the live execution-state surface.
- `docs/function-lane/` - mutable function/value working artifacts.
- `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` - downstream metadata, help, and signature contract for DNA OneCalc and other consuming hosts.
- `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md` - surface admission categories and labeling rules for downstream consumers.
- `docs/FOUNDATION_SPEC_INDEX.md` - indexed read links into Foundation doctrine and reference corpus.
- `crates/` - Rust runtime/function scaffolding for executable slices.
- `formal/lean/` - Lean formalization scaffolding for function/value proofs.
- `CURRENT_BLOCKERS.md` - exceptional prose blocker ledger only; ordinary execution blockers live in `.beads/`.
- `docs/IN_PROGRESS_FEATURE_WORKLIST.md` - in-progress feature register.
- `docs/PARKED_CURRENT_BASELINE_20260401.md` - parked non-deferred baseline note.
- `docs/HISTORY.md` - compact pointer for historical packets removed from the active tree.
- `docs/decisions/README.md` - decision register (`ODR-FN-NNN` entries).
- `docs/handoffs/HANDOFF_REGISTER.csv` - cross-repo handoff register.
- `docs/upstream/NOTES_FOR_OXFML.md` - outbound observation ledger for OxFml.

## Notes
- Function behavior now has a dual version axis (Excel app version/channel plus workbook Compatibility Version), reflected in the OxFunc charter.
- OxFunc assumes read access to Foundation artifacts but does not assume direct-write workflow to Foundation during routine iteration.
- Completeness reporting is scope-qualified by doctrine; see `CHARTER.md` section `7.4` and `OPERATIONS.md` section `11`.
- OxFunc now uses `docs/WORKSET_REGISTER.md` for ordered workset truth and `.beads/` for live execution-state truth; `docs/BEADS.md` defines the local bead method.
- OxFunc does not accept bounded-fit function implementations. A function is only considered implemented when the runtime and the formalization work required by the executable-semantic-model strategy cover the full documented and empirically observed Excel semantics for the declared version scope; the only tolerated limitation is in the XLL verification seam.
- For the current implementation phase, function closure is reported as `function-phase-complete` when the reference-baseline semantics and evaluation seam are understood, no known function-semantic gap remains, and the Lean/formal work required by the function's primary semantic substrate and admitted slice has been attended to and aligned; locale/version sweeps are tracked as later orthogonal validation phases unless explicitly in scope.
- XLL verification-seam limitations must be documented centrally in the seam records and repeated in function verification records wherever those limitations affect the meaning of a parity or closure claim.
