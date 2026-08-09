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
   ACCRINT triple-edge, CONVERT (solve implied factor bits), ACOTH, COMBIN
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
   regression family; MINVERSE distinguishing-matrix race.
5. **Phase 6 — ATANH piecewise search** over the 368-case corpus with
   branch-threshold binary search; then a full catalog re-sweep after each
   primitive lands.

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

### 7.2 Newly identified financial graphs

The following lanes have zero residual on their current discovery, held-out,
and targeted discriminator corpora, but their catalog retirement remains gated
on final provenance-bearing recapture, full tests, and evidence synchronization:

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

### 7.3 BESSELJ and PMT remain open

1. `BESSELJ` requires worksheet COS at both J0/J1 asymptotic cosine sites and
   an x87-double-rounded `cosine*p` only in J0. Those body choices plus live COS
   values decompose the fresh held-out battery at `794/794`; the best executable
   model is `792/794`. The remaining pair is a shared worksheet-COS substrate
   gap at exact phases `0x4062a6de04ab6900/6902`, registered separately as
   BUG-FUNC-047/G4-07. This is an active identification lane, not a signed-off
   repair.
2. The July-25 PMT takeover brief explicitly retracts the July-24
   "proven irreducible / needs provenance" framing. The only defensible result
   is bounded-negative over the documented leaf/operator/size limits. A
   reproducing Excel program exists; larger graphs, coefficient recovery, and
   residual wrapper/predicate axes remain actionable. EXT6 is also incomplete:
   its log stops at shard `191/400`.

Status axes:
1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: landed EFFECT/RRI/NOMINAL sign-off (post-repair full core:
   `1518` passed, `4` ignored);
   shared COS and dependent BESSELJ graphs; all other catalog rows; broad post-catalog discovery;
   declared application-version/Compatibility-Version axes; global OPERATIONS
   Sections 12 and 14 audit.
