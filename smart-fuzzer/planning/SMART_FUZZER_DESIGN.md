# Smart Fuzzer Design

Status: `active_design`

## 1. Goal

The OxFunc parity target is **bit-exact emulation of Excel** for every in-scope function and operator (`517` rows = `494` functions plus `23` operators, after `17` deferred rows in `W050`). All in-scope rows are equally active.

The smart-fuzzer exists to **find OxFunc-vs-Excel discrepancies, biased toward the structural and elusive class** rather than toward piling up additional LSB witnesses in numeric kernels already known to drift.

### 1.1 Bug-Severity Grading

Any OxFunc-vs-Excel discrepancy on an in-scope row is a bug. Bit-exact Excel parity is the goal **including matching Excel where Excel itself is imprecise**. We have not decided that analytical correctness wins over Excel matching; the default repair direction is to match Excel, and the imprecision is recorded.

Severity is graded by magnitude and nature, but the two classes below are both bugs:

1. **Structural mismatch — top priority, fix on discovery.** Wrong value kind, wrong error code, wrong shape/spill behavior, wrong array lift, wrong handling of error/blank/missing/reference inputs, unexpected crash or rejection, generator-induced harness mismatch. Root cause may be the kernel, the function metadata, or the harness; all three are bugs and must be diagnosed before more generation is spent.
2. **Numeric drift — continuous-triage class.** Float drift in numeric kernels. `> 1` ULP drift is more serious than `1` ULP drift, but both are bugs. Each row gets a witness and a `BUG-FUNC-*` stream; repair priority is set by magnitude, scope, and cost.

Sub-tag for numeric drift: a row where OxFunc returns the analytic exact value and Excel is `±1` ULP off is tagged `excel_imprecision_witness` so the *repair direction* is visible — the OxFunc kernel needs to match Excel's imprecise result, not stay analytically correct. The row is still an OxFunc bug under the bit-exact policy and stays in the numeric-drift bug count.

Smart-fuzzer prioritization, comparator classification, and reporting must keep structural and numeric-drift classes visually distinct. A run summary that hides structural mismatches inside a numeric-drift bucket — or hides the `excel_imprecision_witness` sub-tag outside the bug count — is a regression of the fuzzer itself, not just a missing fix.

### 1.2 Scope Boundary

The exploration space is large:

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

### 2.1 AFL/AFL++ Reference Translation

External fuzzer methodology may inform smart-fuzzer tooling design, but it
does not provide Excel semantics. The clean-room rule still limits function
truth to public specifications, published research, and reproducible black-box
Excel observations.

Primary methodology references:

1. Original AFL project page: `https://lcamtuf.coredump.cx/afl/`.
2. Google AFL archive and README: `https://github.com/google/AFL`.
3. AFL++ project: `https://aflplus.plus/` and
   `https://github.com/AFLplusplus/AFLplusplus`.
4. AFL++ feature inventory:
   `https://aflplus.plus/docs/features/`.
5. AFL++ custom mutator documentation:
   `https://aflplus.plus/docs/custom_mutators/`.

The transferable ideas are:

1. **Feedback-guided queue.** AFL keeps inputs that discover new instrumented
   behavior and mutates from that queue. OxFunc should keep structured cases
   that discover new local code coverage, semantic buckets, outcome classes,
   or Excel mismatch classes.
2. **Small starting corpus.** AFL prefers compact, functionally distinct seed
   inputs. OxFunc seeds should be existing scenario manifests, minimized bug
   witnesses, representative pass samples, and small hand-authored edge cases,
   not huge exhaustive manifests.
3. **Corpus culling and minimization.** AFL-style `cmin` and `tmin` map to
   OxFunc queue culling, semantic-bucket-preserving reduction, and
   mismatch-preserving minimization before bug promotion.
4. **Dictionaries and structured mutators.** AFL dictionaries and AFL++ custom
   mutators map to OxFunc typed mutators over invocation records. Raw byte
   mutation over formula text is secondary because it wastes Excel budget on
   invalid syntax and erases prepared-argument distinctions.
5. **Persistent fast local loop.** AFL++ persistent mode maps to long-lived
   local Rust/OxFml harnesses and batched evaluation. Excel remains a separate
   scarce oracle and should be batched, not placed inside the hot mutation
   loop.
6. **Comparison-guided exploration.** AFL++ LAF-Intel/CmpLog-style ideas map
   to OxFunc boundary-hint generation: produce values near domain checks,
   equality comparisons, solver thresholds, overflow/underflow limits, shape
   transitions, and error-code branch points observed in local code. This is
   a generation tactic, not reverse engineering of Excel internals.
7. **Beyond crashes.** AFL explicitly supports using fuzzing to find
   non-crashing design or implementation errors by turning invariant failures
   into a fuzzer-visible failure. OxFunc should treat local invariant breaks,
   local-vs-local disagreement, and local-vs-Excel typed mismatches as
   interesting outcomes while still routing durable Excel findings through the
   ordinary bug stream.

The non-transferable pieces are equally important:

1. Do not instrument Excel; it is a black-box oracle.
2. Do not let code coverage stand in for Excel semantic coverage.
3. Do not treat AFL-found local crashes or panics as function closure evidence.
4. Do not spend Excel budget on raw syntactic fuzzing until a structured
   invocation has survived local validity checks.

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
6. `numeric_equivalence`: exact bits, exact decimal string, diagnostic ULP
   distance, or not numeric; diagnostic distance is not a pass class under the
   current bit-exact parity policy.
7. `source_surface`: direct Rust, OxFml adapter, Excel worksheet, XLL bridge.

Important rule: seam-level failures and function-semantic mismatches must stay
separate. A blocked host-query, missing provider, XLL marshalling limit, or
bind/admission reject is not automatically an OxFunc semantic failure.

### 5.1 Excel Comparator Plumbing

A comparator that claims **bit-exact typed equality** between OxFunc and live
Excel must not pass numeric inputs through formula literal text. Excel's
formula parser is not always correctly-rounded for the long literals a
fuzzer-generated `{value:.17}` or `{value:.17E}` print produces, especially
when the integer part has already consumed much of the f64 significand
budget. The result is that `=FUNC(text_for_v)` may compute `FUNC(v')` for
some `v' ≠ v`, and the comparator then reports drift caused by the harness
rather than by either side's algorithm.

Numeric inputs must therefore be passed through cell `Range.Value2`, which
is bit-exact for every IEEE-754 double Excel accepts (subnormals and
non-finites excluded by Excel itself). Formulas reference those cells. The
empirical confirmation, the rule in full, the runner inventory, and the
adoption order live in
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`.

This plumbing rule is binding for new comparator runners and binding before
any new BUG-FUNC-* exactness stream is closed.

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

The comparator should use bit-exact typed parity policies:

1. exact error-code match for worksheet errors,
2. exact boolean/text match unless a function contract states a normalized
   display form,
3. exact shape match for arrays and spills,
4. exact numeric bits or exact `Value2` string for numeric payloads,
5. separate display mismatch from semantic value mismatch.

Approximate numeric agreement may be recorded as diagnostic data for triage,
but it must not classify an OxFunc-vs-Excel row as a pass. Known PMT/PPMT/IPMT
financial exactness drift remains a known deviation class, not a tolerance lane.

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

### 6.9 AFL-Style Semantic Feedback Queue Prototype

The first AFL-inspired prototype should be local-first and optional-backend.
It may use AFL++ through a Linux/WSL toolchain later, but the first useful
prototype can be an in-repo Rust loop that copies AFL's queue discipline
without adding a hard external dependency.

Prototype scope:

1. target one low-pressure pure-value function family already admitted to the
   current OxFunc-accessible runner,
2. consume structured invocation records rather than raw formula text,
3. mutate with the existing typed mutator vocabulary,
4. run only the cheap local OxFunc/OxFml surface in the hot loop,
5. retain cases that add new coverage or semantic signal,
6. batch a small favored subset through Excel after local queue growth,
7. write only compact queue and rollup artifacts unless a mismatch is found.

Prototype interestingness signals:

1. new instrumented Rust edge or region coverage when available,
2. new function id plus arity-shape bucket,
3. new argument value-kind vector,
4. new coercion or array-lift bucket,
5. new shape class or reference-kind bucket,
6. new local semantic outcome class,
7. new worksheet error code,
8. new local panic, rejection, or unstable outcome,
9. new local-vs-Excel typed mismatch class after oracle sampling,
10. successful minimization of an existing mismatch.

Prototype artifacts:

1. `feedback_queue.jsonl` — retained interesting cases with parent linkage,
   generator, mutator, seed, and reason retained.
2. `feedback_coverage.json` — aggregate code and semantic bucket counters.
3. `favored_cases.jsonl` — small queue subset selected for Excel spend.
4. `queue_cull_report.json` — cases removed or superseded by smaller cases
   covering the same signal.
5. ordinary `failure_packets/` and `minimized/` outputs for durable
   mismatches.

Exit gates:

1. the prototype can reproduce at least one existing promoted bug witness from
   a smaller seed or adjacent mutation,
2. or it grows the semantic bucket set without finding a mismatch and records
   the plateau honestly,
3. or it finds a new durable mismatch and routes it through `docs/bugs/`,
4. or it is blocked because the local harness cannot yet expose useful
   coverage or semantic-bucket signals.

This prototype is exploration infrastructure. It must not be reported as
function implementation evidence or semantic closure.

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
should not believe an old claim without fresh Excel confirmation. The former
`POWER` stale signal was freshly confirmed and closed under W078 on 2026-04-29;
future POWER mismatches should be opened as new signals.

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
10. AFL-style semantic feedback queue prototype,
11. optional AFL++/LibAFL backend experiment if the in-repo queue proves useful,
12. agent-assisted tactic loop.

Each stage should close through evidence-bearing beads if this becomes an
active workset.
