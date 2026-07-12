# W109 Phase-5: GAMMALN identification — Stirling region identified, small-x core scoped as custom

Live oracle: Excel 16.0 build 20131, x86-64 AMD host. Corpus: 361 distinct
positive GAMMALN answers (93 legacy + 274 fresh probes in
`smart-fuzzer/work/w109/G3-02-gamma/answers-r2.json`: dense grids on (0,1)
step 0.02, (1,3) step 0.04, (3,9.5) step 0.11, (9.5,13.5) step 0.05, plus
near-1/near-2 cancellation ladders and boundary pairs).

## Identified: the asymptotic region (x ≥ ~11)

Excel's GAMMALN for `x ≥ 11` is the **plain-double Cephes `lgam` Stirling
tail with a platform-class (UCRT) log** — no x87 anywhere:

```text
q  = (x - 0.5)*log(x) - x + 0.91893853320467274178      (plain double, left-to-right)
p  = 1.0/(x*x)
t  = polevl(p, A5)          A5 = Cephes lgam A[] table (5 coefficients)
r  = q + t/x
```

Score: **136/139 bit-exact on every row x ≥ 11** (11–1030). The three
residual rows (x = 11.3 +1 ULP, 12.75 −1 ULP, 91.608499 +1 ULP) are
attributable to Excel's statically-linked CRT `log` differing from the
host UCRT `log` by <1 ULP on those arguments (each flip requires only a
~0.3–0.7 ULP shift of `log(x)`; no FMA/association/coefficient variant of
the surrounding arithmetic moves them, and the same formula with fdlibm's
log scores worse, 132/139).

Notes:
- The earlier "83/83 x87-staged Stirling" result (commit 8b57a49 era) was
  an over-fit: 8 free store-mask bits against 83 rows ≥ 13.19. The dense
  10–13.5 probes kill every x87 staging; plain double + platform log is
  the unique survivor and needs **zero** free bits.
- The x = 11.0 witness (`GAMMALN(11)=0x402e357590954d16`) is 1 ULP ABOVE
  the correctly-rounded ln(3628800) — impossible for any log staging of an
  exact-product reduction, natural for Stirling truncation. This was the
  probe that broke the small-path assumption.

## Boundary

The small/Stirling switch sits in **(10.25, 11.0]** — Stirling matches
nothing below 10.25 (0/25 on [8,10)) and everything from 11.0 up.
Exact threshold pinned next by adjacent-representable bisection (~40 live
probes).

## The small-x core (0 < x < ~11): custom Microsoft rational

Facts established from the dense corpus:
- Accuracy vs true lgamma: ≤ 3.5 ULP everywhere; sub-ULP on (3,10);
  ±3 on (0.9,3); ±2 on (0,0.9). NOT correctly-rounded quality → genuine
  plain-double rational arithmetic, NOT extended/x87.
- `GAMMALN(1) = GAMMALN(2) = 0` exactly (anchored or special-cased).
- `GAMMALN(3)` is 1 ULP above CR(ln 2) → the region does real arithmetic
  even at integer points (no `log(z)+0` early-out).
- Below ~0.5 the form is `-log(x) + (accurate poly)` with a platform-class
  log (fdlibm-structure scores 32/34 there; the poly's own error is
  magnitude-masked).
- On (0.68,4), where a core polynomial is bit-exposed, **every public
  implementation fails at the coincidence baseline (~50%)**.

Ruled out bit-exactly (plain double + FMA + x87 spill + extended stagings,
platform/fyl2x/fdlibm logs as applicable):

| candidate | best score | killed by |
|---|---|---|
| Cephes lgam small path (B/C rational), all 128+ stagings, ln in/out stores | 52 rows unexplainable | dense (0,3) grid |
| Recurrence-to-Stirling, T ∈ 4..16, log-of-product AND sum-of-logs, all stagings | 174/361 max | anti-correlates below 11 (14/220 at T=11 plain) |
| fdlibm `__ieee754_lgamma_r` (full, incl. fdlibm log, FMA variants) | 203/361 | naked-poly bands (0.9,3): 9/40, [2,3): 7/28 |
| UCRT `lgamma` (ucrtbase.dll direct) | 192/361 | scattered ±3 |
| R / SLATEC Fullerton (gamcs Chebyshev, lgammacor) | 177/361 | (0,10) gamma-then-log profile wrong |
| Cody SPECFUN DLGAMA (P1/P2/P4, C-array) | 178/361 | all bands baseline |
| DCDFLIB / NSWC gamln + gamln1 (TOMS 708) | 174/361 | (0.8,2.25]: 6–12 ULP misses |
| AS 245 (Macleod ALNGAM) | 74/361 | 12-digit coefficients → 1e-9 relative; Excel is ~1e-16 |
| GSL Lanczos g=7 | 63/361 | everywhere |
| NR gammln (Lanczos g=5) | 1/93 | 45k ULP (prior session) |
| Boost lgamma_small_imp (64-bit variant used for double) | 131/280 (<15) | all bands baseline |
| Cephes `gamma()` (P/Q + stirf) for worksheet GAMMA | 8/79 positive | growing ULP with x |
| GAMMALN = log(published GAMMA bits) | 37/79 | near-1 rows |
| UCRT tgamma for worksheet GAMMA | 13/156 | everywhere |

Conclusion: the (0,11) core is a **custom Microsoft implementation**
(consistent with the Excel 2010 "function accuracy improvements" rewrite)
— likely a banded rational re-fit that no public source matches. The
worksheet GAMMA errors grow with x like `exp(lgam-with-absolute-error)`,
so GAMMA is expected to be `exp` over this same internal lgamma; COMBIN
and the G3-01 distributions sit on the same substrate. **Identifying this
one custom kernel remains the gate for the whole gamma family.**

## 2026-07-12 live-probe harvest — the (1,2) FORM identified

Live pipeline re-validated (GAMMALN(11)=0x402e357590954d16, GAMMALN(1)=GAMMALN(2)=0)
and a 1016-point dense batch harvested over (0.1, 11.5) into
`answers-dense1.json`. Error-vs-true(mpmath) ULP profile by band:

| band | ULP err | note |
|---|---|---|
| (0.1,0.5) | ±1 | near-correctly-rounded (`-log(x)+poly` regime) |
| (0.5,1.0) | ±2 | |
| (1.0,1.5) | −3..+5 | polynomial EXPOSED |
| (1.5,2.0) | −5..+2 | polynomial EXPOSED |
| (2.0,3.0) | ±4 | exposed |
| (3.0,6.0) | ±1 | accurate again |
| (6.0,10) | −3..+1 | |
| (10,11.5) | ±2 | Stirling region |

The error curve is a SMOOTH ±few-ULP oscillation, NOT piecewise — the apparent
"edges" are just the approximation error oscillating, so there is no hidden
sub-band structure in (1,3).

**KEY: the (1,2) form is `lgamma(1+s) = s·(s−1)·P(s)`** (s = x−1), anchored at
the exact zeros s=0 (x=1) and s=1 (x=2). Extracting `P(s)=Excel(1+s)/(s(s−1))`
and fitting a polynomial: residual falls monotonically with degree (deg4
4.7e-5 → deg12 1.8e-11), and the recovered coefficients MATCH the true-lgamma
expansion — **P(0) = 0.57721566… = γ (Euler–Mascheroni)** to 6.7e-16, P(1)≈
−0.2452512, etc. So Excel's (1,2) core is an accurate series-form approximation
`s(s−1)·P(s)` (NOT an arbitrary minimax rational); the ±5 ULP vs true is the
double-precision Horner evaluation of the degree-high P, not coefficient error.
This is the recoverable form. Note P(0)=γ means the earlier "recovered constant
γ is a true-function value not a fingerprint" caveat is resolved: γ IS the
leading coefficient of the (1,2) core.

## 2026-07-12 recovery attempts on the (1,2) core — the hard wall confirmed

With the `s(s−1)·P(s)` form and 249 dense (1,2) points, three recovery routes
were tried and all fail bit-exact — the core is a genuine custom high-order
approximation, not directly recoverable from ±5-ULP-rounded evaluations:
- **Truncated true Taylor** of P(s): catastrophic (1e14 ULP) — P's Taylor around
  0 converges only slowly toward s=1 (boundary singularity of the numerator's
  series), so Excel is NOT a Taylor truncation.
- **Recurrence** `lgamma(x)=lgamma(x−1)+log(x−1)` for (2,3): RULED OUT — residual
  drifts +5..+9 ULP (would be ~±1 if real), and band error SHRINKS with x
  ((3,4) ±1 vs (1,2) ±5) whereas recurrence would accumulate error. Each region
  has its own direct approximation.
- **High-precision polynomial fit** (mpmath, monomial in s): converges but slowly
  — D=16 still 47666 ULP off (D=15 → 207922). **Rational P/Q fit** (m,m): worse /
  ill-conditioned (≥6e12 ULP). Neither a modest polynomial nor a modest rational
  matches; the exact stored coefficients are not recoverable by direct fitting to
  the rounded data.

**Assessment:** the (1,2) core is the evidence doc's "custom Microsoft
implementation, no public source" — recovering it bit-exact needs (a) much denser
adjacent-double probes to tighten per-coefficient constraints, (b) the exact
evaluation ansatz (variable/center/staging), and (c) LLL/PSLQ against that ansatz,
or a match to an obscure historical lgamma. This is a dedicated multi-session
sub-campaign; deferred. Form + these negatives are the load-bearing progress.

## 2026-07-12 recovery harness + workflow — monomial plateau, form is structural

Built the recovery harness `smart-fuzzer/work/w109/G3-02-gamma/lgamma_recover.py`
(load dense band, high-precision fit of `R` in `lgamma(x)=(x-1)(x-2)·R(x)`, staged
double evaluation, bit-exact scoring, per-coordinate ULP refine, PSLQ constant-id).
Harvested a dense (1,2) constraint set (`answers-g12dense.json`, 1468 points: fine
0.0008 grid + adjacent-double clusters at 7 anchors).

Findings on the monomial ansatz:
- Best staging is `(x-1)·((x-2)·R)`, center 1.5, and it converges with degree
  (D16 27/1468 worst 4377 → D20 **564/1468 worst 54** → D22 571 → then conditioning
  noise). A scaled/Chebyshev-conditioned fit (u=(x-c)/0.5) is needed past D16.
- **PLATEAUS at ~571-578/1468 (~39%)**; the per-coordinate ±ULP refine only reaches
  578 — so the residual is **structural, not coefficient rounding**. The monomial
  `(x-1)(x-2)·Horner(R)` captures the function shape but is NOT Excel's exact
  evaluation structure.

A recovery workflow (6 parallel ansatz families: rational N/D, Cody sub-band split
at 1.5, compensated/double-double + Estrin evaluation, PSLQ-anchored coefficients,
exact published Cephes/fdlibm/Boost branches, alternative variable/no-factor) is
running with adversarial held-out verification of any family that beats the plateau.

## 2026-07-12 recovery workflow — 6 families, convergent hard wall + family ID

A 6-agent recovery workflow (rational, Cody sub-band split, compensated/dd eval,
PSLQ-anchor, exact published impls, alt-variable) with adversarial verification.
**None close** (best 601/1468 ≈ 41%), but they CONVERGED on the cause:

- **Precision monotonically HURTS** (compensated-eval): plain double 571 →
  double-double 460 → x87-extended-throughout 462. Falsifies extended-precision;
  Excel uses plain-double sloppy per-op rounding. `R=excel/((x-1)(x-2))` is
  provably NOT a polynomial (fit residual flat ~2.3 ULP, degree 20→32).
- **Exact-real ceiling of ANY smooth form ≈ 478/1468 — BELOW sloppy-double 571**
  (rational). So Excel's bits encode a specific finite-precision OP-FINGERPRINT,
  not a cleaner function. x87 per-op double-rounded (spill) poly reaches 579 —
  slightly better than plain, still a plateau.
- **FAMILY IDENTIFIED (published-impls):** fdlibm `__ieee754_lgamma_r` `1≤x<2`
  branch (centered at the lgamma minimum `tc=1.46163214496836224576`, 3-way
  parallel poly `p=z·p1−(tt−w·(p2+y·p3))`, `val=tf+p`) lands **within 5 ULP at
  EVERY point**; Boost `lgamma_small_imp<64>` `(x-1)(x-2)(Y+P/Q)` within 6.
  Excel is a minimax rational in THIS family. But refitting Excel's coefficients
  into fdlibm's exact op-tree still caps ~225/750 with a stubborn 4-ULP residual
  that ULP-refine cannot remove.
- A **universal ~5-ULP wall** appears in every form/cut/precision → the residual
  is Excel's **exact per-operation evaluation graph** (op-order + per-op rounding),
  NOT the coefficients, function form, variable, or subbanding.

**Verdict:** GAMMALN(1,2) is the "custom Microsoft implementation" — a minimax
rational in the fdlibm/Boost family, reproducible only by matching Excel's exact
internal op-graph. Not recoverable from observed bits by form/coefficient fitting.
Hardest lane in W109; DEFERRED. The one untested experiment for cycle-back: take
fdlibm's tc-centered op-tree structure EXACTLY and apply the x87 per-op double-round
(spill) model while jointly refitting coefficients — the only precision×op-graph
combination the workflow did not isolate. Harness: `lgamma_recover.py`;
data: `answers-g12dense.json` (1468 pts). NB: `identify_constant` tol was 1e-25
(can't fire on doubles) — widen to ~1e-15 if PSLQ-anchoring is retried.

## 2026-07-12 tree-aware probing — TREE largely identified, op-graph is the floor

Following the 1990s-x87 + Gaussian-quadrature-probing insight: instead of blind
fitting, probe at exactly-representable offsets and read the tree directly.

**Findings (the tree is now substantially pinned):**
- **(b) x87 on fdlibm's tc-tree does NOT help**: fdlibm's exact tc-branch scores
  177/750 under the realistic x87 model (80-bit within expressions, round at each
  `double` temp) vs 179/750 plain — same worst-5. Excel's tree ≠ fdlibm's tree.
- **Exact-offset peel** `x = 1 + m·2⁻ᵏ` (so x−1 is an exact power of two): the value
  peels term-by-term. `excel(1+2⁻ᵏ)/2⁻ᵏ → −0.5772156649015329` to ~52 bits ⇒
  **D₁ = correctly-rounded −γ** (digamma(1)); Excel near x=1 tracks the TRUE function
  to ~1 ULP.
- **Breakpoint at x=1.5** (Cody's classic split): the window-median error flips sign
  — `+1/+2` ULP on [1.1,1.5], zero at 1.5, `−1/−2` on [1.55,2] — the fingerprint of
  two separate minimax approximations meeting at 1.5 (rational in x−1 below, x−2 above).
- **Cody form `lgamma(x)=(x−1)·[(x−1)·N/D + D₁]`, D₁=−γ, refit N/D on [1,1.5]**: best
  229/718 at (5,5), worst 15. Still a plateau.

**Why every form plateaus (~32–41%):** fitting to ±5-ULP-rounded output cannot recover
coefficients to better than ~5 ULP (fdlibm's PUBLISHED coeffs give worst-5 everywhere;
any REFIT from Excel's rounded bits gives worst≥5-15). So the residual is the rounding
floor of the fit itself. Closing it needs Excel's EXACT stored coefficients + exact
op-order — information not present in the output bits. LLL can't break the floor because
setting up the lattice requires the exact op-graph, which is the unknown.

**Tree now known:** fdlibm/Cody-family minimax rational, split at 1.5, D₁=−γ, near-1 ≈
true function. **Unknown (the wall):** the exact N/D coefficients + per-op evaluation graph
on each half. Realistic closure paths: (1) obtain Excel/MS's actual lgamma source constants;
(2) a joint op-graph × coefficient search far larger than fitting. DEFERRED — best-characterized
hard lane in W109.

## Next steps (Phase-5b)

1. **Pin the Stirling boundary** by live bisection in (10.25, 11.0].
2. **Identify the internal CRT log** from the 3 residual Stirling rows +
   targeted probes: for x ≥ 11 the result is `(x-0.5)*log(x)+…`, so each
   GAMMALN answer brackets `log(x)` to sub-ULP; harvest a few hundred
   bracketing constraints and race UCRT-log revisions / SSE2 minimax
   variants against them.
3. **Recover the custom small-x kernel**:
   a. Map band edges: dense ULP-error-vs-true curve; edges appear as
      error-curve discontinuities; bisect each edge to the exact double.
   b. Within each band, hypothesize `prefix(x) * (Y + P(t)/Q(t))` forms
      (anchored at the exact zeros) and recover coefficients by integer
      relation / LLL on high-density answer bits (the racer gets a
      `fit-rational` mode).
4. Only after 1–3: promote `gammaln` kernel, compose GAMMA (+ reflection
   via identified excel_sin for negatives), COMBIN, then re-race G3-01.

Working artifacts: `check_gammaln_split.rs`, `check_gammaln_recur.rs`,
`check_gammaln_fdlibm.rs`, `check_gamma_cephes.rs` (racer bins);
`band_matrix.py`, `boost_small.py` (work dir).
