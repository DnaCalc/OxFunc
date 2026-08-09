# W109 — Active Model-Discovery: Calculation-Graph Search Over Excel Oracle Bits

Status: `active`
Started: `2026-07-11`
Prereq: W108 (x87 transcendental core), TBILLYIELD association closure.

## 1. Doctrine

The remaining catalog rows are **model-identification problems, not precision
bugs**. Excel is an unlimited black-box oracle; the objective per row is to
identify the smallest calculation graph — structure, association, store
barriers, constants, branch thresholds, iteration schedule — that predicts
Excel's exact result bits on independent inputs. Consequences:

1. Never reward average accuracy against a correctly-rounded reference; the
   target is Excel's bits (which are sometimes deliberately less accurate).
2. Random inputs validate; **constructed inputs discover**. Prefer probes that
   isolate one staging choice (exact-arithmetic controls, double-rounding
   windows, metamorphic transforms, identity decompositions, intermediate
   isolation).
3. Ask Excel the fewest, highest-information questions: search offline for
   inputs where surviving candidates disagree, ask only those.
4. Negative results are load-bearing: every killed candidate goes to the
   [ruled-out ledger](../function-lane/DISCREPANCY_RULED_OUT_LEDGER.csv).
5. Promotion needs the full closure discipline: discovery + adversarial +
   held-out (never searched) + metamorphic sets, all 100% bit-exact, plus a
   live sign-off sweep. (The ATANH 297/368 reversal is the failure mode this
   prevents.)

## 2. The three-part system (built, all green)

| Piece | Where | What |
|---|---|---|
| Oracle cache | `smart-fuzzer/tools/OracleCache.psm1` (+ `Test-OracleCacheHelpers.ps1`, 38 checks) | Persistent JSONL cache keyed by exact request bits and pinned to an `oracle-environment-v2` manifest (Excel version/build/bitness, workbook Compatibility Version, and CPU). Cached runs validate the live host before serving any shard; deliberate `-NoCache` fresh sign-off is separate. Shards remain under `smart-fuzzer/cache/oracle/build-<n>/`. |
| Calculation-graph racer | `smart-fuzzer/tools/calc_graph_racer/` | Serde-JSON candidate DSL (per-node eval model: strict binary64 / x87 extended with CW + store barrier; ROM constants; branches; fold-sums incl. the legacy per-step-stored x87 spill loop), bit-faithful evaluator, lexicographic scoring, association/store-mask/model enumerators. |
| Experiment scheduler | `calc_graph_racer` subcommands `race` / `distinguish` / `eliminate` + `smart-fuzzer/tools/Run-W109ProbeBatch.ps1` (+ `Test-W109SchedulerLoop.ps1`) | Surviving-candidate state per row under `smart-fuzzer/work/w109/<row_id>/`, offline disagreement-maximizing probe ranking, elimination on exact bits, kill records. |

Substrate: `oxfunc_core` feature `research-x87` exposes
`excel_numeric::research` — raw x87 instructions (`FSIN`, `FCOS`, `FPTAN`,
`FPREM`/`FPREM1` with completion loop, `FYL2X`, `FYL2XP1`, `F2XM1`, `FSCALE`,
ROM constants) and the `Ext80` extended-precision temporary whose memory
round-trip is bit-transparent (proven by recomposing the full `fFEXP`/`fFLN`
chains from raw ops against the entire live-Excel witness corpus). All asm
stays in `excel_numeric/x87.rs`.

**Racer acceptance fixtures** (CI, no Excel): from witnesses alone the racer
must rediscover (a) TBILLYIELD's `((100-p)/p)*(360/days)` association among
all 14 trees, and (b) POWER's x87 reciprocal/sqrt staging against 4 rivals,
including *searched* double-rounding discriminators
(`tests/rediscover_tbillyield.rs`, `tests/rediscover_power.rs`).

## 3. Phase 1 result — XNPV pilot (CLOSED, G6-11)

Full loop exercised end-to-end; see
[W109_XNPV_IDENTIFICATION_20260711.md](../function-lane/W109_XNPV_IDENTIFICATION_20260711.md).
Headlines:

- 480-candidate space; 2 free recon witnesses killed 24/36 of the first cut;
  live round 1 killed **everything**, exposing two missing axes — the
  integer-exponent POWER dispatch and per-step-stored x87 accumulation;
  72 constructed window probes then separated base/term/sum staging; the two
  final survivors differ only on a years axis proven observationally
  equivalent by exhausting the whole integer date domain.
- Identified graph: `rate<=0 -> #NUM!` (new guard finding; OxFunc previously
  accepted `(-1,0]`); `base=RN53(RN64(1+rate))`; full worksheet POWER kernel
  per term; `term=RN53(RN64(v/pow))`; forward per-step-stored x87 sum.
  **XNPV's body is legacy x87-compiled code** — every assignment double-rounds
  through a memory spill. This generalizes W108 beyond transcendentals.
- Sign-off: 1530/1530 numeric + 175/175 error rows
  (`verify_xnpv_promotion`); in-crate pins; workspace green; catalog row
  removed; ruled-out ledger updated.

Parallel deliverable: the W103/W108 Phase-E annuity corpus (5,154 live rows)
replayed through current OxFunc
(`pmt_ppmt_local_eval --bin phase_e_replay`, rollup under
`smart-fuzzer/runs/w109-phase-e-replay/`): 3117 exact / 1318 at 1 ULP /
719 above, incl. sign-of-zero branch bugs on IPMT/ISO-EM rows — the true
G6-01 surface for Phase 2/6 work.

## 4. Roadmap (dependency-ordered)

1. **Phase 2 — closed-form quick wins**: YIELDMAT, NPER (FYL2XP1 variants),
   CONVERT (solve implied factor bits), ACOTH, COMBIN/COMBINA
   family (x87 continuous product), CUMPRINC half-schedule, GAUSS/PHI.
   Candidate prior after the pilot: *legacy spill-loop double-rounded
   arithmetic + confirmed x87 transcendental kernels*.
2. **Phase 3 — trig reduction (G4-01)**: `FSIN`/`FCOS`/`FPTAN` with
   `FPREM1`-style reduction (raw primitives already exposed); fitted-π
   confirmation from large-argument residuals; COT/SEC/CSC reciprocal
   staging; then replay GAMMA reflection (G3-02) and Bessel inheritors.
3. **Phase 4 — solver reconstruction (YIELD, ODDFYIELD, RATE, IRR)**: extend
   the DSL with iterate-nodes (method x seed x perturbation x tolerance x
   publication rule), exhaustive schedule grid scored jointly across all four
   functions; guess-sweeps, exact-root witnesses, residual fingerprints,
   back-solved iterate chains. Kernels first (Phases 1-3), schedules second.
4. **Phase 5 — statistical substrate**: GAMMALN kernel, incomplete
   beta/gamma continued fractions, legacy-vs-modern pair differencing,
   fixed-statistic probes for the tests, metamorphic identification for the
   regression family. The MINVERSE distinguishing-matrix race is now signed
   off separately in Section 7.6.
5. **Phase 6 — ATANH and ACOTH signed off independently.** The original
   368-case reversal triggered a dense 5,902-row map, exact live boundary
   bisection, retired refinement set, and fresh post-selection held-out. The
   exact ATANH graph landed in `a03a75f`. ACOTH did not inherit its former
   small-input helper: its separate answer-blind campaign identified a direct
   inverse odd-power series, exact threshold `0x400d92b14ec204f3`, and +0
   reciprocal flush, with a frozen `66552/66552` held-out and
   `268769/268769` production replay; the package landed in `7f7eac9`.
   Continue a full catalog re-sweep after
   each primitive lands.

## 5. Operating procedure per row

```
gen candidate space (bin or hand JSON)  -> work/w109/<row>/candidates.json
race against known witnesses            -> free eliminations
loop:
  calc_graph_racer distinguish          -> batch.json   (offline)
  Run-W109ProbeBatch.ps1                -> answers.json (live, cached)
  calc_graph_racer eliminate            -> survivors + kill records
  (0 survivors => the space is missing an axis: diagnose the kill bits,
   extend the DSL/space, re-eliminate offline against all cached answers)
until 1 survivor (or a proven-equivalent set)
validate: held-out + metamorphic sweeps, 100% bit-exact
promote: production kernel + in-crate pins + full-corpus replay verifier
close:   catalog row out, ruled-out ledger rows in, evidence doc
```

## 6. Pointers

- Durable method: [SMART_SEARCH_AND_ACTIVE_LEARNING_LOOP.md](../SMART_SEARCH_AND_ACTIVE_LEARNING_LOOP.md)
- Campaign prompt: [GPT56_ULTRA_DISCREPANCY_CLOSURE_CAMPAIGN_PROMPT.md](../GPT56_ULTRA_DISCREPANCY_CLOSURE_CAMPAIGN_PROMPT.md)
- Catalog: [OXFUNC_EXCEL_DISCREPANCY_CATALOG.md](../OXFUNC_EXCEL_DISCREPANCY_CATALOG.md)
- Calc map (seed hypotheses): [DISCREPANCY_CALCULATION_MAP.csv](../function-lane/DISCREPANCY_CALCULATION_MAP.csv)
- Ruled-out ledger: [DISCREPANCY_RULED_OUT_LEDGER.csv](../function-lane/DISCREPANCY_RULED_OUT_LEDGER.csv)
- W108 (x87 numeric core): [W108_EXCEL_NUMERIC_CORE_AND_FINANCIAL_POWER_EXACTNESS.md](W108_EXCEL_NUMERIC_CORE_AND_FINANCIAL_POWER_EXACTNESS.md)
- PMT current intermediate/timing evidence: [W109_PMT_INTERMEDIATE_AND_TIMING_DISCRIMINATION_20260809.md](../function-lane/W109_PMT_INTERMEDIATE_AND_TIMING_DISCRIMINATION_20260809.md)
- GROWTH/LOGEST single-predictor checkpoint: [W109_GROWTH_LOGEST_SINGLE_PREDICTOR_DISCOVERY_20260809.md](../function-lane/W109_GROWTH_LOGEST_SINGLE_PREDICTOR_DISCOVERY_20260809.md)
- CUMPRINC exact-graph partial report: `smart-fuzzer/tools/calc_graph_racer/CUMPRINC_EXACT_PARTIAL_REPORT_20260809.md`
- RATE exact-graph partial report: `smart-fuzzer/tools/calc_graph_racer/RATE_EXACT_GRAPH_PARTIAL_REPORT_20260809.md`
- PRICE/DURATION residual-graph partial report: `smart-fuzzer/tools/calc_graph_racer/PRICE_DURATION_RESIDUAL_GRAPH_SCOPE_PARTIAL_20260809.md`
- IRR exact-graph discovery checkpoint: [W109_IRR_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md](../function-lane/W109_IRR_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md)

## 7. 2026-08-09 reconciliation and active checkpoint

This checkpoint supersedes stale row counts and any earlier interpretation that
the PMT residual had been proved irreducible.

### 7.1 Oracle and cache provenance

1. The current live host is Excel `16.0`, build `20228`, 64-bit, workbook
   Compatibility Version `2`, on the recorded x86-64 Windows/CPU profile.
2. The pre-checkpoint default cache is pinned to build `20131`. It must not be
   used as fresh sign-off evidence for build `20228`, and cached execution must
   validate the invoker's material environment before serving answers.
3. Fresh promotion evidence uses `-NoCache` or a new build-specific cache root.
   Answer artifacts must embed capture provenance (Excel version/build/bitness,
   Compatibility Version, OS/CPU/input plumbing, cache mode, runner version,
   PowerShell version, and capture time) rather than relying on terminal history.

### 7.2 Landed financial graph repairs

The following three bug slices passed provenance-bearing recapture, focused and
full tests, formal alignment, and evidence synchronization. Their implementation
and evidence landed in `876635e`; BUG-FUNC-043/044/045 are
`closed_signed_off`, beads `oxf-jwh5.1/.2/.4` are closed, and catalog rows
G6-12/G6-13/G6-14 are retired. This scoped retirement does not close the wider
financial family or W109 campaign:

1. `EFFECT`: x87-double-rounded base construction; for truncated periods below
   `u32::MAX`, LSB-first integer binary exponentiation where every accumulator
   multiply and squaring is `RN53(RN64(a*b))`; at and above `u32::MAX`, the raw
   stored-LN/product x87 EXP chain; then x87-double-rounded subtraction. Current
   evidence is `315/315` banked + `870/870` fresh held-out + `4/4` targeted
   base-add + `160/160` fresh extreme-domain/dispatch outcomes.
2. `RRI`: reject periods below `MIN_NORMAL`; DAZ-normalize value inputs and
   return `+0` on equality before sign guards; DAZ the x87-double-rounded
   quotient and return `-1` on zero; use the quotient directly when
   `periods==1`; otherwise use the x87 reciprocal and raw stored `LN` ->
   reciprocal-first double-rounded product -> `EXP` chain, followed by
   x87-double-rounded subtraction. This deliberately bypasses worksheet POWER's
   special dispatch. Current evidence is `154/154` banked + `4900/4900` fresh
   held-out + `375/375` follow-up + `6/6` wrapper staging + `60/60` edge-domain
   + `35/35` blind disagreement + `6/6` exact-period rows = `5536/5536`.
3. `NOMINAL`: after truncation and x87-double-rounded `1+effect`, periods `1`
   and `2` use a register-continuous `FYL2X`/`F2XM1`/`FSCALE` power; periods
   `>=3` use the raw stored power chain. Both routes store the power before
   `n*(power-1)`. Current evidence is `242/242` adjacent + `600/600` follow-up
   + `2/2` branch-pair + `8/8` wrapper staging.

### 7.3 ACCRINT G6-02 final publication repair

ACCRINT's July schedule identification was correct but its final plain-f64
publication left 13 one-ULP rows. The exact graph stores
`coupon=(par*rate)/frequency` and the identified accrual fraction `a` in
ordinary binary64, then publishes only through
`excel_x87_mul(coupon,a) = RN53(RN64(coupon*a))`. The repair and exact pins
landed in `cd1f9fe`.

Production replay is b39 `25410/25410`, b40 `51420/51420`, b42
`68790/68790`, recaptured build-20228/CV2 NoCache b43 `780/780`, and a fresh
frozen held-out `450/450`, totaling `146850/146850`. The current-reference
answer hashes and provenance are recorded in the W109 bond report and
BUG-FUNC-030 stream. BUG-FUNC-030 is `closed_signed_off`; the already-closed
bead `oxf-bx1u` remains closed with a successor-evidence comment; G6-02 is
retired. No FEC/F3E or evaluator-facing handoff is required. This closes only
G6-02 and does not close the wider bond/financial family or W109 campaign.

### 7.4 COS/BESSELJ signed off; PMT remains open

1. The corrected worksheet-COS graph retains the tiny exact-one guard and
   FPREM1 reduction, keeps FCOS on even quadrants, and reconstructs the signed
   odd-quadrant sine magnitude through continuous-PC64 FPTAN square/ratio/FSQRT.
   It is `1027/1027` on discovery, `1020/1020` on prior validation, and
   `514/514` on a frozen oracle-blind held-out: `2561/2561` total selected
   evidence, with the original 24-row threshold ladder retained separately.
2. BESSELJ routes both J0/J1 asymptotic cosine sites through corrected COS and
   stages only J0 `cosine*p` through `excel_x87_mul`. The landed production
   kernel is `794/794`; the repair, pins, and tooling landed in `ed9f222`.
   BUG-FUNC-046/047 and beads `oxf-jwh5.5/.5.1` are closed signed off, and
   G4-06/G4-07 are retired. No FEC/F3E or evaluator-facing handoff is required.
3. The July-25 PMT takeover brief explicitly retracts the July-24
   "proven irreducible / needs provenance" framing. The only defensible result
   is bounded-negative over the documented leaf/operator/size limits. A
   reproducing Excel program exists; larger graphs, coefficient recovery, and
   residual wrapper/predicate axes remain actionable. The 2026-08-09
   current-build hardening adds three narrower results without closing G6-01:
   the x87-spill helper representative is `226/324`; direct hidden-low-word
   delivery is hard-refuted, `51/60` tested smooth interval systems have exact
   infeasibility certificates, and nine retain numerical negative evidence; the
   power-of-two timing reciprocal identity is `832/832` but does not generalize
   to the frozen general-rate gate, whose best subtractive family is only
   `378/480` and exactly explains `1/15` contexts. The unresolved type-1
   helper/association prevents a global tail-order claim. EXT6 is also
   incomplete: its durable log stops at shard `191/400`. Canonical evidence is
   in `W109_PMT_INTERMEDIATE_AND_TIMING_DISCRIMINATION_20260809.md`.

### 7.5 ATANH G4-02 exact graph signed off

The sparse July x87-ln1p-pair interpretation is retracted. Current-reference
Excel uses three routes: `|x|>=1` publishes `#NUM!`; both subnormal signs publish
positive zero under DAZ; normal inputs below exact threshold
`0x3f1af82b729c1d83` use ordinary binary64 `x+(x*x*x)/3`; inputs at/above the
threshold form the signed ratio through x87-double-rounded add, subtract, and
divide stores before the established worksheet-LN publication.

Evidence is `5902/5902` dense discovery, a 43-step adjacent-double live
bisection, a 7,050-row set explicitly retired into refinement, `8510/8510` on a
fresh post-selection held-out, and `20780/20780` on the durable combined replay.
The implementation, exact pins, and reusable generators/racer landed in
`a03a75f`; focused tests and the full core passed, and Lean carries the route
binding without duplicating x87 arithmetic. BUG-FUNC-027 CLASS-C4 is
`closed_signed_off`, bead `oxf-jwh5.6` is closed, and G4-02 is retired. Other
BUG-FUNC-027 subclasses and the wider W109 campaign remain open. No
FEC/F3E or evaluator-facing handoff is required.

### 7.6 MINVERSE G5-01 exact graph signed off

The July Doolittle structure was correct, but its “plain SSE2 double” claim is
retracted. The negative experiment tested continuously retained x87 regions;
it did not test the legacy per-operation `RN53(RN64(op))` publication pattern.
An exhaustive eight-site mask race now identifies x87 double rounding at LU
factor division, elimination multiply/subtract, forward and backward solve
multiply/subtract, and final division. Completed numeric zero cells publish as
positive zero.

The first `576`-row targeted set changed the model and is explicitly retired
into refinement. A second bank/refinement-disjoint gate froze `32`
discriminators per arithmetic site, `64` signed-zero rows, and `96` controls;
fresh build-20228/CV2 matrix Value2 NoCache capture selected the graph
`416/416`, uniquely across all 256 masks. The landed production surface replays
`607/607 + 576/576 + 416/416 = 1599/1599`. Implementation, exact pins, and
deterministic tooling landed in `bce3558`; full core passes `1521` tests with
`4` ignored, and the 492-job Lean build records the route binding.
BUG-FUNC-025 and bead `oxf-dzfk` are closed signed off; G5-01 is retired. The
separate `MINVERSE(5)` final-cell publication seam remains in parent
BUG-FUNC-023 / HO-FN-010, so this scoped closure does not close that parent or
the wider campaign. No new FEC/F3E handoff is required.

### 7.7 ACOTH G4-03 exact graph signed off

The sparse July reciprocal-FYL2XP1-pair interpretation is retracted. The exact
current-reference graph computes on `abs(x)` and restores sign only for
nonzero results. Below `0x400d92b14ec204f3`, native binary64 `a+1` and `a-1`
feed a stored x87-PC64 division and worksheet LN/half publication. At and above
the threshold, Excel uses the direct inverse odd-power series with each
reciprocal, multiply, divide, and accumulator add stored through x87 PC64 to
binary64. If the initial reciprocal is subnormal, both input signs publish
positive zero.

The last ratio-only discriminator is `0x400d92b14ec204ef`; the first
series-only discriminator is the threshold, and the three intervening doubles
are observational overlap. The candidate scores `202217/202217` on the
discovery bank. It was then frozen before a deterministic prior-disjoint
`66552`-row held-out, which passed `66552/66552` with zero anomalies and no
model refinement. The frozen scorer and actual production `acoth_kernel` both
replay `268769/268769` distinct signed inputs exactly.
The production, formal, pin, and durable-tool package landed in `7f7eac9`.

Focused ACOTH tests pass `7/7`; the full core library passes `1523` with `4`
ignored and all integration/doc-test targets green; all seven reusable ACOTH
racer/generator binaries release-check clean; and the 492-job Lean build
records the positive-zero/ratio/series route order. BUG-FUNC-027 CLASS-C5 and
bead `oxf-jwh5.7` are closed signed off, and G4-03 is retired. Evidence,
process-count provenance, hashes, and the scoped Sections 12/14 audit are in
`docs/function-lane/W109_ACOTH_IDENTIFICATION_20260809.md`. Other
BUG-FUNC-027 subclasses and W109 remain open. No FEC/F3E or evaluator-facing
handoff is required.

CONVERT's exact current-reference graph landed in `8ef5cac`. Every supported
linear route publishes `number*from_factor`, the resulting quotient by the
to-factor, and a separate decimal-prefix delta multiply through x87 PC64-to-
binary64 double rounding. Length uses exact integer angstrom factors; pressure
uses independently rounded reciprocals of the public units-per-pascal table;
temperature uses direct ordered-pair affine formulas; and unsupported `bar`
returns `#N/A`. The graph is exact on the `7026`-row discovery bank, both
explicitly retired refinement attempts, the `4226`-row v3 refinement battery,
an independent Value2/readback control, and a candidate-frozen prior-disjoint
`10418/10418` publication gate. Compiled production replay is `34189/34189`.
Focused/full core and the 492-job Lean route-binding gates pass, G4-05 is
retired, bead `oxf-jwh5.8` is closed signed off, and no FEC/F3E or
evaluator-facing handoff is required. Evidence,
hashes, retired-gate discipline, and the scoped Sections 12/14 audit are in
`docs/function-lane/W109_CONVERT_IDENTIFICATION_20260809.md`.

The COMBIN sublane of mixed G4-04 is also exact on the current reference.
After truncation and `k=min(k,n-k)`, the landed `c879f3f` graph walks factors
cyclically from `i=2`: it stores `(n-k+i-1)/i` through x87 PC64-to-binary64,
stores each accumulator multiply the same way, and multiplies by `n` only at
publication. It replays `505/505` legacy, `20713/20713` current discovery, and
a candidate-frozen prior-disjoint `1024/1024` publication gate, for
`22242/22242` total. Focused/full core, tracked production replay, and the
492-job Lean route binding pass; bead `oxf-jwh5.9` is closed signed off. The
later COMBINA paragraph below supersedes this original gate's open-COMBINA
state; G4-04 now remains open only for ERF/ERFC.PRECISE, and BUG-FUNC-027
remains open for those and unrelated subclasses. Evidence, hashes, rejected controls, and
the scoped Sections 12/14 audit are in
`docs/function-lane/W109_COMBIN_IDENTIFICATION_20260809.md`. No FEC/F3E or
evaluator-facing handoff is required.

The subsequent COMBINA campaign retracted the July GAMMALN/product-
impossibility claim and identified the exact transformed-COMBIN route. COMBINA
applies DAZ and separately truncates both arguments, publishes one from its
zero/zero pool before the asymmetric truncated-n/raw-DAZ-k negative guard, and
then delegates `tn+tk-1,tk` to COMBIN. Paired boundary discovery also corrected
COMBIN admission: DAZ precedes its raw negative guard and truncated `n` is
admitted only through `2_147_483_646`; the cyclic body is unchanged and may
short-circuit a monotone nonfinite accumulator to `#NUM!`.

Production replays `40,330/40,330` COMBINA rows, `2,195/2,195` new COMBIN
admission controls, and the original `22,242/22,242` COMBIN corpora, for
`64,767/64,767`. The central COMBINA gate is a candidate-frozen,
prior-disjoint `2,048/2,048`; the genuinely fresh fractional-ceiling/DAZ gate
passes COMBIN `76/76` and COMBINA `144/144` without refinement. Focused/full
core and the 492-job Lean build pass; the implementation landed in `3f31f44`.
Scoped bead `oxf-jwh5.11` is closed
signed off; G4-04 remains open only for ERF/ERFC.PRECISE, and
BUG-FUNC-027/W109 remain open for those and unrelated subclasses. Evidence,
hashes, retired-v1 discipline, and the scoped Sections 12/14 audit are in
`docs/function-lane/W109_COMBINA_IDENTIFICATION_20260809.md`. No FEC/F3E or
evaluator-facing handoff is required.

### 7.8 GROWTH/LOGEST and CUMPRINC bounded discovery checkpoints

Two additional current-build investigations produced durable negative evidence
without reaching a promotion gate.

For the single-predictor `LOGEST`/`GROWTH` lane, two serialized build-20228/CV2
NoCache rounds cover 360 `LOGEST` calls and 1,260 `GROWTH` calls. The former
two-control claim that `GROWTH` universally publishes the observed `LOGEST`
base times factor-to-x is withdrawn: 240 of 23,328 graphs fit those two rows,
while the best current reconstruction from observed LOGEST cells is only
`666/1240` numeric exact. A one-final-EXP graph also misses 18/20 structural
`#NUM!` outcomes; separately published coefficients followed by POWER/product
match 20/20 structural outcomes but remain far from numerically exact. The
coefficient schedule, subnormal publication, fractional power/product staging,
multivariate/default/orientation/coercion axes, and a genuine held-out remain
open. Canonical evidence is in
`W109_GROWTH_LOGEST_SINGLE_PREDICTOR_DISCOVERY_20260809.md`.

For `CUMPRINC`, a frozen 60-call PMT companion and 540-call CUMPRINC discovery
battery separates published payment, hidden per-period principal, and hidden
range-fold publication. Shipping is `90/540`; the best oracle-blind
discount/geometric family is `190/540`, and the broad public `loan.fs` family
is `172/540`. Exact power-of-two homogeneity coexists with non-exact rounded
range partition identities. A hidden first-principal low word is insufficient,
and the apparent `498/540` Ext80 score is explicitly rejected as 90-parameter
per-query interpolation with no context transfer. No held-out or production
survivor exists; the durable report is
`smart-fuzzer/tools/calc_graph_racer/CUMPRINC_EXACT_PARTIAL_REPORT_20260809.md`.

### 7.9 RATE private-objective and publication checkpoint

The current-build RATE lane now has a frozen 256-row cancellation-tuned
discovery bank and a paired 512-call worksheet-FV companion, both captured on
Excel build 20228, x64, Compatibility Version 2 through exact-bit Value2
plumbing with NoCache and clean pre/post process checks. The RATE generator
makes every tested first residual smaller than `1e-7`, so the bank directly
tests whether Excel publishes the current guess or applies a step.

No tested graph is exact. The frozen balance/finite-difference/update grammar
has zero survivors among 13,824 candidates and a best score of `2/256`.
Replacing the private objective with worksheet FV also tops out at `2/256`.
A 30,720-graph helper race reaches `502/512` on the paired FV calls, but all
associations share ten cancellation/small-rate misses; expanding to 7,864,320
raw-power inline-helper and outer-spill graphs still leaves RATE at `2/256`.
This is a bounded negative over the enumerated grammar, not an
irreducibility result.

The schedule layer does establish one directional fact: a pre-step residual or
delta check that publishes the unstepped current guess scores `0/256` and is
refuted. First-step, residual-stop-next, and delta-stop-next remain
observationally tied while the private objective is inexact. The inherited
x87-power and basin evidence remains useful structure but is not a complete
current-reference graph. The answer-blind 256-row heldout remains sealed and
uncaptured. Canonical evidence, artifact hashes, replay commands, and the
three-axis audit are in
`smart-fuzzer/tools/calc_graph_racer/RATE_EXACT_GRAPH_PARTIAL_REPORT_20260809.md`.

### 7.10 PRICE and DURATION residual-graph checkpoint

The previously landed bond schedule repairs remain materially correct, but
their residual exact-graph lanes are not identified. Fresh build-20228/CV2
Value2/NoCache discovery captures add 528 PRICE rows, 264 DURATION rows, and a
frozen adaptive 72-row PRICE companion to the historical build-20131 controls.
All fresh artifacts carry aligned ids/argument bits, numeric kinds, and clean
pre/post Excel process checks; the initially generated disjoint PRICE and
DURATION heldouts remain sealed and uncaptured.

For PRICE, production is `564/600` and the best coherent Chain-power,
forward-fold, separate-redemption family is `571/600`; every one of the 29
remaining rows is exactly one ULP low. There are zero exact survivors among
1,152 fixed graphs, 288 retained-PC64 variants, 80 factorized-coupon variants,
and 48 fixed association families. The six original misses and 23 adaptive
misses therefore do not support the former simple “shared fractional-pow wall”
attribution: retaining base/exponent/pow/result lifetimes does not improve the
leader. An accumulator or publication axis is still missing.

For DURATION, production and the best fixed graph are `237/264`, max 3 ULP.
Neither 288 retained-PC64 variants nor 72 factorized-coupon variants contain an
exact survivor; factorization lowers maximum distance to 2 without increasing
the exact count. The landed schedule quantities and coarse cashflow ordering
remain supported, but the numerator/denominator accumulator graph is open.
Hashes, provenance, candidate scores, and replay commands are in
`smart-fuzzer/tools/calc_graph_racer/PRICE_DURATION_RESIDUAL_GRAPH_SCOPE_PARTIAL_20260809.md`.

### 7.11 IRR objective/evaluator decomposition checkpoint

A frozen 300-row build-20228/CV2 IRR discovery set and an answer-blind
900-point worksheet-NPV companion now separate three surfaces that the July
description conflated: raw worksheet NPV, worksheet evaluator publication, and
IRR's private objective. The companion captures raw NPV, direct `NPV+c0`, and
referenced-raw-cell `+c0` at the supplied guess and both sides of a binary32
`0.001` perturbation in discount-factor space.

Worksheet direct and referenced-cell composition agree `900/900`. Both snap 18
nonzero near-cancellations to +0; a scale-relative threshold between the
largest snapped ratio `5.684341886080802e-16` and the smallest published
nonzero ratio `2.830802259268159e-14` classifies the discovery, but its exact
constant is not pinned. IRR does not inherit any of those 18 snap decisions,
and adding the smallest classifying snap to the best worksheet-tail objective
worsens the guaranteed two-step subset from `40/72` to `37/72`.

Raw worksheet NPV also remains structurally open: reverse-Horner division leads
at `636/900`, max 4 ULP, with no exact survivor. The best frozen no-snap IRR
objective/schedule graph is only `44/72` on the guaranteed two-step subset; the
public VB Financial.IRR control is `2/300`. The 180-row heldout is still sealed
and uncaptured. Exact hashes, provenance, replay commands, and the three-axis
handoff are in
[W109_IRR_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md](../function-lane/W109_IRR_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md).
The open lane is registered as `BUG-FUNC-048` and W109 child bead
`oxf-jwh5.10`; neither is a completion claim.

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: all remaining catalog rows, including PMT-family and the
   ERF/ERFC.PRECISE members of mixed G4-04;
   broad post-catalog discovery; declared application-version/Compatibility-
   Version axes; global OPERATIONS Sections 12 and 14 audit. PMT specifically
   retains the private helper, type-1 association, timing/publication, adjacent
   schedule, and unfinished larger-graph lanes. GROWTH/LOGEST retain coefficient,
   publication, full-shape/coercion, and held-out lanes; CUMPRINC retains hidden
   principal, range-fold, and held-out lanes. RATE retains its private
   cancellation/small-rate objective, exact FD/update association, publication
   distinction, wider semantic surface, and sealed heldout lanes.
   PRICE/DURATION retain their residual accumulator/publication graphs and
   sealed heldouts. IRR retains its private objective, scale/error boundary,
   remaining schedule/publication graph, and sealed heldout. COS/BESSELJ,
   ATANH, ACOTH, MINVERSE, CONVERT, COMBIN, and COMBINA closures are scoped and
   do not close the wider campaign.
