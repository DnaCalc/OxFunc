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


## Bulk oracle engine + mass tooth sweep (2026-07-17 session 5)

**Run-W109BulkBatch.ps1 landed**: recalc-sheet bulk capture, same ProbeBatch CLI
and shared OracleCache, validated 100.000% bit-identical on 1,831 gate rows
(ERF.PRECISE/CHIDIST/GAMMA.DIST) and byte-identical cache records. Throughput:
**8,800 probes/s** Excel-compute (~400x the per-batch-startup baseline); 114,688
ERF.PRECISE probes captured in 67s wall (`batch/answers-b16.json`). Invariants
preserved: scalar-cell path, args via Value2 references only (never decimal
formula text), Formula2R1C1 for dotted names.

**Mass tooth sweep** (16k-step ladders per binade + two zoom windows,
`dump-b16.txt` via the generalized check_erf190 dump): the erf last-op comb has
EXACT near-dyadic per-binade periods with wildly non-monotonic density —
p(2^-20) = 2^-33 exactly, p(2^-15) = 3*2^-29 exactly, e=-25 nearly quiet
(7 teeth/16k vs 5,960 at e=-20), amplitudes 0.75-1.5 ULP; the bisected
p(2^-30) = 2.996*2^-36 stays the one non-dyadic reading (continuous
measurement vs grid-quantized). No smooth or single-table generator law fits;
the structure suggests dependence on z's leading mantissa bits with
binade-dependent index width. Next: grid-free continuous period measurement
per binade (bisection pairs, now cheap), and correlate tooth positions
against leading-bit tables of z, w, and x simultaneously.


## Fine-comb measurement (2026-07-17 session 5b) — envelope aliasing exposed, bit-grids refuted

Dense per-binade scans at ~p/1000 resolution (81,920 probes in 20s via the bulk
engine, `answers-b17.json`/`dump-b17.txt`) show the ladder-scale "periods" were
ALIASED ENVELOPES: the true primary comb is ~1000x finer (e=-30: p = 4.30e-14
not 4.36e-11; e=-20: 5.42e-14; e=-15: 3.14e-12; e=-25: 1.67e-12; e=-40:
1.04e-20), with tooth amplitudes still 0.55+ ULP. The fine periods are
NON-dyadic (0.757-0.954 x 2^k) and the anchor-integrality test against z-bit,
3z-bit, w-bit and x-bit grids returns mean|frac| = 0.25 = the uniform-random
baseline over 16k+ teeth: **ALL leading-mantissa-bit-table generators are
REFUTED**. The earlier bisected p(2^-30) = 4.36e-11 and the m30 3/64-grid
teeth were beat-envelope structure of this fine comb (explaining their exact
9-period consistency: bisection tracked the envelope, which is real but
secondary). Relative fine-periods are wildly non-monotonic across binades
(7.9e-8, 4.5e-8, 4.6e-5, 3.6e-5, 9.4e-9 at e=-15..-40).

IMMEDIATE next step (cheap, no captures): re-dump the same answers under a
DIFFERENT model config (toggle w_dbl / zz_dbl) and compare tooth positions -
model-side teeth move or vanish, Excel-side teeth are invariant; this
partitions the fine comb between Excel's generator and residual model-side
staging before any further theory.


## Config re-dump partition (2026-07-17 session 5c) — the fine comb IS Excel-side

Differential dumps of the same 82k answers under three model configs (base 304,
w_dbl 432, zz_dbl 305; `dump-b17-{base,wdbl,zzdbl}.txt`):
- **w_dbl toggle: ZERO effect** in every binade (tooth sets identical) — the
  w-spill axis is irrelevant to the comb.
- **zz_dbl toggle: e=-15/-20/-30 tooth sets 100% invariant** => the fine comb
  there is EXCEL'S GENERATOR, not model staging. At e=-25 ~30% of base teeth
  were model-side (5713 -> invariant subset ~4006) and at e=-40 the zz_dbl
  config ADDS ~2000 artifact teeth (base's 1093 all survive => base is clean).
- Consequence: the base-config fine-period table stands for -15/-20/-30/-40;
  re-derive e=-25 from the invariant subset. Part of the cross-binade
  non-monotonicity was model contamination at -25.

The Excel-side fine comb (non-dyadic periods, no bit-grid anchoring, dense at
0.55+-ULP amplitude) is now the confirmed, isolated target for the erf last-op
generator hunt.

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

## b18 matched-resolution scans (2026-07-17 session 5) — the "fine comb" was ALIASING; phase-gradient is the real fingerprint

Battery b18 (242,474 ERF.PRECISE rows, bulk engine): every binade scanned at
THREE matched relative step scales (2e-8 / 2e-9 / 2e-10), anchored at the b17
windows. Result, uniform across all five binades:

- **No resolved comb exists at any scale.** The "median gap" is always 2-5 GRID
  steps at every scale — it rescales x10 whenever the grid does (e.g. s25:
  8.0e-8 -> 8.0e-9 -> 8.0e-10 rel across the m/f/u scans). Every fine-period
  previously tabulated from dense scans (6.4e-8 / 4.4e-8 / 5.6e-5 / 3.2e-5 /
  6.8e-9, and their non-monotonicity) was an ALIASING ARTIFACT of the scan
  grid. The b17 "config re-dump partition" tooth sets remain valid as
  point-sets, but their gap statistics carry no period information.
  METHOD CAUTION (durable): a period estimated from a dense scan is only real
  if it is reproduced at a 10x finer grid; otherwise it is the grid echoing
  its own step through the miss density.
- **The real structure is the phase gradient.** Using the model's extended
  value position within its ULP (phase), miss probability rises monotonically
  toward the rounding boundary:
    e=-25/-30/-40: P(miss) ~ 10% at phase 0 -> ~48% at |phase|=0.5 (identical
                   profiles across all three binades);
    e=-15/-20:     ~36% -> ~52% (much flatter = larger perturbations).
  Misses at phase~0 exist, so the driver reaches >=0.5-1 ULP; the equidistributed,
  spatially-incoherent pattern is the signature of per-row last-bit differences
  in an internal transcendental (exp/log op-graph), NOT a staging/association
  difference (those are spatially coherent). Envelope-scale density variation
  is real (s20 windows: 29-55% miss density).
- CONSEQUENCE: the erf last-op lane MERGES with the static-CRT-exp lane. The
  scoring instruments for candidate exp/log op-graphs are (a) exact-match rate
  on b18 (242k rows), (b) the per-binade phase-gradient profile, (c) the 45-pt
  implied-exp corpus. Period-matching is dead.

Files: batch-b18.json / answers-b18.json / dump-b18-base.txt / analyze_b18.py /
period_rederive.py / excel_side_period_table.json (superseded by this section).

## *INV converged-root landing (2026-07-17 session 5)

Landed in production (validated on the b14 corpora, held-out b19 captured
separately):

- `bisect_inverse` (special_math_common.rs): early-stop 4*EPS bisection
  replaced by FLOAT-LATTICE bisection to adjacent doubles, publish hi
  (f(lo) < p <= f(hi)); order-preserving i64 key + i128 midpoint, sign-safe.
  b14 effect: GAMMA.INV 8->18/60 exact, worst +880,380 -> -16 ULP;
  BETAINV 2->4/30, worst +1,910,580 -> +13 ULP (residual = the pre-BRATIO
  beta forward, a separate lane); publication-rule race: hi 18/60 vs
  closest 17/60 vs lo 7/60 (gamma side) — hi retained.
- `chisq_inv_rt_kernel` (chi_f_t_family.rs): CHIINV now inverts the PUBLISHED
  right-tail surface Q directly (negated-forward convention) instead of P at
  1-p. The 1-p staging carries a systematic -5..-33 ULP bias (rounding loss in
  1-p); Q-direct: 10->16/60 exact, residuals collapse to +-1..5 (worst -91 at
  one deep-tail row).
- `gamma_inv_kernel`: upper bracket now extends by doubling until
  f(hi) >= p (lattice invariant).
- Racer: `check_inv.rs` (calc_graph_racer) — races early-stop vs lattice
  {hi, lo, closest} x staging spaces on the b14 corpora. GAMMA.INV x-space vs
  z-space indistinguishable at beta=1 (all b14 gamma rows); b19 includes
  beta != 1 discriminator rows + FINV/TINV probes of the same
  invert-published-surface principle.

Full oxfunc_core suite green (16 test binaries) after the change.

## CHOPPED-EXP IDENTIFICATION (2026-07-17 session 5) — the kernel's series exp is TRUNCATED toward zero

The decisive break in the "static legacy CRT exp" wall came in two steps:

1. **Real-binary CRT sweep (agent G) closed the entire Microsoft hypothesis
   space.** A 32-bit harness called exp() in the genuine DLLs — msvcr90
   9.0.30729 (the exact Office-2010 CRT generation, loaded via a VC90 SxS
   activation-context manifest), msvcr100 (2010-03-18 binary), 110, 120,
   msvcrt — plus the x87 fallback path (_set_SSE2_enable(0)) and bit-faithful
   transcriptions of the AMD K8 32-entry and x64 64-entry table exps
   (ReactOS libm_sse2 exp.asm == Open64 libacml_mv, AMD provenance pinned).
   ALL REFUTED: the 32-bit SSE2 CRT exp is one Intel-lineage __libm_sse2_exp
   rounding one-sided HIGH (0..+1 vs CR) — the mirror image of Excel; x87 and
   x64-AMD are CR-identical on the corpus. Best public proxy stays fdlibm
   28/45. Excel's exp is in {CR, CR-1}, one-sided LOW, CR-1 on 20/45 rows.
2. **20/45 = 44% ~ half is the truncation signature**: CR-1 exactly when CR
   rounded up. floor(true exp) scores **38/45** (vs CR 25, fdlibm 28) —
   rd_exp and RN64-then-RZ53 agree on all 45 args. The one-sided-low
   "rounds-low CRT exp" story was a truncated PUBLICATION all along; the
   earlier x87-chain refutation had only raced nearest publication.

**Call-site localization** (full 692-row corpora, emulator + replica races):
the chop lives ONLY at the gser series r = exp(t1)/G call site.
- a==1 wrapper rows LOSE under chop (its exp/expm1 are nearest);
- continued-fraction rows LOSE (cf/P 10 lost, 0 gained);
- a<1 (190/200-path) rows LOSE;
- a>=1 series rows GAIN: production-staging replica 148->174/306 training,
  b20 held-out (fresh a in {1.75,2.25,3.25,4.5,5.5,8,12}, df-truncation
  corrected) 65->68/111, worst -22 -> -19. Fractional-a margins are capped by
  the normalizer (exp of internal lgamma, G3-02 lane), not by the exp.
Per-call-site rounding differences inside one compiled function are now a
PATTERN in this kernel (cf. the a>=1 double vs a<1 extended log staging).

**LANDED in production** (special_math_common.rs):
- `exp_rd(x)`: Tang-style 64-entry table exp in double-double (~2^-100),
  directed truncation before exact 2^m scaling; constants generated
  programmatically as bit patterns (a hand-converted table entry error was
  caught by regenerating — never hand-convert constant tables). Validated
  0 mismatches vs mpmath floor-exp on 25k points incl. the +-708/709 edge
  bands; |m|>1022 and the subnormal sliver fall back to nearest exp.
- statement-20/534 call sites now pass r_series = exp_rd(t1)/G to the series
  arm and r = exp(t1)/G to the CF/asymptotic arms.
- **a==1 dispatch landed inside gratio**: (P, Q) = (-expm1(-x), exp(-x))
  nearest (the identification's wrapper dispatch; a pinned CHIDIST(1,2)
  witness enforces it against the chop).
Corpus: CHIDIST 148 -> **152**/195, GAMMA.DIST 151 -> **159**/268 (from
12/195 and 64/268 pre-campaign). Full suite green (1604).

Remaining gamma-side residual: the internal approximation BEHIND the chop
(7/45 rows where Excel's pre-truncation value crosses a boundary that
floor(true) doesn't — an approximation with ~2^-56-ish error), the fractional-a
normalizer identity (G3-02), and the erf/beta sub-kernels. The erf 190-path
exp is NOT the chopped one (b18 phase evidence both-sided; a<1 rows lose
under chop) — the erf last-op hunt continues in the phase-gradient frame.

## *INV published-surface principle extended (b19 held-out)

b19 (fresh rows, never raced): CHIINV Q-direct CONFIRMED held-out (15/40 vs
6/40 for P at 1-p, same systematic negative bias on fresh rows). The same
invert-the-published-surface staging decisively improves FINV (0/32 -> 3/32,
small-p bias -60 -> +2) and TINV (residuals -4..-238 -> mostly +-1..7):
LANDED for f_inv_rt_kernel (roots f_dist_rt's accurate complement form) and
t_inv_2t_kernel (roots t_dist_2t's surface). GAMMA.INV z-space vs x-space:
+4 rows on 48 discriminators (beta=3) for z-space — a hint, below promotion
bar; x-space retained. BETAINV/CHISQ.INV inverter-limited no longer; forward
error dominates.

## Batteries added this session

- b18 (242,474 ERF.PRECISE rows): matched-relative-resolution scans; killed
  the period-table premise (see b18 section above).
- b19 (320 rows): held-out *INV + FINV/TINV staging probes.
- b20 (112 rows): held-out gamma series (fresh a); chopped-exp gate.
- b21 (127 BETA.DIST rows): beta-tail discriminator battery (agent-H spec:
  GRATIO-substitution vs Boost small-b-large-a series vs CR) — scoring in
  flight.

## Beta tail: family PROVEN, realization OPEN (b21, 2026-07-17 session 5)

The b21 discriminator battery (127 live BETA.DIST rows: deep tail a in
{16..200} x b in {0.05..0.95} x y=2^-k, plus fractional-a composites) settled
the beta-tail family question and left the exact realization open.

- **FAMILY PROVEN = DiDonato-Morris TOMS-708 Eq-9 bgrat expansion.** Decisive
  signature: at k=2, a=118/200, Excel sits +41..+63 ULP from the TRUE value
  yet within +-7 ULP of every Eq-9-family realization across 25 rows. A
  63-ULP co-deviation from truth tracked to +-7 across the family is intrinsic
  truncated-asymptotic method error, not rounding coincidence. NSWC driver +
  0.5+(0.5-w1) reconstruction retained; composites are NSWC bup-shaped (not
  Boost's ibeta_a_step: Boost off +6/+14/+22 there).
- **NO exact realization** among: NSWC-double grat1 (4/127, max 56),
  GRATIO-sub nearest/chopped-r/chopped-all (8-10/127, max 37), Boost 1.35-1.42
  (6/127 but best <=4-ULP where the asymptotic error dominates: k=2/3 median 3
  max 8), Cephes 2.8, AS 63, NR betacf, or any routing/cap/eps forcing.
- **Chopped-exp is INERT on the beta tail** (+1/+2 exact over nearest): the
  gamma-side truncation does NOT transfer to the beta r=a*exp(t1) site. The
  transcendental-model axis (cr vs x87) is also inert here.
- Term-cap unidentifiable: the j-series eps-converges in <20 terms; caps
  20..60 are bit-identical.
- Residual bounded/structured (+-30). Next-cycle axes (agent-H): the r/u/h
  PREFIX op-graph (Excel may normalize via its own gamma, not NSWC
  algdiv/gam1) and the inner Q(b,u) kernel's exact arithmetic.

**CRITICAL STATE FACT**: OxFunc production `regularized_beta`
(special_math_common.rs) is a **Numerical-Recipes Lentz continued fraction**,
NOT a BRATIO port. The entire BRATIO identification (bpser plain-double =
literal code identity beating CR on FDIST/TDIST; bup/bgrat/bfrac/basym
routing; accurate-complement stagings) lives only in the Python emulator
(agentA_bratio.py). So the beta side has had NO production landing analogous
to the gamma GRATIO port. **The clear next material win is porting BRATIO to
replace the NR continued fraction** — the bpser bulk (non-tail) region would
gain the same way CHIDIST/GAMMA.DIST did, while the bgrat tail stays the
identified-family-unresolved wall documented here. This is a kernel-scale
lane (bpser, bup, bgrat, bfrac, basym, algdiv, gam1, gamln, brcmp1), not a
drop-in; it should be its own task with its own held-out gate.

Files: agentH_b21.py, agentH_b21_analysis.py, agentH_b21_out.txt,
answers-b21-beta.json; earlier agentH_{routing,routing2,cephes,boost,
gratio_tail,as63,report}.py + agentH_src_* sources.
