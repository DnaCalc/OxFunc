# W107 Smart-Fuzzer Testing Infrastructure Roadmap

Status: `planned`

## Purpose

Own the long-term testing-infrastructure push that turns the existing
smart-fuzzer from a set of useful explorers and run guides into a durable
function-call-space exploration platform.

The target is not simply more random cases. The target is a typed,
metadata-aware, feedback-guided, replayable explorer that can keep learning
from OxFunc-local runs, spend Excel oracle budget carefully, minimize durable
findings, and route every actionable mismatch through the ordinary bug and
handoff surfaces.

This workset answers the 2026-06-26 planning question: W088, W089, W092, W097,
and W104 cover important smart-fuzzer pilots, sweeps, re-measurement, and
category-split work, but no existing workset owns the platform-level
infrastructure roadmap as one coordinated lane.

`.beads/` owns live readiness, execution state, and blockers once this roadmap
is decomposed into implementable tasks.

## Depends On

1. `W072` for bug intake, regression-stream, and durable mismatch promotion.
2. `W088` for the smart-fuzzer pilot substrate, artifact economy, and first
   local-vs-Excel comparator loop.
3. `W089` for invocation-space dimensions, generator planning, and sweep
   artifact vocabulary.
4. `W092` for long-run guide discipline and stop-gate vocabulary.
5. `W097` for cell-ref Excel comparator plumbing and shared severity helpers.
6. `W104` for the Category 1 / Category 2 invocation-test split and the
   context-sensitive catalog.
7. `W105` for the declarative function-spec direction that gives future
   generators and coverage axes a better metadata spine.

## Canonical Surfaces

1. `docs/worksets/W107_SMART_FUZZER_TESTING_INFRASTRUCTURE_ROADMAP.md`
2. `smart-fuzzer/README.md`
3. `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md`
4. `smart-fuzzer/planning/CASE_SCHEMA_V0.md`
5. `smart-fuzzer/planning/RUN_ARTIFACT_CONTRACT.md`
6. `smart-fuzzer/planning/SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md`
7. `smart-fuzzer/tools/CellRefBatch.psm1`
8. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
9. `smart-fuzzer/tools/pmt_ppmt_local_eval/`
10. `smart-fuzzer/corpus/context_sensitive_catalog/`
11. `.beads/` W107 epic and child lanes, once created

## Current Assessment

The existing smart-fuzzer has strong foundations:

1. structured case and run-artifact drafts,
2. typed local evaluators,
3. generic local-plus-Excel tranche execution,
4. shared cell-ref Excel plumbing,
5. broad scalar, finance, array, statistical, and matrix explorers,
6. real bug-stream promotion history,
7. a context-sensitive catalog for future downstream execution.

The platform gap is that these parts are not yet one learning system. The
feedback queue, queue culling, favored-case selection, automated minimizer,
status-map regeneration, persistent experiment index, true rich-case Excel
batching, typed mutator engine, and Category-1 downstream runner remain
separate plans, partial tools, or future lanes.

## Scope

In scope:

1. stabilize the smart-fuzzer case, telemetry, comparison, failure-packet, and
   feedback-queue schemas into one versioned artifact dialect,
2. build a reusable typed invocation model and runner core inside the
   smart-fuzzer tooling surface, shared by broad scalar, finance, array,
   matrix, stochastic, and future Category-1 runners,
3. upgrade the rich case-set Excel path to true batched worksheet execution
   while preserving `Range.Value2` numeric input plumbing,
4. implement a semantic feedback queue with parent lineage, retention reasons,
   semantic bucket coverage, favored-case selection, and queue culling,
5. implement typed mutators over structured invocations rather than raw formula
   text,
6. add local interestingness signals: semantic buckets, outcome diversity,
   metadata coverage, risk adjacency, metamorphic surprise, and optional
   test-only or feature-gated Rust instrumentation counters,
7. add automated minimization for formulas, arrays, numbers, strings, fixture
   cells, references, optional arguments, and context bundles,
8. add a persistent experiment index for scheduling, deduplication, coverage
   queries, and campaign resumption while keeping tracked docs and promoted
   artifacts as authority,
9. integrate W104's Category-1 context-sensitive catalog into a future
   downstream OxCalc -> OxFml -> OxFunc smart-fuzzer runner,
10. define CI smoke and long-run campaign modes with explicit resource,
    artifact-retention, and stop-gate policies.

Out of scope:

1. repairing function-semantic bugs discovered by the fuzzer,
2. claiming function semantic parity from sampled fuzzer passes,
3. replacing `docs/bugs/`, worksets, or `.beads/` with fuzzer-local trackers,
4. using raw formula-text fuzzing as the primary exploration strategy,
5. treating code coverage, local-only agreement, or metamorphic agreement as
   Excel parity evidence,
6. refactoring operational OxFunc function kernels, dispatch, argument
   preparation, value semantics, FEC behavior, registry semantics, or public
   runtime APIs for smart-fuzzer convenience,
7. implementing provider/cube/RTD live-provider parity without the required
   host fixtures and downstream owners.

## Operational Non-Interference Guardrails

W107 is a testing-infrastructure workset. It surrounds the operational OxFunc
function implementation; it does not refactor that implementation.

Default allowed edit surfaces for W107 execution beads are:

1. `smart-fuzzer/**`,
2. smart-fuzzer-specific test fixtures and replay corpora,
3. documentation and workset/bead planning surfaces,
4. test-only helpers, feature-gated observers, or read-only diagnostics needed
   to expose local outcomes to the fuzzer.

Default forbidden edit surfaces for W107 execution beads are:

1. semantic kernels under `crates/oxfunc_core/src/functions/`,
2. production function dispatch, argument preparation, coercion, lift,
   broadcast, reference, or provider behavior,
3. public runtime API shape used by downstream consumers,
4. function registry facts that affect operational evaluation,
5. production FEC behavior.

If smart-fuzzer work reveals that an operational OxFunc change is needed, that
change must leave W107 and move through the ordinary owning workset, bug stream,
or handoff lane with its own evidence and validation. W107 may supply the
reproducer and minimization artifacts, but it does not carry the production
semantic change.

If a W107 child lane genuinely needs a narrow diagnostic hook inside an
operational crate, the bead must state why the hook is read-only or
test-/feature-gated, show that production behavior is unchanged when the hook is
disabled, and include a focused verification step. Such a hook is
infrastructure plumbing only, not a semantic refactor.

## Platform Principles

1. Excel is the scarce oracle; Rust/OxFml local execution is the hot loop.
2. Numeric inputs for bit-exact Excel comparison must use `Range.Value2`, not
   long decimal formula literals.
3. Structured invocation records are the source of generated formulas, not the
   other way around.
4. Structural mismatches are prioritized ahead of accumulating more witnesses
   in already-known numeric-drift bands.
5. Feedback coverage is exploration telemetry, not a semantic parity claim.
6. A persistent experiment database is a working index, not a source of truth;
   promoted findings still land in tracked bug, corpus, function-lane, or
   handoff artifacts.
7. Category 1 and Category 2 are both smart-fuzzer scope; they differ by runner,
   not by importance.
8. Operational function behavior is an oracle target for the fuzzer, not an
   implementation substrate to reshape under this workset.

## Gates

1. Gate 0: roadmap packet accepted and registered.
2. Gate 1: current smart-fuzzer tool, schema, run-artifact, and status-map gap
   audit recorded.
3. Gate 2: unified schema and typed invocation core defined, with migration
   rules for existing smart-fuzzer runners.
4. Gate 3: rich case-set Excel executor upgraded or replaced with true batched
   execution and phase timing.
5. Gate 4: semantic feedback queue prototype runs locally, emits queue
   artifacts, and preserves parent/mutator lineage.
6. Gate 5: typed mutator engine covers numeric, text, logical, error, blank,
   missing, array, reference, optional-argument, and known-bug-adjacent lanes.
7. Gate 6: automated minimizer emits replayable reduced cases or records why
   minimization is blocked.
8. Gate 7: Category-1 downstream-runner spike executes at least one catalog
   class through the real downstream stack or records concrete blockers.
9. Gate 8: campaign scheduler and CI smoke mode are documented and exercised on
   bounded budgets.

## Initial Epic Lanes

1. infrastructure audit and status-map regeneration,
2. schema and typed invocation core,
3. Excel oracle batching and environment capture,
4. smart-fuzzer local runner consolidation and optional OxFml prepared-call
   harness lane,
5. semantic feedback queue and coverage buckets,
6. typed mutator engine and pairwise or strength-3 scheduler,
7. minimization and promotion pipeline,
8. persistent experiment index and campaign scheduler,
9. Category-1 downstream-runner integration,
10. CI smoke, nightly campaign, and operator runbook.

## Entry Conditions

1. Existing W088/W089/W092/W097/W104 smart-fuzzer surfaces have been read by
   the implementing agent.
2. Current dirty-tree work unrelated to smart-fuzzer infrastructure is either
   landed, intentionally split, or avoided by the chosen child lane.
3. The first execution bead chooses one narrow gate; broad platform work should
   not begin by changing every smart-fuzzer runner at once.
4. Any proposed production-code diagnostic hook is explicitly justified as
   read-only or test-/feature-gated before implementation starts.

## Current Checkpoint

2026-06-26:

1. Investigation found no single existing workset that owns the long-term
   testing-infrastructure platform ambition.
2. W107 is created as the planning and provenance owner for that ambition.
3. No runner code, schemas, or generated smart-fuzzer artifacts are changed by
   this checkpoint.
4. Bead decomposition is pending.

## Doctrine Axes

scope_completeness: `scope_partial`
target_completeness: `target_partial`
integration_completeness: `partial`
open_lanes: `[bead_decomposition, infrastructure_audit, schema_unification, batched_excel_oracle, semantic_feedback_queue, typed_mutator_engine, automated_minimizer, experiment_index, category1_downstream_runner, ci_campaign_modes]`
