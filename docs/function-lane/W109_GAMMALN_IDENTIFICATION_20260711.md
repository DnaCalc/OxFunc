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
