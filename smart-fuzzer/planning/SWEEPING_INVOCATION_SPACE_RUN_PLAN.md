# Sweeping Invocation-Space Run Plan

Status: `planning_only`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This plan defines the next smart-fuzzer sweep before execution. The purpose is
to understand and record the adjustable dimensions of the OxFunc invocation
space, then make later generation and comparison runs accountable to compact
coverage evidence.

No tests or sweeps are authorized by this note.

## 1. Objective

Explore the Excel function invocation space broadly enough to discover
unexpected OxFunc-vs-Excel divergences and to describe, with compact telemetry,
which areas of the space were sampled.

The run is not a proof of semantic closure. Passing samples are coverage
signals. Unexpected mismatches, unstable outcomes, and blocked context rows are
the durable outputs.

## 2. Dimension Inventory

The first implementation step is a generated dimension inventory. Each function
row should expose the levers that can change generated invocations.

### Function Surface

Track:

1. function name, aliases, and operator forms,
2. category and family,
3. support phase, deferred status, and current-version scope status,
4. volatility and context sensitivity,
5. array support and spill behavior markers,
6. provider, cube, RTD, async, external-reference, formula-binding, or
   host-context requirements,
7. known bug stream adjacency,
8. source-risk band from static analysis and previous fuzzer evidence.

### Arity And Syntax

Track:

1. minimum and maximum argument count,
2. optional argument positions,
3. variadic groups and practical caps,
4. omitted optional arguments,
5. explicit missing arguments,
6. empty arguments produced by formula syntax,
7. too-few and too-many argument controls,
8. function-call syntax variants where Excel syntax exposes more than one path.

### Value-Type Axes

Track for every argument position:

1. declared type,
2. prepared value type,
3. scalar number, text, logical, error, blank, empty cell, and missing value,
4. reference-preserving vs value-materialized paths,
5. array/scalar lifting expectations,
6. coercion expectations and rejection classes,
7. callable/lambda or unevaluated forms where applicable.

### Numeric Axes

Generate bands rather than flat random doubles:

1. exact zero and signed zero,
2. tiny magnitudes and subnormal-adjacent values where the seam admits them,
3. ordinary small, medium, and large finite magnitudes,
4. integer boundaries including 1, -1, 2, powers of two, and 2^53-adjacent
   values,
5. fractional values near whole numbers and half values,
6. date serial neighborhoods, including 0/1, leap-year boundaries, and negative
   or high serial values where functions accept them,
7. percentage/rate bands including near-zero rates, high rates, and
   solver-sensitive seeds,
8. exponent, logarithm, distribution, and financial overflow/underflow
   neighborhoods,
9. NaN/infinity only as local-seam probes when they are representable and not
   as Excel input claims.

### Text Axes

Track:

1. empty string,
2. whitespace-only text,
3. numeric-looking and date-looking text,
4. boolean-looking and error-looking text,
5. casing and culture-sensitive text,
6. wildcard characters, pattern delimiters, regex-like text, and escapes,
7. delimiter-heavy strings,
8. Unicode samples and normalization-sensitive strings,
9. long-string bands near practical and documented limits.

### Array And Shape Axes

Track:

1. scalar vs row vector vs column vector vs 2D array,
2. empty or empty-ish arrays where representable,
3. single-cell arrays contrasted with scalars,
4. mismatched shape pairs,
5. broadcast and array-lift expectations,
6. spill-size bands,
7. edge probes near Excel grid limits using sampled shapes rather than enormous
   materializations,
8. arrays containing mixed numbers, text, logicals, blanks, errors, and nested
   reference-derived values where admitted.

### Reference Axes

Track:

1. single cell reference,
2. rectangular area,
3. same-sheet multi-area reference,
4. cross-sheet references,
5. whole-row and whole-column references where feasible,
6. spill anchors,
7. structured-reference fixtures where available,
8. reference input contrasted with equivalent array literal,
9. external or provider-bound references as blocked/context lanes unless a
   fixture is explicitly provided.

### Context Axes

Track:

1. Excel application version and channel,
2. workbook compatibility version,
3. date system,
4. locale/profile and decimal/list separators,
5. caller cell location,
6. calculation mode and volatility controls,
7. worksheet dimensions and occupied-neighborhood fixtures,
8. provider/cube/RTD capability flags,
9. OxFml parser/preparation mode and direct Rust-value mode.

## 3. Current Target Universe Boundary

The default W089 target universe is the broad pure-function differential
surface: OxFunc value calls and prepared OxFml calls whose inputs, providers,
reference fixtures, and comparison semantics are explicit in the run manifest.

In the current default target universe:

1. ordinary deterministic value functions are in scope;
2. array and reference-sensitive functions are in scope when the run provides
   explicit array/reference fixtures and reference materialization policy;
3. `RAND`, `RANDBETWEEN`, and `RANDARRAY` stay in scope as stochastic lanes,
   but they require aggregate distribution and invariant checks rather than
   per-draw bit-exact comparison against Excel;
4. `NOW` and `TODAY` stay in scope only when the run declares the date system,
   clock provider, and recalc timing policy;
5. dynamic-array value behavior is in scope, while spill collision, occupied
   worksheet neighborhood, and full-grid limit behavior require workbook
   fixtures.

All default-universe invocations must satisfy the published OxFunc arity
metadata. Calls below `arity_min` or above `arity_max` are outside the pure
OxFunc smart-fuzzer comparison universe because the function call has not
passed OxFml/FEC/F3E admission. They may be kept as admission-negative evidence
for OxFml, but they must not be sent to the Excel COM comparison lane as
semantic parity cases.

Explicitly deferred from the current pure target universe:

1. `RTD` and other async real-time subscription behavior, because comparison
   needs a host subscription provider, scheduler, topic lifecycle, and
   eventual-value policy;
2. `LET`, `LAMBDA`, and formula-scope callable formation or invocation, because
   their Excel comparison depends deeply on OxFml parsing, binding, scope, name
   resolution, and formula evaluation rather than a pure value call;
3. helper-family inline-lambda lanes such as `BYROW`, `BYCOL`, `MAP`, `REDUCE`,
   `SCAN`, `MAKEARRAY`, and `ISOMITTED` unless the run provides a concrete
   callable fixture; built-in-aggregator lanes for `GROUPBY` and `PIVOTBY` may
   be tested separately from inline `LAMBDA` lanes;
4. live provider, cube, web, stock-history, external-link, and other
   external-data lanes unless the run declares a reproducible provider fixture;
5. host/workbook/caller metadata lanes such as `CELL`, `INFO`, `FORMULATEXT`,
   `SHEET`, `SHEETS`, `INDIRECT`, and `OFFSET` unless the run declares a
   workbook/reference/caller fixture;
6. locale and alternate Excel-version sweeps unless the run declares those
   profiles as primary axes.

Deferred rows should remain visible in coverage rollups as explored blocked
lanes. They should not be silently removed from the catalog and should not be
reported as function mismatches.

## 4. Sampling Strategy

The sweep should combine deterministic coverage and feedback-guided exploration.

1. Build a small mandatory basis for each function from metadata: arity edges,
   declared argument-type representatives, omitted/defaulted optional controls,
   scalar/array/reference contrasts, and error/blank lanes.
2. Add pairwise or strength-3 combinations across high-risk dimensions rather
   than full Cartesian products.
3. Use family mutators for numeric solvers, financial functions, lookup
   functions, text slicing/searching, date/time functions, dynamic arrays,
   aggregators, and reference-sensitive functions.
4. Use local outcome diversity as a cheap feedback signal: prioritize inputs
   that produce new typed outcome classes, new error codes, new array shapes, or
   unusual numeric regions.
5. Use static source-risk and known-bug adjacency to spend more Excel budget on
   risky areas without starving low-risk catalog coverage.
6. Use metamorphic relations where they are precise enough to be useful, for
   example inverse pairs, monotonicity probes, shape invariants, alias/operator
   equivalence, or reference-vs-array contrast controls.

## 5. Excel Spend Policy

Local evaluation can be high volume. Excel evaluation must be selective.

Candidate priority:

1. local outcome class not previously seen for the function/dimension band,
2. known-risk or recently touched function family,
3. source-risk hotspots,
4. boundary values and shape edges,
5. randomly sampled low-risk controls for confidence and drift detection,
6. stale-claim checks only as fresh confirmation probes; the former POWER
   stale-claim check was closed under W078 on 2026-04-29.

Known PMT/PPMT/IPMT exactness deviations are expected and should be reported as
known financial drift unless reduction shows a separate root cause.

Stochastic rows should spend Excel quota in repeated batches that support a
declared statistical comparator. A single random draw is not exact parity
evidence.

## 6. Coverage And Roadmap Trace

The durable run output should describe explored areas, not every pass case.

Required compact artifacts for the later run:

1. `manifest.json`: git revision, tree state, runner versions, Excel metadata,
   workbook compatibility, seeds, budgets, and artifact schema version.
2. `dimension_inventory.json`: function-by-function tweakable-dimension map.
   The W089 schema source is
   `smart-fuzzer/planning/DIMENSION_INVENTORY_AND_COVERAGE_TAXONOMY.md`, and
   the derived builder is `smart-fuzzer/tools/Build-DimensionInventory.ps1`.
3. `generator_matrix.json`: generated from
   `smart-fuzzer/tools/Build-SweepPlanningArtifacts.ps1`, with mandatory basis
   tags, typed mutators, and planning budgets.
4. `coverage_rollup.json`: counters by function, family, argument type, arity
   class, value band, array shape, reference kind, context band, local outcome,
   and Excel comparison class.
5. `roadmap_trace.json` and `roadmap_trace.md`: human-readable highlights of
   explored regions, sparse areas, blocked lanes, and next sampling decisions.
6. `telemetry.jsonl`: compact sampled case telemetry, not narrative evidence.
7. `excel_candidates.jsonl`: selected Excel-evaluation candidates and selection
   reason codes.
8. `failure_packets/`: only unexpected mismatches, unstable outcomes, or
   blocked harness rows that need durable review.

## 7. Classification

Every Excel comparison row should land in one of these classes:

1. exact typed match,
2. known expected deviation,
3. unexpected mismatch,
4. Excel harness blocked,
5. OxFml seam blocked,
6. context/provider blocked,
7. invalid generator case,
8. unstable or non-reproducible.

Only unexpected mismatches and unstable outcomes should enter minimization by
default. Known PMT/PPMT/IPMT rows should be summarized separately so they do
not dominate reports.

Current OxFunc comparison policy expects bit-exact matches for all functions.
The sweep should not classify approximate numeric agreement as a pass. Any
future comparator experiment that studies tolerance-like behavior must be
separate from parity classification and must not weaken the default mismatch
rule.

For pseudo-random functions, bit-exact per-draw equality is not the comparison
unit. Those lanes require a separate run-level statistical profile outcome,
for example distribution bounds, range invariants, shape invariants, integer
inclusion rules, and recalculation-change checks. A statistical profile
mismatch is still promoted as a fuzzer finding.

## 8. Agent And Code Feedback Loop

The practical feedback loop should be:

1. code generates the dimension inventory and first coverage matrix,
2. agent review looks for missing dimensions, bad assumptions, and high-risk
   blind spots,
3. code runs local exploration under fixed budgets,
4. code ranks Excel candidates from coverage and local-outcome feedback,
5. Excel comparison produces compact comparison classes,
6. agent review clusters unexpected mismatches and proposes minimization or
   bug-promotion paths,
7. code records minimized reproducers and updates the roadmap trace.

The agent should not manually bless large pass sets. Its useful role is to
review coverage blind spots, mismatch clusters, generator validity, and
promotion decisions.

## 9. Gate Plan

1. Planning gate: this document and W089 beads only.
2. Inventory gate: generated dimension inventory and coverage taxonomy.
3. Generator gate: metadata-driven basis cases plus family mutators.
4. Local gate: high-volume local dry-run and rollup without Excel spend.
5. Excel gate: bounded candidate sample with comparison classes.
6. Triage gate: minimized unexpected mismatches and bug-stream routing.

The first sweep should not start until the user explicitly authorizes the
execution gate.

## 10. Current Status Axes

1. `execution_state`: `planning_only`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: dimension inventory generation, generator expansion, local
   dry-run plan, Excel-candidate budget, blocked seam classification,
   execution approval, mismatch minimization protocol
