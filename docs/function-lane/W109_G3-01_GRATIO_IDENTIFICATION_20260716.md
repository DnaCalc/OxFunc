# W109 G3-01 — Distribution substrate identified as the DCDFLIB/NSWC GRATIO family (2026-07-16)

Status: **major identification, not yet bit-closed.** The incomplete-gamma side of the
G3-01 distribution substrate is the **TOMS 654 GRATIO branch structure (DiDonato &
Morris / A. H. Morris, NSWC), compiled in plain SSE2 double** — NOT an x87-extended
continued fraction as the 2026-07-14 sweep hypothesized. The remaining unknowns are
reduced to three named sub-identifications (Excel-internal erf/erfc, the internal
Γ/lgamma normalizer, Taylor-path micro-staging) plus the beta-side sibling (BRATIO,
TOMS 708, unprobed).

Work dir (gitignored, captures + harness): `smart-fuzzer/work/w109/G3-01-dist/`
Racer binary (tracked): `smart-fuzzer/tools/calc_graph_racer/src/bin/check_igamma.rs`
All live-Excel answers are also in the persistent OracleCache.

## The multi-view collapse (battery B1, 829 probes, build 20131)

- Legacy ≡ modern bit-for-bit everywhere probed: CHIDIST≡CHISQ.DIST.RT,
  FDIST≡F.DIST.RT, TDIST(·,·,1)≡T.DIST.RT, GAMMADIST≡GAMMA.DIST, BETADIST≡BETA.DIST.
- GAMMA.DIST beta-scaling is transparent for exact transforms (β=1 vs β=2 at doubled
  x agree 33/33) → one internal P(a, y), y = x/β double-divided first.
- CHIDIST is NOT RN53(1 − published P) (16 eq / 17 ne) → Q has its own publication.
- Deviation-from-correctly-rounded propagates IDENTICALLY through views → one kernel.

## Key evidence chain

1. **Excel ~67% CR-exact overall** with |δ| up to 47; δ magnitude scales like
   |a·ln x|·2⁻⁵³ → double-rounded exp-argument (battery-1/2 deviation map).
2. **Implied-argument decode** (`arg_fit.py`): at a=3 the argument error equals the
   exact rounding residue of `R53(R64(3·R53(ln x) − x))` on every row (double log,
   double argument); at a=2 it is ~2× the ln residue; **at a=1 the argument is clean
   of ANY double-rounding** — impossible for one code path…
3. **…because a==1 is a dispatch**: `a=1+2⁻²⁰` is NOT clean (battery-5), and the
   a=1 slice fits the exponential-CDF closed form (−expm1(−x) family): wrapper race
   179/205 vs 143/205 without. GAMMA.DIST(x,1,β) IS the exponential CDF.
4. **Faithful GRATIO transcription** (`cdflib_py.py`, from the scipy v0.14 cdflib
   Fortran) races at 416/692 overall, with the match rate concentrated exactly where
   GRATIO's branch map predicts (`gratio_detail.py`):
   - `closed-int` (integer a, a ≤ x < 31: finite exp(−x)·Σxᵏ/k!): **199/218 = 91%**
   - `asymp` (x ≥ 31): 4/6; `temme` (a ≥ 20, a=15 via the |a|≥15 gamma branch): 5/5
   - `taylor` (x ≤ max(a, ln10), the wk[20] backward-tail series): 48% with small
     per-a biases — normalizer + micro-staging residual
   - `closed-halfint` (erfc1-based) and `erf(a=1/2)` routes: fail ±1..±24 → Excel
     uses ITS OWN erf/erfc, not the NSWC rationals.
5. **Normalizer**: NSWC `gamma()` value error (~8e-15 rel at a=2.75) explains the
   constant +32..+42 offsets at fractional a; CR-Γ improves (a=1.25: 0→10/20) but is
   not exact either; a=4/6 (Γ exact) still miss → independent Taylor micro-staging
   residual. The internal Γ/lgamma is the same unknown as G3-02 — this lane now gives
   a per-a MEASUREMENT channel for it (each (a, many-x) slice over-determines the
   internal normalizer value).
6. **Excel's erf is near-CR** (ERF.PRECISE 158/176 CR-exact, ±1-2 tails): NSWC ruled
   out (113/176, regime flip at its 0.5 boundary), Cody SPECFUN CALERF ruled out
   (121/176 erf, 56/176 erfc, both exp models). 352 ladder points captured
   (`answers-erfp/erfcp.json`). Next candidates: fdlibm s_erf/s_erfc, Boost.
   ERF.PRECISE/ERFC.PRECISE/GAUSS (G4-04/G3-07) get closed by the same sub-lane.

## Corrections to prior claims

- The 2026-07-14 sweep verdict "x87 80-bit EXTENDED kernel; fix = x87-extended CF"
  is WRONG. The extended-precision convergence analysis only proved the CF converges;
  the kernel is plain double with the GRATIO branch structure. (The catastrophic
  6224-ULP CHIDIST row is OxFunc's plain-double NR complement, not an Excel-side
  extended CF — Excel's closed-int branch avoids the cancellation entirely.)
- "Distributions = one incomplete-γ CF kernel" under-described the structure: five
  branches (closed-int / closed-halfint / Taylor / CF / Temme) + the a==1 and a==0.5
  dispatches.

## erf sub-lane findings (2026-07-17 session)

**Wiring PROVEN by cross-view**: `GAMMA.DIST(k²/1024, ½, 1, TRUE) ≡ ERF.PRECISE(k/32)`
and `CHIDIST(k²/512, 1) ≡ ERFC.PRECISE(k/32)` — **160/160 AND 160/160 bit-exact**
(`answers-xv-gd/xv-chi.json`). The gratio a=0.5 dispatch calls exactly the published
ERF.PRECISE/ERFC.PRECISE routines; one identification closes all of them + CHIDIST
odd-df + half-integer paths.

**Architecture PINNED, tables custom.** Ruled out bit-exactly: NSWC (113/176 erf),
Cody SPECFUN CALERF (121/176), fdlibm s_erf (160/176 — closest but ±2), Boost
1.35–1.42 erf_imp<53> (155-157/176; tables identical across versions), and the local
Microsoft UCRT erf (146/176) — all with CR/fdlibm/x87 exp models. Established:
- **erfc computes exp(−RN53(z·z)) with an UNSPLIT argument** — proven by regression
  of the messy-grid (full-mantissa z) residual against the exactly-computable
  RN53(z·z) rounding error: slope +0.95, residual stdev 7.9e-16 → 2.0e-16 (±1 ULP
  floor). This kills all split-argument families (fdlibm/Cody) structurally.
  CAUTION (method): the original k/32 ladder had EXACT z² everywhere — dyadic-clean
  grids can make split-vs-unsplit invisible; always add a full-mantissa grid.
- **No tiny-z shortcut** (Boost's z<1e-10 form ruled out): tiny-z rows are consistently
  CR+1, and 12/15 give the SAME ratio double — the small branch's value at 0 is
  pinned exactly: **R(0⁺) = 0x3ff20dd750429b6e = CR(2/√π)+1 ULP**.
- **No Boost 5.8f erf→1 cutoff** (CR, not 1.0, just above 5.8f). erf = 1−erfc above
  the small branch (long CR-exact run for z ≥ 1.375 on the clean grid).
- **Small-branch/complement boundary at 0.5** (messy crossing scan consistent).
- The apparent erfc hard-zero in (26.543, 26.544] is NOT a Cody XBIG: the last
  finite witness sits in the smallest normal binade (0x001…) — it is the known
  Excel-wide **subnormal publication flush** (same as the PHI lane's pinned flush).
- Detrended erfc residual is a flat ±1-ULP evaluation floor across [0.5, 6] — a
  uniform-quality rational (or intervals of equal quality; no sharp boundary signal
  at Boost's 1.5/2.5/4.5 or fdlibm's 1.25/2.857).

Data: `answers-erfp/erfcp.json` (clean k/32 ladders), `answers-erfm/erfcm.json`
(full-mantissa), `answers-b7erf/b7erfc.json` (lineage fingerprints),
`answers-b8erf/b8erfc.json` (flush bisect + 0.5 crossing). Harness: `erf_map.py`,
`erf_cody.py`, `erf_fdlibm.py`, `erf_boost.py`.

## erf coefficient-recovery outcome (2026-07-17 session) — THERE ARE NO COEFFICIENTS

The "custom Microsoft rational tables" hypothesis DISSOLVED. Findings, in order:

1. **All remaining published candidates ruled out** (agent sweep + direct races):
   Boost int_<64> tables, Cephes ndtr, Ooura gamerf derf, Hart 5666, SLATEC,
   renormalized Cody/Cephes variants, plus every constant/rational micro-form
   (multiply, divide, split-constant, Padé [m/n] m,n≤8, Taylor truncations,
   two-product sums — all swept over constant neighborhoods).
2. **The z<0.5 branch is the NSWC gratio a<1 DIRECT path (branch 190)** — the same
   TOMS-654 source as the gamma side, with a mis-transcription trap: for a=½,
   x=z²<0.25 always routes 190 (`ans = exp(a·ln x)·g·(0.5+(0.5−j))`), NEVER the
   complementary 200-path (whose 1−q staging is catastrophic at tiny z in ANY
   precision — proof by granularity).
3. **The mystery constant is g = 1 + gam1(½)** — the NSWC gam1 rational itself,
   evaluated in x87 EXTENDED: h = `0x3fc06eba8214db6c` (cf. fdlibm's efx
   `…db69` — 3 ulp apart), g ≈ true·(1+8.5e-17), matching the measured effective-
   constant center (+6.5e-17 ± wobble). The earlier "q0 = sqrt(π_double)/2 anchor"
   was this same value seen through a division-form lens (1/g quantization
   coincidence, 11/15 partial fit).
4. **x87-extended compilation required**: all-double staging gives ±8-ULP tiny-z
   wobble (ruled out); extended log/exp keeps the exp(0.5·ln x) round-trip
   sub-ULP as observed. NOTE the tension with the gamma-side a≥1 path 20, which
   PROVED double-rounded log staging (`sp_both`) — different call sites of the
   same compiled function can spill differently; treat per-branch.
5. **Best models**: true-x87 Rust race (`check_erf190.rs`, Ext80 fFEXP/fFLN,
   512 spill configs): 663/1218 on z<0.5 (misses ±1, one ±2); designed
   max-|δ(z²)| battery (`answers-b10.json`): z̃=z-direct×g wins 37/50 over
   sqrt/explog-of-RN53(z²) and reflection forms. ~92% of rows within ±1.
6. **Residual — now QUANTIFIED (2026-07-17 hunt session)**: the last staging op
   is a deterministic sawtooth measured at full resolution on 128-step mantissa
   ladders in two binades (`answers-b11.json`, m30: z≈2⁻³⁰; m20: z≈2⁻²⁰) via the
   true-x87 dump mode (`check_erf190 <dir> dump` → per-row model phase):
   ε(m) = rising ramp ≈ +0.145 ULP per 1/128-mantissa-step, cut by −0.85-ULP
   down-teeth whose frequency GROWS with mantissa (period ≈6 steps at m≈1.4,
   ≈2.4 at m≈1.9 → tooth-phase ∝ m³ᐧᐧ⁴), amplitude ×3 larger at m20 than m30.
   This is the beat of an x²-magnitude term against a fixed ~2⁻⁶⁴ quantum on a
   z²-scale accumulation — i.e. the series/j/inner cluster in an arrangement
   OUTSIDE the tried parametrization (13 axes enumerated and ruled out at the
   850/1508 plateau: zz/series/j/zl/gam1-eval/gam1-ret/g/w/inner spills,
   association orders, GRAT1 pass-r, closed z-direct/sqrt/explog forms — b10
   designed max-residue battery pins z̃ as z-direct 37/50). Next probe: derive
   the exact tooth-position law from the dump (teeth are bit-precisely located),
   fit candidate quantized-accumulation generators analytically (sum⊕t at 64-bit,
   inner-poly x-term phases, alternative j-series arrangements incl. the Excel
   variant possibly fusing (1−j)), and verify against the m20 tooth set before
   any spill re-enumeration. Data: `answers-b9train.json` (1190),
   `answers-b9heldout.json` (256, UNTOUCHED — promotion gate), `answers-b10.json`
   (50), `answers-b11.json` (256 ladder), `answers-b11c.json` (511 erfc
   complement view, unanalyzed — pair-decoding reserve), `dump-m30.txt`.

## erf tooth-law measurements (2026-07-17 second hunt session) — bit-precise phenomenology

The last-op sawtooth was measured to bit precision via oracle-driven batched
bisection (`tooth_bisect.py`, 16-point rounds, ~16× bracket shrink/round;
`tooth_positions.json`). Facts any candidate mechanism MUST reproduce:

1. **Teeth are steps in Excel-vs-true-chain-model ε**, −0.85 ULP each, ramp
   +0.145 ULP per 1/128-mantissa step between teeth (z≈2⁻³⁰ binade).
2. **Linear in z, constant period within a binade, near-zero-anchored**: 11+
   consecutive teeth at exactly equal Δz; two teeth bisected to ~2⁻⁶⁶-relative:
   t1 = 1.13327213842e-9, t2 = 1.52563759794e-9 = t1 + EXACTLY 9 periods →
   **p(e=−30) = 4.35961621689e-11** (= 2.996·2⁻³⁶, NOT exactly 3·2⁻³⁶);
   t/p ≡ 0.995 (mod 1) — grid anchored ~at zero.
3. **Regime transitions**: half-quantum grid slip at m = 1.5 exactly; spacing
   collapse ≈3× after V crosses its binade (m = 2/g ≈ 1.7724).
4. **Cross-binade nesting**: at e=−40 BOTH a coarse (~6e-14) and a fine
   (5.949e-17/k, k∈{1,2} unresolved — bad bracket) structure exist; periods are
   NOT a smooth function of e (apparent log-slope ≈1.95 between −40/−30 but the
   law breaks at −25/−15). Multi-scale/nested sawtooths.
5. **Amplitude law: ε_amp·|zl| ≈ 25.7 ULP constant** across binades
   (0.93@|zl|=27.7, 1.48@17.3) ⟺ constant-absolute perturbation of zl²/2 —
   no computed quantity matches yet.
6. **Q-side**: ERFC ≡ RN53(1 − P-internal) at the tooth zone (48/48) — the
   generator is P-side/pre-complement; the complement is double-staged.
7. Ruled out structurally: all log-spaced generators (exp/ln tables, F2XM1
   boundaries, zl-grids — teeth are z-linear), x-quantized generators
   (1/m-spacing contradicts the exact arithmetic tooth progression), decimal
   round-trips, w-quantization (amplitude would be enormous), single/double/
   split-constant forms, and 1024 spill/association configs of the 190-path
   with true x87 chains (plateau 850/1508).

**Continuation plan**: (a) resolve p(−40) k-ambiguity with a fine scan
IMMEDIATELY after t4 (the t5 bracket excluded the first ~5e-17); (b) bisect
neighbor-teeth at e=−15 (t8 = 3.70029939053e-5 done) and e=−25 (t6 bisection
stalled — teeth shallower there); (c) fit m30's full 128-point ε as
ramp+sawtooth and examine the residual for the second generator; (d) test
generators whose PHASE is z-linear but AMPLITUDE is ulp-scale (beat/modulation
constructions); (e) the b11c erfc complement mid-range pair-decoding remains
unanalyzed. All captures cached; batched-bisection machinery reusable.

## Multi-agent closure sweep (2026-07-17 session 3) — five parallel verdicts

1. **BETA SIDE CONFIRMED = BRATIO (TOMS 708)** (`agentA_bratio.py`, full dependency
   tree transcribed): argument stagings pinned — wrappers pass the ACCURATE
   complement (`FDIST: x=d2/den, y=d1*F/den`; `TDIST: x=df/den, y=t²/den`, one-tail
   = 0.5·two-tail bit-exact). Decisive branch-differential: **bpser in plain double
   BEATS correctly-rounded betainc on FDIST/TDIST** (11/6, 5/3, 8/4) — literal code
   identity; F.DIST.RT bpser 5/5. **bgrat (a≫b tail) is the Excel-custom/extended
   sub-kernel** (double 2/12 vs CR 8/12) — the beta-side erf-analog. bpser's ±1-2
   residual = the x87 EXP/LN signature. Branch battery for bgrat: `answers-b15.json`
   (126 rows, cached).
2. **Taylor staging CORRECTED + normalizer MEASURED** (`agentB_*`): Excel sums the
   series FORWARD with 1/a as an OUTER factor (28/45 vs wk-backward 16/45 at a=2);
   normalizer = divG with **G = CR-double Γ(a) ±1 ULP ≡ exp(internal GAMMALN)** —
   NOT NSWC gamma (+22..33, score 0), NOT worksheet GAMMA(a). At a=2, EVERY miss
   resolves within ±1 ULP of exp(t1) with zero series changes — the residual is
   Excel's ≤-nearest-rounding exp (x87 CRT, one-sided). Feeds G3-02: the internal
   lgamma at fractional a is pinned to exp⁻¹(CR-Γ ±1).
3. ***INV = CONVERGED ROOTS** (`agentC_*`): DCDFLIB gaminv's early-stop schedule
   RULED OUT (worst of three models); Excel publishes fully-converged near-CR roots
   of its OWN forward (Excel-vs-CR scatter = conditioning-amplified last-bit forward
   rounding, κ-correlation 0.968/0.995). The *INV lane needs no schedule archaeology:
   ported forward + converge-to-last-bit inverter.
4. **erf sawtooth decomposition** (`agentD_*`): single z-linear sawtooth (NOT two
   superposed) threads 100% of the m<1.5 publish intervals (A=0.855±0.008,
   p=3/64 exactly, uniform teeth); past the V-binade the SAME generator's period
   collapses to ≈P/3.5; second generator = a slow BEAT ENVELOPE (weak at m30,
   strong at m20, lag-1 autocorr +0.89 sub-publish). Near-1-constant-multiply
   forms RULED OUT by the noise-free equal-period argument; the generator is a
   z-linear beat with slope-difference ≈2⁻⁴⁷·⁶ (≈21 ULP of g) or a 2⁻⁶⁴-quantum
   accumulation at z-scale. Still unidentified; comb structure at e=−40/−15
   (`answers-b14.json`) consistent with the beat picture.
5. **GRATIO PORTED TO OXFUNC** (merged c71cde5 + corrections fa275e0):
   regularized_gamma_p/q now run the identified kernel with the Excel-variant
   deviations (no a=0.5 erf dispatch; erfc1 via depth-1 recursion) + the agent-B
   staging/normalizer. Corpus: CHIDIST 12→**144**/195 exact (max overflow-class→52),
   GAMMA.DIST 64→**137**/268 (max 38→21). 1507 lib tests green, no pins changed.

## x87-exp last mile (2026-07-17 session 4) — the kernel exp is a STATIC LEGACY CRT

Racing the corrected staging with real chains (`check_gratio_x87.rs`) and ctypes
DLLs settled the exp identity question by elimination:
- **x87 worksheet chain (fFEXP/fFLN) REFUTED for the kernel**: RN53 of the
  64-bit chain ≈ CR at double granularity — all four exp/log combos score
  IDENTICALLY on 516 rows; it cannot produce the one-sided ~38% miss rate.
- **Modern UCRT exp/log ≈ CR** (25/45 = the CR score) — not Excel's.
- **msvcrt / msvcr100 / 110 / 120 as loadable on this host: all 25/45** (they
  forward to near-CR code on modern Windows) — not Excel's.
- **fdlibm e_exp remains the best public proxy (28/45)** — its rounds-low bias
  partially matches. Agent-B's decode stands: every a=2 miss is exactly a
  ∓1-ULP exp(t1) deviation, Excel's exp ≤ nearest (one-sided low).

CONCLUSION: the kernel exp is the 2010-era CRT exp STATICALLY LINKED inside
Excel's binaries (the same pattern as the statically-linked C pow identified in
the bond lanes) — an Intel-style table-based SSE2 exp with ~0.5-1 ULP one-sided
error. Recovery paths: (a) transcribe the msvcr100-era x64 exp op-graph from a
period binary; (b) treat the remaining per-row exp deviations as a measured
±1-ULP table (they are fully determined by the corpus: implied exp(t1) bits are
recoverable per row). The Cephes-igam-form series staging (ans from 1.0,
c/ans <= MACHEP stop, ·ax/a publication) is confirmed as the exact series form.


## bgrat verdict (2026-07-17 session 4, agent F) — tail region is an op-graph wall

Branch census on the fresh b15 battery (126 rows): bpser 59, bup_bpser 37,
bgrat 14, bpser_sym 9, bfrac 4, bup_bgrat 3. Findings:
- **Excel's beta TAIL region (bgrat + bup composites) tracks correctly-rounded
  betainc far better than NSWC-double** (CRc complement-form 12/20 vs 6/20 on
  pure-bgrat rows; bup_bpser CR 36/61 vs double 28/61) while PRESERVING the
  NSWC `0.5+(0.5-w1)` reconstruction — but it is NOT vanilla NSWC bgrat at any
  precision (every precision/eps/term-cap axis inert at 11/31), NOT the
  Excel-variant grat1 wiring, NOT NR-betacf (3/27), and not exactly CR either:
  structured ±4..±7 residuals persist at large-a/small-b and deep tails
  (bfrac rows to −35 vs CR — Excel's own tail carries real relative error).
  Extended precision DEGRADES the large-a rows: the residual is asymptotic
  truncation/formulation error, so Excel's tail is a DIFFERENT expansion —
  the same op-graph-wall class as GAMMALN-core; closure needs the binary's
  routine, not more fitting.
- **Honest held-out caveat**: the bpser beats-CR signature reproduces on the
  F-surfaces (11/12-vs-6/12, 5/5-vs-3/5) but NOT on the deliberately
  boundary-straddling b15 stress battery (30/59 vs 32/59 tied). bpser ==
  NSWC-in-double stands on well-conditioned surfaces; deep-tail rows drift.
- **Gamma-side a<1 wiring VALIDATED**: Excel-variant grat1 (no a==0.5
  dispatch) at prec-64/x87 scores 26/37 on the GAMMA.DIST a<1 rows with ALL
  remaining misses collapsed to ±1 ULP (at a=0.5) — the a<1 path is closed
  to the ±1 level under the extended model.

## Open sub-identifications (recipes)
2. **Internal Γ normalizer**: with the gratio structure pinned, solve per-a for the
   normalizer double that bit-matches each fractional-a slice (interval intersection
   over ~20 x-points each — the slice over-determines it); compare against published
   GAMMA bits, CR-Γ, exp(published GAMMALN), and G3-02's custom-rational hypotheses.
   This is a new measurement window into the G3-02 wall.
3. **Taylor micro-staging**: enumerate on the a=2 slice (Γ=1 exact, gln=0 exact,
   47/80): term-recurrence staging, sum/publish orders, exp/log model per-op. Then a=6.
4. **CF fine detail** (fractional a, x ≥ 1.1 route, and the smalla-series j/rexp
   staging at a<1) — after 1-3.
5. **Beta side = BRATIO (TOMS 708)** — same library, same era: capture BETA.DIST
   discriminating batteries (bpser/bfrac/bgrat branch probes), transcribe bratio.f,
   race. FDIST/TDIST then need only their argument-transform staging.
6. **Wrapper details**: a==1 P-form on x>1 (expm1 vs 1−exp Q-primary — 12 residual
   ±1 rows in (1,3)); CHIDIST df=2/GAMMA.DIST consistency on the Q side.

## Files

- `gen_batteries.py`, `analyze.py` — B1-B4 design + deviation map (829 probes)
- `batch-b5.json`/`answers-b5.json` — integer-dispatch/fractional-a discriminators (207)
- `emulator.py` (mpmath staged models), `check_igamma.rs` (true-x87 Ext80 race —
  ruled the extended family out at stage A)
- `implied_arg.py`, `arg_fit.py`, `sum_decode.py`, `staging_fit.py` — residual decoders
- `cdflib_py.py` (faithful GRATIO + gam1/rlog/rexp/erf/erfc1/gamma_nswc transcription,
  injectable LOG/EXP/GAMMA_FN), `gratio_race.py`, `gratio_detail.py`
- `erf_map.py`, `erf_cody.py`, `answers-erfp/erfcp.json` — the erf sub-lane
- `cdflib/*.f` — the fetched TOMS 654 Fortran sources

## Method notes (durable)

- **Multi-view exact-transform probing** (CHIDIST(2x,2a) vs GAMMA.DIST(x,a,1) vs
  GAMMA.DIST(2x,a,2)) collapses 12 surfaces to one kernel measurement and isolates
  per-function staging for free.
- **Implied-argument decode**: for kernels shaped `sum·exp(arg)`, tiny-x rows make
  `arg_implied = ln(P_excel/sum_true)` measurable to ~2⁻⁵³ absolute; comparing against
  exact rounding residues of candidate stagings identifies WHERE the double-roundings
  sit, one op at a time. The ×a amplification of the ln-residue was the smoking gun.
- **Dispatch discovery by contradiction**: when one op-graph cannot explain two slices
  of the same code path, suspect an argument-value dispatch (a==1 here; the near-integer
  probe a=1+2⁻²⁰ is the cheap decisive test).
- **Branch-differential scoring**: with a multi-branch reference implementation,
  per-branch match rates (91% / 48% / 25%) localize which subroutines are shared vs
  Excel-custom far faster than whole-function scores.
