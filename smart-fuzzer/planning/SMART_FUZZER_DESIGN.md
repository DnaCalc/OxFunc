# Smart Fuzzer Design

Status: `planning_sandbox`

## 1. Goal

The smart-fuzzer should find invocations where OxFunc/OxFml behavior diverges
from Excel, and it should quantify the explored surface well enough to improve
regression confidence without pretending that sampled agreement is semantic
closure.

The system explores a huge invocation space:

1. function and operator identity,
2. arity and syntactic omission,
3. value kind and value payload,
4. scalar, array, reference-like, callable, rich-value, and provider lanes,
5. caller context, workbook compatibility version, locale/profile, and host
   capability,
6. evaluation surface: Rust function call, OxFml adapter, worksheet formula,
   COM/Excel, and later selected XLL seams.

## 2. Core Principle

Excel evaluations are scarce. Rust evaluations are cheap.

Use Rust and static metadata to explore broadly, then spend Excel budget on
cases that are likely to teach something:

1. new local outcome class,
2. new argument-kind combination,
3. edge-value crossing,
4. known-risk family,
5. source-code branch/risk signal,
6. mismatch with a derived metamorphic expectation,
7. near a previously confirmed bug stream,
8. low local confidence because the function is seam-heavy or host-dependent.

Artifact discipline is part of the design. The pass-to-fail ratio should be
high, so per-case documentation cannot be heavy. Most passing cases should live
only as compact structured rows with enough information to support statistics,
replay sampling, deduplication, and future prioritization. Detailed human notes
belong on mismatch clusters, reduced failures, promotion candidates, and small
coverage exemplars.

## 3. Inputs

Primary local inputs:

1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
3. `docs/function-lane/VALUE_UNIVERSE_PRELIM_SPEC.md`
4. `docs/function-lane/COERCION_AND_CONVERSION_PRELIM_SPEC.md`
5. `docs/function-lane/FUNCTION_SLICE_*_CONTRACT_PRELIM.md`
6. `docs/function-lane/*SCENARIO_MANIFEST_SEED.csv`
7. `crates/oxfunc_core/src/functions/*.rs`
8. `crates/oxfunc_core/tests/fixtures/*.json`
9. `docs/bugs/BUG_*_REGISTER.csv` and `docs/bugs/streams/*.md`
10. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`

These inputs give the fuzzer function identities, arity profiles, prepared
argument profiles, coercion lift profiles, seam-heavy rows, known bug families,
and existing replay idioms.

## 4. Invocation Case Model

Every generated case should have a structured record, not only a formula string.
That record should be compact by default. It is not a narrative artifact unless
the case is promoted because it fails, minimizes a failure, or explains a
coverage boundary.

Minimum fields:

1. `case_id`
2. `generator_id`
3. `source_seed_ref`
4. `function_id`
5. `canonical_surface_name`
6. `entry_kind`
7. `arity_shape`
8. `argument_specs`
9. `formula_text`
10. `caller_locus`
11. `cell_fixture`
12. `context_bundle`
13. `evaluation_surfaces`
14. `local_outcome`
15. `excel_outcome`
16. `comparison_policy`
17. `comparison_result`
18. `classification`
19. `minimization_state`
20. `promotion_state`

For high-volume passing cases, the persisted row can omit verbose fields when
they are reproducible from:

1. generator id,
2. seed,
3. static index version,
4. formula text or structured invocation digest,
5. outcome digest,
6. coverage bucket ids.

Argument specs must preserve the distinctions OxFml currently depends on:

1. literal scalar,
2. direct array literal,
3. opaque array value,
4. area reference,
5. same-sheet multi-area reference,
6. mixed-sheet or unsupported reference source,
7. omitted argument,
8. missing argument,
9. blank cell,
10. empty text,
11. worksheet error,
12. callable value,
13. rich or presentation-bearing return surface where applicable.

## 5. Outcome Model

Comparison must be typed and layered.

Core observed fields:

1. `execution_status`: observed, failed, skipped, blocked.
2. `semantic_status`: value, worksheet_error, spill, rich_value,
   presentation_value, reference_like, bind_reject, seam_reject,
   harness_blocked.
3. `value_payload`: canonical scalar, array, error code, reference summary, or
   rich-value summary.
4. `display_payload`: Excel `.Text` or display-oriented observation when
   relevant.
5. `shape`: scalar or array shape.
6. `numeric_equivalence`: exact bits, exact decimal string, ULP distance,
   family tolerance, or not numeric.
7. `source_surface`: direct Rust, OxFml adapter, Excel worksheet, XLL bridge.

Important rule: seam-level failures and function-semantic mismatches must stay
separate. A blocked host-query, missing provider, XLL marshalling limit, or
bind/admission reject is not automatically an OxFunc semantic failure.

## 6. Architecture

### 6.1 Static Indexer

Build a derived index from CSV metadata, contract docs, known manifests, bug
streams, and source files.

Useful derived facts:

1. supported/deferred/not-current status overlay,
2. ordinary vs seam-heavy,
3. arity min/max and optional positions,
4. prepared-argument profile,
5. coercion lift profile,
6. kernel signature class,
7. reference visibility,
8. host/provider dependency,
9. current bug/workset family,
10. source-code risk hints.

Initial source-code risk hints can be simple and clean-room safe:

1. many `match` arms over value kinds,
2. custom numeric iteration or convergence loops,
3. direct `f64` equality or domain checks,
4. manual array shape/broadcast logic,
5. `MissingArg` and `EmptyCell` handling,
6. reference resolver calls,
7. host/provider calls,
8. local TODO/panic/unwrap in semantic paths,
9. functions recently touched by bug streams.

### 6.2 Typed Generator

The generator should be grammar-aware and type-aware:

1. start from existing manifest rows and contract examples,
2. mutate values within the declared value universe,
3. vary arity and omitted optional positions,
4. lift scalars into row/column/2-D arrays,
5. turn direct arrays into range fixtures and references,
6. inject errors, blank cells, empty text, booleans, and textified numbers,
7. vary caller locus for implicit intersection and caller-context functions,
8. vary shape around broadcast boundaries,
9. construct same-sheet multi-area references separately from 3D and mixed-sheet
   forms,
10. generate formulas only from structured invocation records.

The generator should keep a reproducible random seed on every case.

### 6.3 Fast Local Evaluator

The fast local lane should execute as much as possible without Excel:

1. direct `eval_surface_value_call` for pure OxFunc function calls,
2. `eval_surface_extended_call` for presentation/rich-value rows,
3. OxFml preparation adapter for parse/bind/prepared-argument surfaces,
4. optional Rust instrumentation counters for semantic branch coverage later.

The local lane should emit typed outcomes even when Excel has not been run.
Those outcomes drive novelty scoring and prioritization.

### 6.4 Prioritizer

Candidate score should combine:

1. static risk score,
2. local outcome novelty,
3. argument-kind novelty,
4. function-family coverage deficit,
5. closeness to known bug classes,
6. metamorphic surprise,
7. shrinkability,
8. Excel cost estimate.

The prioritizer should maintain separate budgets for broad coverage and
high-risk exploitation so a single noisy family does not consume all Excel runs.

### 6.5 Excel Batch Executor

The first Excel runner should be boring and measurable:

1. one long-lived Excel process,
2. formulas written in rectangular batches,
3. calculation mode explicitly controlled,
4. `Formula2`, `Value2`, `.Text`, formula echo, and spill shape captured where
   possible,
5. workbook compatibility metadata captured per run,
6. Excel version/build/channel captured per run,
7. manifest hash and git revision captured per run,
8. hard timeout and per-case failure classification.

The first gate is to measure actual throughput. Thousands of evaluations per
second may be possible for simple formulas in large batches, but the runner
must record cold start, warm process, formula write, calculate, and extraction
costs separately.

### 6.6 Comparator

The comparator should use family-specific policies:

1. exact error-code match for worksheet errors,
2. exact boolean/text match unless a function contract states a normalized
   display form,
3. exact shape match for arrays and spills,
4. numeric exactness class by function family:
   - exact bit or exact `Value2` string where expected,
   - ULP threshold for numeric approximations only when explicitly allowed,
   - family-specific tolerances for known iterative/statistical lanes,
   - no generic epsilon that hides real Excel quirks.
5. separate display mismatch from semantic value mismatch.

All mismatches should be assigned a typed mismatch kind.

### 6.7 Minimizer

When a mismatch is found, reduce it while preserving the mismatch predicate:

1. simplify formula structure,
2. shrink arrays by row/column and by element,
3. shrink numbers toward nearby critical values,
4. remove unused fixture cells,
5. reduce reference shapes,
6. replace formulas with literal values only when that does not erase the seam
   being tested,
7. preserve function-semantic vs seam/harness classification.

Minimization outputs are candidates for bug intake and permanent regression
assets.

### 6.8 Agent Loop

Agent calls should be advisory and artifact-bound.

Useful agent tasks:

1. inspect a mismatch cluster and propose likely owner family,
2. suggest new generator tactics from source-code risk signals,
3. review under-covered function families,
4. produce targeted prompt packets for deeper model review,
5. draft bug intake records from minimized artifacts.

Agent outputs must never become semantics without replay evidence and ordinary
promotion.

## 7. Coverage And Confidence

The fuzzer should report coverage as explored dimensions, not as semantic
closure.

Useful coverage axes:

1. function/operator id,
2. function family and workset owner,
3. arity shape,
4. optional omitted positions,
5. argument value kind vector,
6. array shape class,
7. reference kind,
8. prepared-argument structure,
9. outcome class,
10. error code,
11. numeric domain band,
12. source-code risk bucket,
13. existing bug-family adjacency.

Confidence should be stated as:

1. explored rows,
2. distinct typed partitions hit,
3. Excel comparison count,
4. local-only exploration count,
5. unresolved blocked/harness rows,
6. open mismatch count,
7. promoted regression count.

Do not translate sample count into implementation closure language.

### 7.1 Data Retention Policy

Use three retention tiers:

1. `telemetry_row`
   - compact generated-case and outcome rows for ordinary passes and expected
     rejects;
   - retained for coverage statistics, novelty scoring, and rerun sampling;
   - no per-case prose required.
2. `representative_sample`
   - small selected pass samples for each important coverage bucket;
   - useful for sanity checks and documentation of explored shape;
   - still machine-readable first.
3. `failure_packet`
   - full record for mismatches, unstable outcomes, blocked harness findings,
     and minimized reproducers;
   - includes comparison details, reduction lineage, owner classification, and
     promotion state.

The fuzzer should summarize ordinary passes by function family, argument-kind
vector, arity shape, value-domain bucket, array/reference shape, and outcome
class. It should not generate one markdown note per passing case.

## 8. Pilot Scope

Start with five bounded pilots:

1. live financial-payment residuals:
   - PMT / PPMT and adjacent payment-family lanes.
2. numeric approximation and solver families:
   - normal distribution, `RATE`, `XIRR`, financial iteration lanes.
3. array-lift and broadcast:
   - ordinary operators, text scalar functions, lookup-family array needles.
4. omitted optional arguments:
   - `SORT`, `SORTBY`, `TAKE`, `DROP`, `INDEX`, lookup controls.
5. reference and aggregate preparation:
   - `SUM`, `COUNTBLANK`, criteria/database functions, multi-area materialization.

This pilot surface should distinguish live known bugs from stale bug-stream
signals. Existing bug records are useful prioritization inputs, but the fuzzer
should not believe an old claim without fresh Excel confirmation. In particular,
any remaining `POWER` bug signal is a review target, not an assumed current
failure.

## 9. Promotion Path

A candidate mismatch becomes durable only after:

1. Excel observation is reproducible with version/channel/compat metadata,
2. local outcome is captured from the relevant OxFunc/OxFml surface,
3. comparator classifies the mismatch,
4. minimizer produces a stable reduced case or records why reduction is blocked,
5. the case is routed through `docs/bugs/` if actionable,
6. regression tests or scenario manifests are updated under the owning workset,
7. any seam/handoff requirement is filed if ownership crosses repos.

## 10. Implementation Stages

Sequence-only staged rollout:

1. artifact schema and throughput benchmark,
2. static indexer over metadata and existing manifests,
3. simple typed generator for scalar/array literals,
4. direct Rust evaluator and typed local outcome serializer,
5. Excel batch runner and comparator,
6. minimizer,
7. risk scoring from bug streams and source-code hints,
8. OxFml adapter lane,
9. wider reference/provider/seam-heavy lanes,
10. agent-assisted tactic loop.

Each stage should close through evidence-bearing beads if this becomes an
active workset.
