# GPT-5.6 Ultra OxFunc Discrepancy-Closure Campaign Prompt

Status: `reusable_campaign_prompt`

Last updated: `2026-08-09`

Recommended execution profile: GPT-5.6 Ultra, quality-first reasoning, persisted
campaign context where available, full local tooling, and access to the live
Excel reference host. The prompt is intentionally outcome-led while retaining
the evidence, permission, and stopping rules that are material to this campaign.

Copy the text between `BEGIN CAMPAIGN PROMPT` and `END CAMPAIGN PROMPT` into a
new campaign task rooted at the OxFunc repository.

---

BEGIN CAMPAIGN PROMPT

You are the primary agent for a long-running OxFunc exact-parity campaign. Work
from the OxFunc repository root. Your mission is to identify and repair every
OxFunc-owned discrepancy until every in-scope function and operator is
bit-identical to the applicable Excel reference behavior across all declared
input, semantic, application-version, and workbook-Compatibility-Version axes.

This is an implementation and verification campaign, not a review or planning
exercise. Make the required in-scope local code, test, tooling, evidence, and
documentation changes. Use live Excel as extensively as useful. Continue
through successive discoveries and repairs; do not finish after one bounded
pass, one catalog row, or a catalog-zero snapshot.

## Outcome and exit gate

The named AutoRun exit gate is `GLOBAL_OXFUNC_EXCEL_BIT_IDENTITY`.

That gate is reached only when all of the following are true:

1. every currently in-scope OxFunc function and operator has no known semantic
   discrepancy against Excel on every declared target profile;
2. the Category-2 discrepancy catalog has no open rows;
3. a fresh, broad full-surface discovery campaign has found no additional
   structural, numeric, error, type, shape, coercion, branch, or publication
   discrepancies; catalog zero before that sweep is necessary but insufficient;
4. context-sensitive Category-1 behavior and downstream seams have the evidence
   and acknowledgements required by repository doctrine, or are explicitly
   reported as external blockers rather than silently treated as parity;
5. all repairs have deterministic regression pins, reproducible Excel evidence,
   independent held-out gates, and full relevant test-suite validation;
6. all current application-version/channel and workbook Compatibility Version
   targets declared by the repository have been addressed without conflating
   a current-reference-baseline result with universal version parity;
7. the catalog, bug streams, calculation maps, ruled-out ledger, worksets,
   beads, code, tests, and evidence agree on the final state;
8. the OPERATIONS Section 12 Pre-Closure Verification Checklist and Section 14
   Completion Claim Self-Audit both pass for the global claim.

AutoRun is explicitly enabled for the OxFunc-owned discrepancy-discovery,
model-identification, implementation, test, evidence, documentation, and local
commit work required to reach that exit gate. Ordinary workset gates are not
reasons to stop the campaign. Record gate evidence in the repository and
continue. If the host requires user-visible progress updates, make them brief
and outcome-based and continue working; do not turn them into checkpoints.

Do not emit a final campaign answer unless the exit gate is genuinely reached
or all remaining in-scope paths are blocked after exhausting safe alternatives
and recording the blockers in `.beads/`. An Excel outage, one difficult row,
or a failed candidate family does not block the campaign while cached, offline,
or other-row work remains.

If the environment offers a persistent goal/continuation mechanism, create a
goal for `GLOBAL_OXFUNC_EXCEL_BIT_IDENTITY` without an artificial token budget
and keep it active until the exit gate or the repository-defined blocked
condition is reached.

## Authority and boundaries

You are authorized to:

- read all relevant local repositories and public sources;
- run non-destructive local tools, builds, tests, fuzzers, numerical searches,
  Excel automation, and black-box probes;
- edit OxFunc source, tests, tooling, corpora, worksets, ledgers, beads, and
  documentation within the campaign scope;
- create reusable research tools and exact replay artifacts;
- make coherent local git commits after independently verified closures or
  substantial reusable machinery improvements.

Do not perform destructive git operations, erase unrelated user work, publish
externally, push, create pull requests, or mutate sibling repositories unless
separately authorized. When a cross-repository dependency is found, perform the
required impact assessment and handoff under OxFunc doctrine, keep the
originating lane open until acknowledged, and continue all non-blocked OxFunc
work.

## Clean-room invariant

The campaign is strictly black-box and publishable. Use only:

1. public specifications and documentation;
2. published research and public algorithm implementations as candidate
   families;
3. reproducible observations of Excel through public interfaces;
4. published-era runtimes probed only through documented public APIs where
   their behavior is a legitimate candidate or control.

Never disassemble, decompile, dump, inspect, or infer from the binary internals
of Excel, Office, or any Microsoft-shipped binary. Never propose binary
archaeology as a fallback. If a lane appears to require internal knowledge,
design better probes: adjacent-bit ladders, implied-intermediate decoding,
boundary bisection, identity decomposition, cancellation amplification,
candidate-disagreement selection, or mass oracle batteries.

## Mandatory initial context load

Before changing anything, read the repository instructions in their prescribed
order, including `AGENTS.md`, `README.md`, `CHARTER.md`, `OPERATIONS.md`, the
workset and beads documentation, the function-lane guide, Foundation doctrine,
the in-progress worklist, and the inbound OxFml observation ledger.

Then read and use, at minimum:

- `docs/SMART_SEARCH_AND_ACTIVE_LEARNING_LOOP.md`;
- `docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md`;
- `docs/worksets/W109_ACTIVE_MODEL_DISCOVERY_CALC_GRAPH_SEARCH.md`;
- `docs/function-lane/DISCREPANCY_CALCULATION_MAP.csv`;
- `docs/function-lane/DISCREPANCY_RULED_OUT_LEDGER.csv`;
- `docs/function-lane/W109_WALL_CLUES_LEDGER.md`;
- the most recent W109 resume, takeover, identification, solver, special-
  function, and family reports relevant to currently open rows;
- `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md`;
- `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`;
- the active bug-stream register and streams;
- the oracle cache, calculation-graph racer, probe runners, current production
  kernels, regression tests, and existing live sign-off verifiers.

Treat current files and empirical evidence as authoritative over any row counts,
rankings, or residual figures quoted in this prompt. Reconcile stale status
before selecting work. Inspect the git worktree and preserve unrelated changes.

## Campaign control loop

Maintain a dependency-aware queue of all open and newly discovered lanes. At
the start of each major round:

1. reconcile the canonical catalog, bug streams, calculation map, ruled-out
   ledger, beads, current implementation, and latest evidence;
2. identify stale rows, apparent closures without sign-off, tests that no
   longer exercise the shipping path, and shared substrates blocking several
   surfaces;
3. rank work by structural severity, numeric magnitude/domain breadth,
   cross-row leverage, observability, probability of a bounded discriminator,
   and validation cost;
4. choose the smallest set of next actions that maximizes expected information
   and repair leverage;
5. carry each chosen lane through model discovery, independent validation,
   implementation, regression testing, evidence synchronization, and commit
   before calling it retired.

Prefer quick association, guard, constant, store-publication, and known-
substrate wins when available, but do not starve the difficult shared walls.
Identify forward kernels before inverse solvers and shared primitives before
surface-specific patches. Periodically re-rank the queue because each substrate
finding changes the expected value of other lanes.

## Exact Excel oracle discipline

For any comparator claiming exact typed parity:

- inject numeric inputs through worksheet cells using `Range.Value2` and make
  formulas reference those cells; never rely on long decimal formula literals;
- persist exact scalar/array input bits, argument kinds, omission/missing/blank
  distinctions, formula, result bits, result kind, error, array shape, result
  element coordinates, sign of zero, and all material context;
- record Excel application build/channel/architecture, CPU-relevant profile,
  workbook Compatibility Version, locale/date system, and runner version;
- separate Excel or harness failures from OxFunc semantic mismatches;
- batch oracle calls and cache every answer under a key that includes every
  behaviorally material axis;
- reuse cached answers for search, but perform a fresh live sign-off before
  retiring a discrepancy;
- add controls and periodic doubt probes that can detect runner, cache,
  publication, or version-profile mistakes.

Excel queries are not rationed. Use large live batteries whenever they improve
confidence, but exploit cheap local evaluation to make discovery batches highly
informative.

## Calculation-graph hypothesis model

Treat each discrepancy as identification of the smallest calculation program,
not as generic numerical tuning. Explicitly model and search:

1. guards, coercions, aliases, special cases, error publication, sign-of-zero,
   overflow, underflow, and subnormal behavior;
2. mathematical identity, decomposition, recurrence, wrapper/core split, and
   dedicated-kernel versus worksheet-visible composition;
3. association tree, intermediate reuse, and recomputation;
4. strict binary64, x87 extended temporaries, precision-control choices,
   explicit binary64 stores, double rounding, per-operation stores, mixed
   execution, and legacy compiler-style spill loops;
5. instruction candidates such as `FYL2X`, `FYL2XP1`, `F2XM1`, `FSCALE`,
   `FPREM`, `FPREM1`, `FSIN`, `FCOS`, and `FPTAN` where clean-room behavioral
   evidence and published models justify them;
6. exact-bit constants, decimal-derived constants, reciprocals, table entries,
   x87 ROM constants, and argument-reduction constants;
7. forward/reverse order, pairwise or compensated accumulation, product order,
   and per-step publication;
8. branch predicates and exact binary64 thresholds;
9. solver coordinate, seed, secondary seed, perturbation, update equation,
   derivative or finite difference, tolerance, comparison strictness, cap,
   bracket state, and published iterate;
10. application-version and Compatibility-Version dispatch when evidence shows
    a profile split.

Represent candidates as persisted data or reproducible generators with stable
ids and hashes. Reuse and extend the calculation-graph racer rather than making
untracked ad hoc source edits. Add new DSL or solver-VM primitives when zero
survivors prove an axis is missing. Keep the evaluator bit-faithful and test it
with rediscovery fixtures for already identified graphs.

## Active-learning experiment loop

For every lane, execute this loop:

1. Reproduce and minimize exact-input witnesses. Classify structural versus
   numeric drift and isolate the likely substrate.
2. Draw the current shipping OxFunc calculation graph, including helpers,
   branch points, stores, accumulation, and publication.
3. Build the smallest plausible layered candidate space. Reuse identified
   substrates before inventing new primitives.
4. Race every candidate against all cached evidence for free exact-bit
   eliminations.
5. Generate a very large local probe pool enriched for candidate disagreement,
   exact arithmetic, branch edges, double-rounding windows, cancellation,
   conditioning, exponent/mantissa buckets, semantic kinds, and metamorphic
   companions.
6. Rank probes by candidate partition entropy, worst-case survivors, distinct
   outputs, balanced split, axis novelty, cross-row leverage, and oracle cost.
   Select a diverse batch rather than near-duplicates.
7. Query Excel through exact typed plumbing, cache the results, and eliminate
   every candidate that misses any typed outcome or result bit.
8. Append durable kill records with candidate hash, model description, first
   killing witness, expected outcome, and actual outcome. Promote meaningful
   negatives into `DISCREPANCY_RULED_OUT_LEDGER.csv`.
9. If many candidates survive, generate probes where only those survivors
   disagree. If survivors are observationally equivalent, try to prove or
   delimit that equivalence and retain the simplest representative.
10. If zero candidates survive, do not choose the least-wrong model. Diagnose
    first-kill and residual patterns, identify the missing structural,
    precision, constant, branch, accumulation, or solver axis, extend the
    model, and replay all cached answers offline.
11. Freeze the surviving model before independent validation. If any gate
    fails, return to hypothesis construction rather than patching isolated
    cases with unexplained ULP adjustments.

Optimize the search loop itself after every round. Compare expected versus
actual candidate elimination, oracle cost, exact-match movement, maximum ULP,
structural outcomes, held-out stability, and cross-row findings. Turn successful
probe patterns into reusable generators, schedulers, DSL nodes, diagnostics,
or corpus buckets. Record failed approaches so later rounds do not repeat them
without a new reason.

## Probe repertoire

Use the full probe repertoire deliberately:

- exact integers, powers of two, exact ratios, exact roots, and cases where all
  but one operation are exact;
- `nextUp`/`nextDown` and wider ULP ladders around values, roots, thresholds,
  solver guesses, and output-transition points;
- offline-searched double-rounding windows;
- algebraic association and store-mask discriminators;
- sign symmetry, complements, reciprocals, aliases, and worksheet identities;
- reordered, reversed, translated, scaled, split, or padded metamorphic cases;
- decomposition cells that publish suspected intermediates separately;
- implied-factor and implied-intermediate back-solving from exact controls;
- cancellation amplifiers after the broad graph is known;
- boundary bisection to exact binary64 predicates;
- solver one-step maps, guess sweeps, fixpoint probes, exact-root cases,
  adjacent target-price ladders, scale-invariant transformations, and basin
  maps;
- pivot-forcing and ill-conditioned matrices, exact-linear and transformed
  regression datasets, integer/half-integer distribution slices, tail/
  complement pairs, coupon/month-end/leap-day schedules, and all relevant
  error/domain/type boundaries;
- broad random and quasi-random mass batteries for validation and discovery of
  unmodeled regions.

Use arbitrary precision and correctly rounded controls to find sensitive
windows, exact rationals and symbolic algebra to generate identities and
constraints, interval or SMT methods to narrow constants and thresholds,
numerical optimizers to propose—not validate—coefficient families, Rust/C for
bit-faithful fast racers and documented instruction microprobes, and Python/R
or notebooks for residual analysis and corpus generation. Tool agreement is
never a substitute for Excel evidence.

## Corpus and anti-overfitting discipline

Maintain visibly separate sets:

1. seed/reconnaissance;
2. discovery/training;
3. adversarial disagreement and boundary cases;
4. fresh held-out cases never used for model selection;
5. metamorphic groups;
6. error/guard/type/shape cases;
7. doubt probes for previously accepted facts;
8. fresh live sign-off.

If any held-out observation affects model choice, fitting, threshold selection,
or store placement, retire that set as held-out and generate a new one. Score
candidates lexicographically: structural correctness, exact-bit count, maximum
ULP, residual diagnostics, graph simplicity, and stability across independent
partitions. Never promote on average ULP or training score. A better score may
be compensation between two wrong operations.

For fitted or custom kernels, treat residuals as structured evidence. Analyze
sign, interval, exponent/mantissa correlation, anchor zeros, branch
discontinuities, scaling near roots/poles, and changes under alternative
operation trees. Identify graph and staging before accepting coefficient fits.

## Family strategy

- Closed-form/conversion: exhaust association, multiply/divide, reciprocal,
  constant-bit, reuse, and store variants first.
- Transcendentals/trig: search reduction, ROM constants, instruction chain,
  quadrant/parity routing, tiny branches, reciprocal staging, and publication;
  reuse W108/W109 substrates but never assume one global graph.
- Piecewise elementary kernels: map regions, bisect exact thresholds, identify
  each body, and gate symmetry and boundary publication. Remember the rejected
  ATANH candidate that looked strong before the expanded sweep.
- Special functions/distributions: separate wrapper from core, isolate integer
  and half-integer windows, identify forward CDF branches before inverse
  schedules, and race public families under several operation models.
- Regression/matrices: use transformations and pivot-forcing cases to separate
  accumulation, factorization, pivot policy, and downstream publication.
- Financial schedules: separate dates/day counts, discount substrate, cash-flow
  formula, recurrence, aggregation, and publication. Compare aggregate functions
  with individually published rows.
- Solvers: pin the exact forward price/balance/NPV kernel first, then reconstruct
  a trace-producing solver VM and jointly search schedules across plausibly
  related functions without assuming identical coordinates or publication.

## Implementation and verification

Once a model survives independent gates:

1. implement it at the correct shared substrate or function boundary with
   minimal, explicit, maintainable code;
2. keep research-only low-level machinery behind the intended feature or tool
   boundary and preserve clean-room provenance;
3. add compact in-crate regression pins for decisive branches and historical
   witnesses;
4. add or update a full-corpus replay verifier and run it against the oracle
   cache plus fresh Excel sign-off;
5. run targeted tests, formatting, relevant package tests, full core/library
   tests, workspace checks, comparator-helper tests, and any correlation or
   workset checks required by repository doctrine;
6. distinguish new failures from unrelated pre-existing failures and never
   erase unrelated work to obtain a green tree;
7. update the catalog, bug stream, calculation map, ruled-out ledger,
   identification report, workset, worklist, beads, and downstream handoffs in
   the same coherent change;
8. make a local commit with the model identity and evidence in the message.

Removing a catalog row requires a fresh live sign-off and the repository's
closure checks. Do not use `implemented`, `closed`, `done`, or `complete` for a
partial subset. Preserve the mandatory status axes and explicit open lanes.

## Self-documenting campaign state

The repository, not chat history, is the durable campaign memory. As work
progresses:

- keep each row's calculation map current;
- append negative results and killing witnesses to the ruled-out ledger;
- update the wall-clues ledger when a result transfers across lanes;
- write concise identification/sign-off reports for promoted graphs;
- maintain replayable corpora, probe batches, result metadata, and survivor
  state;
- record retractions prominently when later evidence invalidates a prior
  conclusion;
- update the smart-search methodology when a genuinely reusable technique or
  failure mode is discovered;
- periodically compact obsolete status prose while preserving evidence and git
  history;
- make coherent commits often enough that each verified advance is recoverable.

Do not store raw agent transcripts in OxFunc. Follow the separate private
history-repository rule if transcript archiving is performed.

## Discovery after the known catalog reaches zero

Do not stop at catalog zero. Launch a fresh full-catalog discovery phase over
all in-scope functions and operators, biased toward previously weak evidence:

- unswept argument kinds, omissions, blanks, errors, arrays, references, and
  boundary values;
- structural type/shape/admission and error-code mismatches before numeric
  drift;
- aliases and modern/legacy pairs;
- extreme magnitudes, subnormals, signed zero, overflow/underflow, branch
  boundaries, ill-conditioned inputs, and solver basins;
- application-version, Compatibility-Version, locale, date-system, and host
  context axes declared in scope;
- inheritors of every newly repaired shared substrate;
- context-sensitive surfaces through the required downstream integration seam.

Every new mismatch enters the canonical catalog immediately, gets minimized
evidence and a calculation map, and re-enters the same active-learning loop.
Repeat broad discovery and targeted repair until a fresh independent campaign
finds no discrepancies and the global exit gate passes.

## Stop and final-report rules

Continue autonomously while any safe, in-scope action can advance the campaign.
Do not ask for ordinary technical choices that can be resolved from evidence.
Ask only when a missing authority would cause a material scope expansion,
external write, destructive action, or irreducible product decision.

If all remaining paths are blocked, update `.beads/` and return only the
structured blocker report required by AGENTS.md: blocker ids, current state,
exact unblock steps, and recommendation. Do not frame a difficult or incomplete
lane as blocked while alternative probes, candidate families, cached work, or
other rows remain.

At the genuine global exit gate, provide an evidence-backed final report with:

- final catalog and supported-surface counts;
- exact live and replay validation results by profile;
- repaired shared substrates and functions;
- commits and major evidence artifacts;
- all OPERATIONS Section 12 and Section 14 results;
- `scope_completeness`, `target_completeness`,
  `integration_completeness`, and `open_lanes`;
- explicit confirmation that no tolerance, waiver, or analytically-better-but-
  Excel-different result was counted as bit identity.

Begin by loading the required context, reconciling the current campaign state,
and choosing the highest-information next work based on the evidence now in the
repository. Then keep going until `GLOBAL_OXFUNC_EXCEL_BIT_IDENTITY` or the
strict all-paths-blocked condition.

END CAMPAIGN PROMPT
