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

## Optimization Direction

OxFunc is also the semantic kernel surface for future high-throughput DNA Calc recalculation.

The intended stack shape is:
1. OxFml compiles formula structure: slots, scopes, LET/LAMBDA binding, references, child evaluation order, lazy control forms, and trace policy.
2. OxFunc owns every function/operator semantic: coercion, array lifting, reference-visible behavior, helper/callback semantics, errors, host/provider projection, volatility, locale, and dependency declarations.
3. DNA Calc and host layers may optimize workbook graphs, scheduling, caching, concurrency, and future compiled backends by consuming OxFunc's resolved call-site handles and metadata.

Current direction from W096:
1. resolve function/operator identity once through `SurfaceCallSite`,
2. invoke through uniform catalog-keyed dispatch rather than broad string matching,
3. expose optimizer metadata and hoistability gates as contract data,
4. use reusable runtime-context and scratch-buffer seams for hot paths,
5. avoid moving function-specific shortcuts into OxFml or DNA Calc.

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
- `docs/KNOWN_EXACTNESS_DEVIATIONS.md` - project-wide register of current
  OxFunc-vs-Excel exactness residuals that are known but not waived.
- `docs/worksets/W070_OXFUNC_BEADS_MIGRATION_AND_ACTIVE_TREE_REDUCTION.md` - current migration plan for bead-based execution and active-tree reduction.
- `docs/worksets/` - workset packets and historical provenance; not the live execution-state surface.
- `docs/function-lane/` - mutable function/value working artifacts.
- `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` - downstream metadata, help, and signature contract for DNA OneCalc and other consuming hosts.
- `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md` - surface admission categories and labeling rules for downstream consumers.
- `docs/FOUNDATION_SPEC_INDEX.md` - indexed read links into Foundation doctrine and reference corpus.
- `crates/` - Rust runtime/function scaffolding for executable slices.
- `formal/lean/` - Lean formalization scaffolding for function/value proofs.
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
- Known exactness residuals are tracked centrally in
  `docs/KNOWN_EXACTNESS_DEVIATIONS.md`; entries there are not passes, waivers,
  or tolerance allowances.
- For the current implementation phase, function closure is reported as `function-phase-complete` when the reference-baseline semantics and evaluation seam are understood, no known function-semantic gap remains, and the Lean/formal work required by the function's primary semantic substrate and admitted slice has been attended to and aligned; locale/version sweeps are tracked as later orthogonal validation phases unless explicitly in scope.
- XLL verification-seam limitations must be documented centrally in the seam records and repeated in function verification records wherever those limitations affect the meaning of a parity or closure claim.
