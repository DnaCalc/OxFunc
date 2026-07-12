# W109 GAMMALN — Published lgamma Coefficient Catalog & Run Plan

Reference material for the GAMMALN (0,11) custom-core identification (catalog
G3-02). Purpose: Excel's core is a minimax rational in the Cody/fdlibm family;
recovering it bit-exact requires the EXACT stored coefficient table (fitting from
rounded output is rounding-floor-limited — proven below), so this catalogs the
published coefficient sets to test verbatim.

## Target fingerprint (extracted from live-Excel probing, 2026-07-12)

- **Structure:** minimax rational, **split at x=1.5**, near-1 linear coefficient
  **D₁ = correctly-rounded −γ** (digamma(1)); form `lgamma(x) = xm·(D + xm·P(xm)/Q(xm))`
  with `xm = x−1` (lower half) / `x−2` (upper half). Read directly from exact-offset
  probes `x = 1+m·2⁻ᵏ` and the error-median sign-flip at 1.5.
- **Cody & Hillstrom (1967) ranges are [0.5,1.5], [1.5,4.0], [4.0,12.0]** — the 1.5
  breakpoint matches EXACTLY, and (1.5,4) as one rational is consistent with the
  observed band errors ((1.5,2) ±5 shrinking to (3,4) ±1). Likely full structure:
  3 Cody rationals over (0.5,12) + reflection below 0.5 + **Stirling above ~11**
  (already identified, plain double + platform log, 136/139).
- **Arithmetic:** plain-double sloppy per-op rounding (extended precision HURTS).
- **The wall:** the exact P/Q coefficients + per-op evaluation graph. A ~5-7 ULP
  gap separates Excel from every published table so far → Excel uses its OWN
  coefficient set within this structure.

## Catalog (tested + candidate)

| Source | Era | Structure | Split / D₁ | Applicability | Tested → best on (1,2) |
|--------|-----|-----------|------------|---------------|------------------------|
| **Cody & Stoltz — netlib SPECFUN `algama`/`dlgama`** | 1967→2002 | `xm·(D+xm·P/Q)`, XNUM deg7 / XDEN deg8-monic, per range | **1.5 / −γ** | **HIGHEST** (exact structural match) | **365/1468 worst-7** (double); 342 x87 |
| **Cody & Hillstrom 1967 — lower-precision variants** | 1967 | same, fewer terms / different degree | 1.5 / −γ | **HIGH** (netlib is the 18-digit variant; MS may use a 16-digit or different-degree one → different coeffs) | **NOT TESTED** |
| **Hart, Cheney & al. "Computer Approximations" (1968) — LGAM index tables** | 1968 | rational, many JCAM precision indices | varies | **HIGH** (the standard 1990s coefficient source; many vendors copied Hart tables verbatim) | **NOT TESTED** |
| fdlibm / SunPro `__ieee754_lgamma_r` (= glibc, musl, BSD) | 1993 | tc-centered t[0..14] + u/v rationals | tc≈1.4616, 1.23, 1.73 | MEDIUM (family; different tree) | 384 (double), 382 x87 |
| Boost `lgamma_small_imp<64>` | 2000s | `(x-1)(x-2)(Y+P/Q)` | 1.5 | MEDIUM | 374 (worst-6) |
| Cephes `lgam` (Moshier) | 1980s | `log(z)+z·P/Q` small path | — | LOW | 106 |
| SLATEC Fullerton `DLNGAM` / R `lgammafn` | 1970s/90s | Chebyshev `gamcs` + Stirling `lgammacor` | — | LOW | 177 |
| DCDFLIB / NSWC `gamln`+`gamln1` (TOMS 708) | 1990s | rational | — | LOW-MED | 174 |
| AS 245 (Macleod 1989) `ALNGAM` | 1989 | 12-digit rational | — | LOW (precision too low) | 74 |
| NR `gammln` (Lanczos g=5) | 1988 | Lanczos | — | RULED OUT | ~1/93 |
| GSL `gsl_sf_lngamma` (Lanczos g=7) | 1990s | Lanczos | — | RULED OUT | 63 |

Note: musl / glibc / BSD `lgamma` are all the SunPro fdlibm code — no separate test needed (= fdlibm 384).

## Exact coefficients on file (netlib Cody/Stoltz `dlgama`, double)

`SQRTPI = 0.9189385332046727417803297`  ·  `PNT68 = 0.6796875`

Range **0.5≤x≤1.5** — `xm1=(x-0.5)-0.5` (x≥0.6796875, else `xm1=x`, `corr=-log(x)`);
`result = corr + xm1·(D1 + xm1·XNUM/XDEN)`; `XNUM=0,XDEN=1` then 8× `XNUM=XNUM·xm1+P1[i]; XDEN=XDEN·xm1+Q1[i]`:
- `D1 = -5.772156649015328605195174e-1`  (= −γ)
- `P1 = [4.945235359296727046734888, 2.018112620856775083915565e2, 2.290838373831346393026739e3, 1.131967205903380828685045e4, 2.855724635671635335736389e4, 3.848496228443793359990269e4, 2.637748787624195437963534e4, 7.225813979700288197698961e3]`
- `Q1 = [6.748212550303777196073036e1, 1.113332393857199323513008e3, 7.738757056935398733233834e3, 2.763987074403340708898585e4, 5.499310206226157329794414e4, 6.161122180066002127833352e4, 3.635127591501940507276287e4, 8.785536302431013170870835e3]`

Range **1.5<x≤4** — `xm2=x-2`; `result = xm2·(D2 + xm2·XNUM/XDEN)`:
- `D2 = 4.227843350984671393993777e-1`  (= 1−γ)
- `P2 = [4.974607845568932035012064, 5.424138599891070494101986e2, 1.550693864978364947665077e4, 1.847932904445632425417223e5, 1.088204769468828767498470e6, 3.338152967987029735917223e6, 5.106661678927352456275255e6, 3.074109054850539556250927e6]`
- `Q2 = [1.830328399370592604055942e2, 7.765049321445005871323047e3, 1.331903827966074194402448e5, 1.136705821321969608938755e6, 5.267964117437946917577538e6, 1.346701454311101692290052e7, 1.782736530353274213975932e7, 9.533095591844353613395747e6]`

Range **4<x≤12** — `xm4=x-4`; `result = D4 + xm4·XNUM/XDEN`:
- `D4 = 1.791759469228055000094023`
- `P4 = [1.474502166059939948905062e4, 2.426813369486704502836312e6, 1.214755574045093227939592e8, 2.663432449630976949898078e9, 2.940378956634553899906876e10, 1.702665737765398868392998e11, 4.926125793377430887588120e11, 5.606251856223951465078242e11]`
- `Q4 = [2.690530175870899333379843e3, 6.393885654300092398984238e5, 4.135599930241388052042842e7, 1.120872109616147941376570e9, 1.488613728678813811542398e10, 1.016803586272438228077304e11, 3.417476345507377132798597e11, 4.463158187419713286462081e11]`

Stirling `C = [-1.910444077728e-03, 8.4171387781295e-04, -5.952379913043012e-04, 7.93650793500350248e-04, -2.777777777777681622553e-03, 8.333333333333333331554247e-02, 5.7083835261e-03]`

(fdlibm t[0..14] on file in `smart-fuzzer/work/w109/G3-02-gamma/fdlibm_x87.py`.)

## Key negative — DO NOT re-fit

Fitting the P/Q coefficients from Excel's rounded output — even with Cody's EXACT
op-graph fixed — plateaus: refit on [1,1.5] gives 185/719, no better than netlib's
own published 180/719. Reason: each observation is a ±5-7-ULP-rounded double, so
coefficients cannot be recovered to the <1-ULP precision needed. **Only verbatim
published tables can close it.** (Harness: `lgamma_recover.py`; data:
`answers-g12dense.json` 1468 pts, `answers-peel.json` 399 exact-offset pts.)

## Run plan

- **R1 — Hart "Computer Approximations" LGAM tables (highest priority).** Fetch the
  LGAM index coefficient tables (JCAM 5401-series etc.). For each precision index,
  evaluate in the Cody op-graph (and the Hart-specified form) on the dense (1,2)
  set, plain-double + x87, scored per half. Hart is the coefficient source most
  likely reused verbatim by a 1990s vendor.
- **R2 — Cody & Hillstrom 1967 lower-precision variants.** Fetch the paper's tables
  (it publishes several degrees / precisions; netlib is the 18-digit one). Test each
  degree variant verbatim — Excel may target ~16 digits with a shorter rational.
- **R3 — extend the range check.** Test netlib Cody P2/Q2 (the 1.5<x≤4 rational) on
  fresh (2,4) probes and P4/Q4 on (4,12): if Excel matches Cody's structure there
  better than on (1,2), it narrows which range's coefficients differ. Harvest (2,4)
  and (4,11) dense sets.
- **R4 — half-mixing / vendor tables.** If any table closes ONE half (e.g. [1,1.5]),
  fix that half and hunt the other independently. Also test IMSL/NAG-derived open
  reimplementations and the DCDFLIB gamln1 (0.8,2.25] rational verbatim.
- **R5 — precision model per candidate.** For the winning coefficient family, sweep
  the evaluation precision model (plain double / x87 per-op spill / x87 sub-expr with
  double temps) — the op-graph is the last ULP.
- **Gate:** a candidate "closes" at ≥1465/1468; verify held-out on `answers-dense1.json`.

Scope note: all candidates are PUBLISHED numerical-library coefficients — no Excel
decompilation. If none of the published tables match, Excel used in-house-fitted
coefficients and the lane is not closable by this route.

## Extended search findings (2026-07-12) — the published-table route is closed

- **`GAMMALN.PRECISE` == legacy `GAMMALN`** on modern build 20131: identical bits at
  every probed point, both ±3 ULP from true. Excel 2010 introduced `.PRECISE`, but on
  this build BOTH resolve to the SAME custom Cody-family implementation — no cleaner
  modern library (Boost/fdlibm) shortcut exists, and it is NOT correctly-rounded.
- **Modern-rewrite angle closed:** the legacy/precise split does not give a second,
  matchable target.
- **Reliably machine-readable published tables are exhausted** and none match:
  netlib Cody 365, fdlibm 384, Boost 374, Cephes 106, DCDFLIB 174, SLATEC/R 177,
  AS245 74, NR/GSL Lanczos ruled out.
- **Book/paper sources are unreliable for bit-exact work:** Hart "Computer
  Approximations" (1968) is not on the Internet Archive as full text (Open Library /
  HathiTrust catalog only); the Cody & Hillstrom 1967 paper mirror refused connection;
  and OCR of 16-digit coefficient tables would corrupt the very digits that must be
  perfect. Abramowitz & Stegun / Hastings give only ~8-digit approximations (too coarse).
- A literature note surfaced that **Cody's lgamma coefficients for the (1,2) interval
  are "unpublished."**
- **"Both are the new function" verified & pursued:** GAMMALN≡GAMMALN.PRECISE means we
  ARE probing the modern (2010-rewrite) function, so the modern-library candidates were
  tested against the actual bits: **exact Boost.Math source `lgamma_small_imp` = 374/1468
  (worst-6)** — Boost's structure matches the fingerprint perfectly (split 1.5, `(z-1)(z-2)(Y+P/Q)`,
  Y+R(0)=γ, reflected upper half) but its coefficients are not Excel's. **System lgamma also
  ruled out:** Windows UCRT `ucrtbase.dll` lgamma = 335/1468, Python `math.lgamma` = 19/1468 —
  Excel does not delegate to the OS runtime. Best of ANY implementation on the dense (1,2)
  set: fdlibm 384/1468 (26%).

**Conclusion:** Excel's GAMMALN (= GAMMALN.PRECISE) is the Cody structure (split 1.5,
D₁=−γ, `xm·(D+xm·P/Q)`) with an **unpublished coefficient set** — most likely Microsoft's
own minimax re-fit or a proprietary-vendor table. It is NOT recoverable from output bits
(fitting is rounding-floor-limited) and NOT matched by any public table. The lane is
**closed to reverse-engineering by this route**; genuine closure would need the actual
Microsoft/vendor source constants. DEFERRED as characterized-and-blocked. The value banked:
the tree is fully identified (Cody family, 3 rationals over (0.5,12) + Stirling), the exact
netlib Cody constants are on file as the nearest published reference, and the fitting/precision
dead-ends are documented so the lane is not re-attacked by the same routes.
