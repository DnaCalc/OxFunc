# W109 wall-clues ledger

A running trail of clues and instruments for the parked op-graph walls,
collected en route through other lanes (started 2026-07-18, four-lane sweep).
Read together with W109_CAMPAIGN_RESUME_20260718.md (wall definitions and
designed probes). Add to this file whenever a lane surfaces something a wall
will want; date every entry.

## Wall 1 — chain-microdetail ±1 (series r / exp idealization class)

- **2026-07-18 (lane 1):** two fresh banked class members, both on the plain
  exp path (no series involvement):
  - `b24W-b2-0340`: WEIBULL cdf, x bits `4018222d34979553` (x≈6.03338,
    α=2, β=2, t≈9.10041): Excel's internal `exp(−9.10040519235115…)` is
    **+1 ULP above our chain** (published cdf 1 below all staging
    candidates). Args and expected bits in `answers-b24-weibull.json`.
  - `b28-3838`: WEIBULL cdf, args x=`3fc3d9d7eab87363` α=`401dd48b49e40d40`
    β=`3fd1feab7fc34ab8` (t≈0.0117): production −2 ULP. Only miss in 6,000.
  - Class rate stays ~0.01–0.02% across every corpus (1/17,300 b24-weibull;
    1/6,000 b28; 3/30k POISSON; 4/18k expm1). The rows are argument-stable
    (deterministic), scattered, and NOT correlated with the F2XM1 fraction
    bins we binned earlier — worth re-binning against the reduction
    residue `f = t·log2e − round(t·log2e)` with the new rows added.
- **Instrument:** `x87_serve` op server + `x87client.py` (lane 1) makes
  op-by-op chain dissection scriptable from Python — apply to agentQ_diag7's
  seven rows without hand-built Rust harnesses.

## Wall 2 — ln-amplification at a≥3 (GAMMA.DIST series)

- **2026-07-18 (lane 1):** WEIBULL/EXPON proved the closed-form bodies form
  their products as **x87 double-rounded ops** (RN64 then RN53 spill). The
  b26 a·L staging race assumed a plain product; a DR product `RN53(RN64(a·L))`
  is a one-line re-race on the banked b26 rows (x87client makes this
  minutes). If the gamma series body is (even partially) the legacy x87
  class, the a-growth of the worst-case (4→7→10) could be DR-accumulation,
  not L-delivery.
- Caveat the other way: GRATIO itself was proven plain-SSE2 by structure
  racing — the two body classes COEXIST in the 2010 rewrite (lane-1 big
  picture). Which class owns the `t1 = a·L − y` site is now a live question
  rather than settled background.

## Wall 3 — erf 190-path (C10r plateau, 2^Ez grid source)

- **2026-07-18 (lane 7): b33 CROSSING SWEEP — the wall resolves into
  per-window integer-ULP term corrections.** Whole windows land bit-exact
  under a single term nudge (A1w1 `M+1` 60/60; A2w1 `b1+1` 58/60; A3w2
  `M−2` 56/60): every window has a CONSTANT argument offset of
  ±(1–4)·2⁻⁵³. Same-anchor consistency separates lf-side (A1: both
  windows −1·2⁻⁵³) from bd0-side (A3/A4: p-varying) realizations; the
  chain is exonerated with POSITIVE evidence (staircases reproduce exactly
  under one correction). stirlerr is inert (ulps ~2⁻⁶² — below
  resolution). **Endgame = integer-ULP linear system**: ~10 short windows
  per anchor overdetermine the per-class term corrections; then match the
  solved values against published bd0/lf realizations. Template:
  `lane7_b33*.py` + answers-b33. Slow-walk windows need near-mode p
  (design note: ulps/step ran 3–1,066, target 5–50).
- **2026-07-18 (lane 6d): EXACT-CONSTRAINT REFRAME (user directive) — the
  wall's information budget stated precisely.** Single reads carry no
  information below publication granularity (decode floor ≡ output
  granularity in argument space); match-rate deltas in the 32–36% band
  were phase, not signal. Pair search RUN AND RETIRED BY ARITHMETIC
  (zero pairs ≤64 ulp64 in b29; the p-dial is 2^8 too coarse to design
  collisions). **The right instrument is the b33 boundary-crossing sweep**
  (erf tooth-law bisection with a KNOWN argument model — first time both
  sides of the equation are available): p-ulp sweeps for fixed (k,n),
  each published-staircase transition = one exact inequality pinning
  CHAIN(arg) to a known boundary within one p-step; anchors spanning
  |arg| 10–600 separate argument-side (low |arg|) from chain-side
  (high |arg|) deviations. Full design in the lane-6d note section —
  execute as the next session's opening move.
- **2026-07-18 (lane 6c): joint solve — flagship hypotheses refuted;
  status sharpened.** x²-spill (erf), M-as-tbyte and every lf composition
  (BINOM), and the c-vs-reduction-fraction chain-error map (all-rows) are
  ALL refuted. With the argument families CLOSED (lc/lf doubles + exact
  subtract), the residual ±1 publication scatter is chain/publication-side
  — but note the epistemic correction: windowed-76% was weak evidence
  (any monotone chain windows); the true discriminators are the no-window
  classes (confirmed = visible-level term classes) and correction-map
  structure on ARGUMENT-CERTAIN rows. Next: (i) c(f) map on
  argument-certain subsets of BOTH corpora (same map ⇒ chain-side);
  (ii) cross-corpus exact-argument pairs (<1 ulp64 apart) — their
  published values must agree under ANY deterministic chain, the cleanest
  litmus; (iii) ±300 narrow-interval solve with joint term nudges.
- **2026-07-18 (lane 6b): MERGE CONFIRMED — walls 3 and the BINOM blocker
  are ONE.** The erf j-scan (`check_erf190 <dir> jscan`, dev corpus only,
  b9heldout untouched) reproduces the BINOM signature number-for-number:
  windowed 76.5% (BINOM 76%), no-window 23.5% (= the j-pipeline park-phase
  visible class), centers median-0 with ±240-ulp64 sub-double scatter.
  The unified wall: **sub-double argument composition at extended-entry
  exp sites** (how 32-bit codegen forms `a·ln_ext(x)` / `lc − 0.5·lf` at
  64-bit before the chain) + a per-site ~24% visible-level term class.
  One unknown, two corpora, one instrument (the narrow-interval solver).
- **2026-07-18 (lane 6): THE CHAIN IS EXONERATED — the wall is
  ARGUMENT-SIDE.** The j-interval scan on the b29 oracle pairs proves the
  composed extended-entry fFEXP chain reproduces Excel bit-for-bit given
  the right 64-bit argument (consistent interval for 76% of rows; the rest
  are known ±1-term rows). What is unknown is the argument's sub-double
  content (below the decode floor): single-source lf hypotheses all
  rejected by the narrow-interval solver; a second source correlates with
  EXTREME p. **Re-read the erf C10r plateau as argument-delivery detail,
  not chain realization — run the same j-scan on b9heldout with C10r's
  argument model to confirm the merge.** Method: `lane6_jscan/solver.py`
  (the narrow-interval solver is the reusable instrument); server ops
  `lnext` (hardware extended fyl2x as hi+lo) + `cexpext2`.
- **2026-07-18 (lane 5): THE WALL GAINED A DIRECT ORACLE.** BINOM's
  dbinom_raw argument staging is now identified to publication-noise level
  (82.5% of decodable b29 rows at d≈0.00), and the end-to-end blocker is
  the SAME primitive as this wall: the fFEXP chain entered with an
  EXTENDED argument. The ~500 correct-argument b29 rows are
  (arg_ext64, published_bits) pairs — attack the extended-entry chain
  realization on THESE (argument exactly known!) instead of erf's
  never-recovered arguments. Tooling: `cexpext2 hi lo` server op runs the
  current composed chain from an exact 64-bit argument; it agrees with
  Excel only ~45% on correct-arg rows — enumerate reduction/publication
  micro-variants (f2xm1 argument handling, rndint mode, store order)
  against these pairs. A win here closes BOTH the erf plateau AND BINOM
  (and presumably NEGBINOM + the POISSON small-λ side).

- **2026-07-18 (lane 1):** no direct new probe, but two transferable facts:
  (i) association ORDER is recoverable and matters — the WEIBULL pdf race
  was stuck at 805/1200 until the tree enumeration found division-first;
  the erf polynomial-evaluation candidates were enumerated over staging
  variants but (check the agentJ ledger) possibly not over association
  orders of the rational's numerator/denominator combine.
  (ii) mixed spill masks (some intermediates extended, some spilled) are
  REAL in this codebase — round-6 style tree×mask enumeration via mpmath
  is cheap and exhaustive; the erf chain-floor question ("parked vs
  register-continuous") is exactly a mask question.
- b9heldout (256 rows) remains the reserved unraced promotion gate.

## Wall 4 — bgrat body op-graph

- **2026-07-18 (lane 1):** the falsified bgrat families were "53-bit per-op
  DR" and "register-resident" — but the WEIBULL pdf shows the actual bodies
  use **specific C-source association orders** (division first) under
  per-op DR, and that getting the ORDER wrong while the op class is right
  still costs ~25% of rows at ±1. Recommend: re-enumerate the bgrat body
  candidates as expression TREES (round-6 method, `lane1_pdf_round6.py` as
  the template) over the shared-z group-intersection corpus before
  declaring the family dead.
- If bgrat forms powers `x^a`/`(1−x)^b` anywhere via pow, the site is now
  known: `excel_pow_chain` (DR product, no shortcuts) — earlier races used
  idealized pow models at some stages.

## Wall 5 — GAMMALN b1/b2 coefficients

- **2026-07-18 (lane 4, agent-S + b32 fresh gate): B2 RE-LANDED as
  fully-continuous x87 with LM-refit coefficients** (fresh b32: 549 vs
  518 / 1,200; noise floor 1.077 vs 1.113; B2 and B4 are now the SAME
  op-graph class — architecturally coherent). B1 confirmed AT ITS WALL:
  published-1967 plain double is the held-out optimum of the entire
  (d,s,e,fma,recip)×association staging family; misses reach −3.4 ULP
  pre-round (not coefficient-fixable). What remains is outside the family:
  next probes = two-step argument reductions, non-minimax (re-weighted
  Remez) coefficient family, outer 2-op mask after e/e/e refit
  (agentS_results.md §7).
- Method clues banked: (i) CVP pre-round distances need mp.prec≈200 —
  dps-15 catastrophically cancels and fakes integer clustering; (ii) the
  gn2 fit was held2-contaminated (agentL fit_sets excluded only `held-`) —
  always exclude EVERY held set by prefix; (iii) the fresh-corpus rule
  worked exactly as designed: a formally-gate-passing candidate whose
  selection touched held-out LOST on never-probed rows (505 < 518).

## Wall — internal extended lgamma (G3-02)

- **2026-07-18 (lane 2): NEW MEASUREMENT WINDOW.** BINOM.DIST general-k is
  (leading hypothesis) `exp-chain(lnΓ_int(n+1) − lnΓ_int(k+1) −
  lnΓ_int(n−k+1) + k·lnp + (n−k)·lnq)` — the implied-argument decode of a
  BINOM pmf row reads a THREE-LGAMMA linear combination to ~0.02
  ULP-of-argument. Same integers recur across rows ⇒ overdetermined
  systems solve for per-integer internal-lgamma values bit-for-bit.
  Banked: b29 corpora + `lane2_binom_implied*.py` (the decode machinery).
  Every simpler lnC realization is REFUTED at the exact-bit level
  (see the lane-2 note section) — the ±2-ULP(arg) bell with uniform
  fractional part is the wall's own fingerprint.

## Wall 6/7/8 — POISSON k≥1, distribution pow, BETA.DIST probes

- **2026-07-18 (lane 1): wall 7 (distribution pow staging) is CLOSED** —
  `excel_pow_chain` landed, b27D 113/113, WEIBULL/EXPON signed off at
  99.983%/100.000% held-out. The POWER wrapper owns the 0.5→sqrt shortcut;
  the CRT pow underneath is the pure chain (story-grade: the shortcut is
  Excel's, not the C runtime's).
- ~~For wall 6 (POISSON k≥1 product staging): predicted x87-DR~~ —
  **superseded by lane 2 (2026-07-18)**: POISSON pmf is TWO routes (k=1 =
  extended-composed direct product, exact at large λ; k≥2 = Loader
  saddle-point dpois, bit-exact at λ ≳ 14; small-λ staging + branch
  structure open). The old "direct product proven / 21% unexplained"
  verdict was a route-blind-window artifact — withdrawn. See the lane-2
  note section for the full refutation ledger.
- BINOM cdf(0) ≡ pmf(0) bit-identical 1000/1000 (b24BT) — k=0 shares the
  pmf fast path, no bratio at k=0.
- Loader's dbinom x==0 branch is `(p < 0.1) ? -bd0(n,nq) - np : n·log(q)` —
  **SETTLED (b29b): Excel DOES flip below p=0.1** (bd0-form 383/400) — the
  Loader-control-flow smoking gun. 16 neither-rows = bd0-series/compose
  sub-staging, part of the bounded general-k enumeration.
- **2026-07-18 (lane 3): wall 8 first half CLOSED** — Excel has no
  integer-shape BETA.DIST special path (b30); A/B bounds staging broadly
  confirmed. Both OxFunc-side integer fast paths (gamma cdf, beta cdf)
  REMOVED from production.
- **GAMMA.DIST pdf MEASURED (b31, 4,750 rows banked) — new named wall:
  the closed-form-pdf extended-composition body class.** Triage REFUTED at
  the exact-bit level: production log-composed (16.1%), direct separate-pows
  (18%), ratio forms (22%), and R's dgamma-via-dpois structure (20% — NOT
  R's dgamma, unlike POISSON which IS Loader at k≥2). The sharpest read:
  a=1 (pdf = e^{−x/β}/β) fits the **reciprocal-based EXPON-style staging**
  (λ=1/β then λ·exp(−λx)) at 42.3% with a TIGHT ±1–2 residual — the same
  signature as POISSON k=1's extended-composed product (C4 class, ~70%).
  Hypothesis: POISSON small-λ, POISSON k=1, GAMMA.DIST pdf (and predictably
  CHISQ.DIST/BETA.DIST pdfs) share ONE legacy x87 body class whose
  extended-vs-spilled composition pattern is the single remaining unknown —
  crack it once, land it four times. Instruments ready: x87_serve
  mulex/mulee/pmfk ops + the b24/b31 banked corpora. Fractional-a b31F rows
  (1,000) additionally read the internal lgamma through stirlerr's
  non-integer branch if the dgamma road ever reopens.
