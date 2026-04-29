# W089 Smart-Fuzzer Sweeping Invocation-Space Exploration

Status: `planning_artifacts_ready_execution_gated`

## 1. Purpose

Plan the next broad smart-fuzzer run over the OxFunc function invocation space.
The immediate focus is not execution. It is to inventory the tweakable
dimensions, assign them to compact coverage telemetry, and prepare a run shape
that can explore widely without turning ordinary passing cases into heavy
documentation.

W089 builds on the W088 pilot substrate. It broadens the target from a bounded
financial-family pilot to a catalog-wide exploration plan that can later spend
local Rust evaluation cheaply and Excel comparison selectively.

## 2. Depends On

1. `W088` for smart-fuzzer pilot artifacts, artifact economy rules, throughput
   measurements, static risk indexing, and the first local-vs-Excel comparator
   loop.
2. `W070` for bead-based execution discipline.
3. `W072` for bug intake, root-cause, regression-stream, and failure promotion
   protocol.
4. `W044` for the current library-context snapshot export input.
5. `W049` for runtime provider/snapshot direction.
6. `W051` for the current in-scope-not-complete surface.

## 3. Parent Doctrine And Spec Surfaces

1. `CHARTER.md`
2. `OPERATIONS.md`
3. `docs/BEADS.md`
4. `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`
5. `smart-fuzzer/README.md`
6. `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md`
7. `smart-fuzzer/planning/RUN_ARTIFACT_CONTRACT.md`
8. `smart-fuzzer/planning/SWEEPING_INVOCATION_SPACE_RUN_PLAN.md`
9. `smart-fuzzer/planning/DIMENSION_INVENTORY_AND_COVERAGE_TAXONOMY.md`
10. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
11. `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
12. `docs/bugs/README.md`

## 4. Upstream Dependencies

1. `OxFml` prepared-argument, parser, evaluator, and reference-materialization
   seams.
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` for the current upstream seam
   floor.
3. Live Excel comparison harness availability for later execution gates.

## 5. Scope

In scope for the planning gate:

1. define the sweep dimensions that can be varied,
2. map dimensions to metadata, value-type, argument-type, source-risk, and
   harness-context inputs,
3. define compact coverage rollups and highlights traces for broad exploration,
4. separate known deviation classes from unexpected mismatch promotion,
5. create the bead sequence for implementing and later running the sweep.

In scope for later W089 execution gates:

1. generate a catalog-wide dimension inventory,
2. extend typed generators and mutators to cover the chosen dimensions,
3. run high-volume local OxFunc/OxFml evaluations,
4. select bounded high-value candidates for Excel,
5. compare typed outcomes and minimize unexpected mismatches,
6. route durable mismatches through the ordinary bug stream.

Out of scope for this planning gate:

1. running the sweep,
2. claiming function semantic closure from sampled passes,
3. repairing known PMT/PPMT/IPMT financial exactness drift,
4. treating any stale `POWER` row as live without fresh confirmation,
5. broad locale or alternate Excel-version sweeps beyond captured dimensions,
6. provider/cube/RTD parity claims where the live provider context is absent.

## 6. Sweep Dimensions

The run plan must account for these primary tweakable dimensions:

1. function identity, aliases, operators, family, volatility, admission status,
   deferred status, known-bug adjacency, and source-risk band;
2. arity, optional-argument omission, explicit missing arguments, too-few and
   too-many calls, variadic limits, and syntax forms;
3. argument declared types, prepared value types, coercion paths, array lifting,
   reference preservation, and scalar-vs-array contrast;
4. numeric bands including signed zero, small/subnormal-adjacent values,
   ordinary magnitudes, integer boundaries, fractional values, date serial
   bands, overflow/underflow neighborhoods, and solver-sensitive seeds;
5. text bands including empty strings, whitespace, numeric-looking text,
   booleans/errors-as-text, casing, Unicode, wildcard/regex-like characters,
   delimiters, and long-string boundaries;
6. logical, error, blank, empty-cell, omitted, and explicit `MissingArg`
   distinctions;
7. array shape, orientation, jagged or invalid shape where representable,
   broadcast expectations, spill size, mismatched dimensions, and grid-limit
   probes;
8. reference shape including scalar cell, rectangular area, same-sheet
   multi-area, cross-sheet references, structured references where available,
   spill anchors, and reference-vs-array substitutes;
9. evaluation context including caller location, workbook compatibility, date
   system, locale/profile, calculation mode, volatile inputs, and host/provider
   capability flags;
10. execution seam including direct Rust value calls, prepared OxFml calls,
    formula text through Excel, and future XLL or host seams;
11. comparison policy including exact typed equality, numeric bit equality,
    error-code equality, array cellwise comparison, and value-vs-display
    separation; current OxFunc parity expectations are bit-exact, so tolerance
    buckets are not accepted as pass classes;
12. exploration feedback including local outcome diversity, branch/error
    discovery, metamorphic relation checks, source-risk ranking, and
    Excel-candidate budget allocation.

## 7. Known Deviation Handling

1. PMT/PPMT/IPMT financial-payment exactness drift is an expected blocked class
   under the existing bug stream. W089 may use it as a reference mismatch lane,
   but must not attempt repair unless that bead is explicitly reopened.
2. Any `POWER` deviation must be freshly confirmed before it is treated as a
   current bug. Stale POWER claims are review inputs, not assumed failures.
3. Provider, cube, RTD, external-link, and other host-dependent lanes should be
   classified as blocked or context-deferred unless the run has the necessary
   host fixture.

## 8. Artifact Economy

The W089 sweep expects a high pass-to-fail ratio.

Rules:

1. passing cases remain compact telemetry rows or aggregate counters,
2. coverage rollups and dimension-roadmap traces are durable planning outputs,
3. detailed packets are reserved for unexpected mismatches, unstable outcomes,
   blocked harness rows, and minimized reproducers,
4. confirmed failures move through `docs/bugs/` and regression streams rather
   than becoming permanent smart-fuzzer prose records.

## 9. Gates

1. Gate 0: planning-only packet and bead graph. No tests or sweeps run.
2. Gate 1: dimension inventory schema and coverage taxonomy.
3. Gate 2: generator matrix and typed mutator expansion plan.
4. Gate 3: local-evaluator dry-run plan and budget, with execution still gated.
5. Gate 4: Excel candidate-selection, batching, and comparison budget plan.
6. Gate 5: explicit user-approved sweeping run.
7. Gate 6: triage, minimization, bug promotion, and roadmap-trace update.

## 10. Initial Epic Lanes

1. dimension inventory and coverage taxonomy,
2. generator matrix and typed mutator plan,
3. local evaluator expansion and dry-run budget,
4. Excel candidate selection and batching budget,
5. blocked/deferred seam classification,
6. roadmap trace and compact reporting artifacts,
7. unexpected mismatch triage and minimization protocol,
8. explicit execution gate for the first sweeping run.

## 11. Reporting Contract

All W089 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Current planning status:

1. `execution_state`: `planning_artifacts_ready_execution_gated`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: first sweeping run execution and post-run triage

## 12. Execution Notes

### 2026-04-29 Dimension Inventory Bead

Bead: `oxf-1avj.1`

Added the W089 dimension-inventory and coverage taxonomy surface:

1. `smart-fuzzer/planning/DIMENSION_INVENTORY_AND_COVERAGE_TAXONOMY.md`
2. `smart-fuzzer/tools/Build-DimensionInventory.ps1`

The builder derives a compact `dimension-inventory-v0.json` from the current
library-context snapshot and related registers. It records arity, value-type,
numeric/text, array, reference, context, execution-seam, bit-exact comparison,
known-deviation, blocked/deferred, and coverage-counter axes for each surface.

This bead did not run a sweep and did not compare against Excel.

Status axes after this bead:

1. `execution_state`: `inventory_schema_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: generator matrix, local dry-run plan, Excel candidate budget,
   blocked seam classification, execution approval, mismatch triage protocol

### 2026-04-29 Planning Artifact Closure Pass

Beads: `oxf-1avj.2` through `oxf-1avj.8`

Added the remaining W089 planning surfaces:

1. `smart-fuzzer/planning/GENERATOR_MATRIX_AND_TYPED_MUTATOR_PLAN.md`
2. `smart-fuzzer/planning/LOCAL_EVALUATOR_DRY_RUN_BUDGET.md`
3. `smart-fuzzer/planning/EXCEL_CANDIDATE_SELECTION_AND_BATCHING_BUDGET.md`
4. `smart-fuzzer/planning/BLOCKED_DEFERRED_SEAM_CLASSIFICATION_MAP.md`
5. `smart-fuzzer/planning/ROADMAP_TRACE_AND_COMPACT_REPORTING_ARTIFACTS.md`
6. `smart-fuzzer/planning/UNEXPECTED_MISMATCH_TRIAGE_AND_MINIMIZATION_PROTOCOL.md`
7. `smart-fuzzer/planning/FIRST_SWEEP_EXECUTION_GATE.md`
8. `smart-fuzzer/tools/Build-SweepPlanningArtifacts.ps1`

The planning builder derives generator, local-budget, Excel-budget,
blocked-seam, and roadmap-trace template cache artifacts from the dimension
inventory. These outputs are planning inputs, not run evidence.

This pass did not run a sweep, did not run Excel comparison, and did not run
regression tests.

Status axes after this pass:

1. `execution_state`: `planning_artifacts_ready_execution_gated`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: first sweeping run execution and post-run triage
