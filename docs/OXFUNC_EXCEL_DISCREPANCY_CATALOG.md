# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-07-18` (G3-01 lane-1: distribution pow pinned as POWER's chain
without the 0.5→sqrt shortcut (`excel_pow_chain` landed); WEIBULL.DIST + EXPON.DIST
bodies identified as legacy x87 per-op-double-rounded units and SIGNED OFF held-out
99.983% / 100.000%; see the G3-01 row)
Previous reconcile: `2026-07-17` (G3-01: chopped series-exp identified + landed with the
a==1 dispatch — CHIDIST 152/195, GAMMA.DIST 159/268; *INV lattice inverter +
published-surface stagings landed for CHIINV/FINV/TINV; see the G3-01 row)
Previous reconcile: `2026-07-16` (G3-01 gamma-side substrate identified as plain-double
DCDFLIB GRATIO — the prior x87-extended-CF verdict corrected; see the G3-01 row)
Previous reconcile: `2026-07-11` (live Excel build `20131`; trig `G4-01`
identified and signed off `5425/5425` across all six functions; YIELDMAT `G6-09`
identified and signed off `1250/1250`; NPER `G6-08` identified
and signed off `1286/1286` + `7/7` error rows; XNPV `G6-11` identified
by the W109 calculation-graph search, repaired, and signed off `1530/1530`
numeric + `175/175` error rows; previous reconcile 2026-07-10: stale YIELDDISC
row removed; MMULT/MINVERSE `1x1` publication observations moved to Category 1;
TBILLYIELD association repaired and signed off `2156/2156`)
Last history cleanup: `2026-06-26`

## Purpose

OxFunc targets bit-exact emulation of Excel for every in-scope function and
operator. This file is the single live worklist of every open OxFunc-vs-Excel
discrepancy that OxFunc can evaluate locally: Category 2, context-free cases
under [ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md).

Context-sensitive Category 1 discrepancies do not live here. They live in
`smart-fuzzer/corpus/context_sensitive_catalog/` and are evaluated downstream
through OxCalc -> OxFml -> OxFunc.

Cases where OxFunc already agrees with Excel but Excel itself deviates from the
mathematically-most-accurate result live in
[EXCEL_MATH_DEVIATION_CATALOG.md](EXCEL_MATH_DEVIATION_CATALOG.md), not here.

## Maintenance Rules

1. This is the only open-status tracker for Category 2 discrepancies.
2. Fixed, signed-off, stale, or resolved items are removed from this file.
3. Detailed history, root cause, and evidence stay in `docs/bugs/streams/`,
   `docs/bugs/BUG_STREAM_REGISTER.csv`, git history, and run artifacts.
4. A function may appear more than once when it has distinct discrepancy types.
5. Newly found local discrepancies get a row here immediately, even before a full
   bug stream exists.

## Legend

Severity:
- `STR`: structural mismatch: wrong kind, error code, shape, array behavior, or admission.
- `NUM-L`: numeric, large: materially wrong number or `> ~2` ULP drift.
- `NUM-S`: numeric, small: `≤ ~2` ULP drift.

Maturity:
- `M0 noted`: witnessed, not minimized, no repair.
- `M1 tested`: minimized reproducers or focused tests exist.
- `M2 repair-tried`: repair attempted or repair direction proven, not landed.
- `M3 fixed-unsigned`: fix landed locally and locally green, awaiting live-Excel sign-off.
- `HO downstream`: OxFunc-side handled, blocked on downstream or seam acknowledgement.

## Current Summary

Open Category-2 rows: `22`

| Group | Current rows |
|-------|--------------|
| G1 error-code/domain guards | 0 |
| G2 structural kind/shape/admission | 0 |
| G3 special/statistical numeric exactness | 7 |
| G4 elementary/trig numeric exactness | 4 |
| G5 matrix numeric/shape | 1 |
| G6 financial exactness/solver | 10 |
| G7 comparison/misc semantics | 0 |
| G8 untriaged inbox | 0 |

W108 resolved (bit-exact via the x87 backend, removed from tracking): `EXP`, `LN`, `LOG10`,
`LOG(x, base)`, and `POWER` — 64-bit Excel computes these with the legacy x87 CRT
transcendental chain (`87tran.asm`, CW `0x133F`), reproduced bit-for-bit by
[`crate::excel_numeric::x87`] on the reference x86-64 host. `POWER` (BUG-FUNC-042, signed
off) is the fractional-path `exp(y·ln x)` with the `y<0` reciprocal staging and the
`|y|==0.5→sqrt` special case (715/715 live rows). `EXP`/`LN`/`LOG10`/`LOG` were never catalog
rows (W108-A research findings). Many small-ULP G3/G4 residuals whose kernels call `exp`/`ln`
internally may now be closable by routing those calls through the x87 backend.

## Bounded Reconnaissance Evidence — 2026-07-10

Every open row below now has a stable `G*-NN` reconnaissance id, two exact-input
test cases, live Excel 16.0 build 20131 result bits, and a bounded calculation-path
search map:

- [reconnaissance report](function-lane/DISCREPANCY_RECONNAISSANCE_PASS_20260710.md)
- [48-case corpus](../smart-fuzzer/corpus/discrepancy-recon/catalog-row-recon-v0.json)
- [exact result ledger](function-lane/DISCREPANCY_RECON_RESULTS_20260710.csv)
- [calculation-path map](function-lane/DISCREPANCY_CALCULATION_MAP.csv)

The path entries are black-box hypotheses, not implementation claims. They
explicitly enumerate strict-f64, x87 extended/store-boundary, association,
accumulation, table-constant, and solver-schedule alternatives for future search.

## G1 — Error-Code And Argument-Domain Guards

No current open rows.

## G2 — Structural Kind, Shape, And Admission

No current open rows.

## G3 — Numeric Exactness: Special And Statistical Functions

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G3-01 — BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, TDIST, TINV (+ CONFIDENCE.T, Z.TEST unprobed) | Distribution scalar numeric drift, `1`-`28` ULP. **W109 (2026-07-16): gamma-side substrate IDENTIFIED — the DCDFLIB/NSWC GRATIO branch structure (TOMS 654, DiDonato–Morris), plain SSE2 DOUBLE** — the 2026-07-14 "x87 80-bit extended CF" verdict is WRONG (extended only proved convergence; the true-x87 `check_igamma` race killed the extended family at stage A). Evidence (692-row multi-view corpus, 12 surfaces collapsed: legacy≡modern bit-for-bit, β-scaling transparent, one internal `P(a,x)`): per-branch differential match of a faithful GRATIO transcription — **closed-int `91%`**, asymp 4/6, Temme(a≥15) 5/5, Taylor 48%, erf-routes fail → **`a==1` wrapper dispatch to the exponential CDF (−expm1(−x), proven via `a=1+2⁻²⁰` NOT clean)**; `a==0.5`/half-integer paths use **Excel's OWN near-CR erf/erfc** (NSWC + Cody CALERF ruled out; 352-pt ERF.PRECISE/ERFC.PRECISE ladders captured → same sub-lane closes ERF.PRECISE/GAUSS G4-04/G3-07); fractional-a normalizer = **internal Γ (the G3-02 wall, now MEASURABLE per-a through this window)**; Taylor micro-staging open (a=2 slice, Γ exact, is the clean enumeration target). Beta side **CONFIRMED = BRATIO (TOMS 708)** (2026-07-17 agent sweep: bpser-in-plain-double BEATS correctly-rounded on FDIST/TDIST = literal code identity; accurate-complement argument stagings pinned; one-tail=0.5·two-tail bit-exact; **bgrat is the Excel-custom sub-kernel**, branch battery cached). Gamma-side staging CORRECTED (forward-summed series, 1/a outer; normalizer = CR-Γ±1 ≡ exp(internal lgamma), NOT NSWC gamma — a G3-02 measurement window); a=2 residual = one-sided x87-exp signature, next lever = real fFEXP chain. ***INV = fully-converged near-CR roots of Excel's own forward** (gaminv schedule RULED OUT; κ-correlation 0.97-0.995) — no solver-VM needed, just the forward + converge-to-last-bit. **GRATIO KERNEL PORTED+LANDED** (c71cde5/fa275e0): CHIDIST 12→144/195 exact (catastrophics eliminated), GAMMA.DIST 64→137/268 max 21 ULP, 1507 tests green. **W109 session 5 (2026-07-17): the series exp is CHOPPED — Excel's gser `r = exp(t1)/Γ` publishes a TRUNCATED (round-toward-zero) exp** (floor-of-true scores `38/45` on the implied-exp corpus vs CR 25, fdlibm 28; every real 2010-era MSVC CRT exp refuted by direct 32-bit binary probes incl. msvcr90 9.0.30729 via SxS manifest — the CRT SSE2 exp rounds one-sided HIGH, the mirror of Excel). Chop is CALL-SITE-LOCAL: a==1 wrapper (nearest exp/expm1, now dispatched inside gratio), CF and a<1 paths are NOT chopped. Landed as `exp_rd` (double-double, validated 0/25k vs floor-exp) → **CHIDIST 152/195, GAMMA.DIST 159/268** (b20 held-out gate: +3/111, fresh a-slices). ***INV inverter LANDED**: float-lattice bisection to adjacent doubles (early-stop bisection had `+880k`/`+1.9M` ULP catastrophes at small roots — b14: GAMMA.INV 8→18/60 worst −16, BETAINV 2→4/30 worst +13) + **invert-the-published-surface staging** (CHIINV roots Q directly, held-out-confirmed b19 15/40 vs 6/40; FINV roots the FDIST complement form 0→3/32 with small-p bias collapsed; TINV roots the two-tail surface, residuals −238→±7). erf "fine comb" REFUTED as grid aliasing (b18 matched-resolution scans, 242k rows): the erf last-op fingerprint is now the per-binade phase-gradient; the hunt merges with the internal exp/log identity. Beta tail (b21 discriminator, 127 rows): **family PROVEN = DiDonato–Morris TOMS-708 Eq-9 bgrat** (at k=2 Excel sits +41..+63 ULP from truth yet within ±7 of every Eq-9 realization across 25 rows — intrinsic asymptotic method error), realization exact-matched by NONE (NSWC grat1, GRATIO-sub, Boost 1.35–1.42, Cephes, AS63, NR all fail; chopped-exp inert here). **BRATIO PORTED TO PRODUCTION (2026-07-17 session 6)**: op-for-op from the validated transcription (bit-identity 20,008/20,008 vs the spec), NR continued fraction deleted, accurate-complement wrapper stagings landed (FDIST/TDIST/TTEST + all F/T inverter closures). Held-out b22 (671 fresh rows): 167→**285**/655 exact, worst ±145→126; b21 deep tail: 0/127 worst **8,848** → 4/127 worst 56 (catastrophic tail class eliminated); 422 improved/60 regressed, regressions confined to the bgrat wall. Open: bgrat-tail realization (Eq-9 family, arithmetic unmatched), integer-shape fast-path routing, A/B-bounds staging. **Lane-1 (2026-07-18): distribution pow CLOSED — `exp(RN53(RN64(y·ln x)))`, POWER's chain WITHOUT the 0.5→sqrt shortcut (b24 re-race with the real chain 33,145/33,145; b27D product-staging discriminator 113/113; landed as `excel_pow_chain`). WEIBULL.DIST + EXPON.DIST bodies IDENTIFIED as legacy x87 per-op-double-rounded compilation units (WEIBULL pdf = division-first left-to-right C expression, tree×spill-mask race 1,600/1,600) and SIGNED OFF: b28 held-out 5,999/6,000 (99.983%), b28c 4,000/4,000 (100.000%). Two body classes (plain-SSE2 GRATIO/BRATIO vs x87-DR closed-form) coexist behind the 2010 stats surface.** See [W109_G3-01_GRATIO_IDENTIFICATION_20260716.md](function-lane/W109_G3-01_GRATIO_IDENTIFICATION_20260716.md) + [W109_WALL_CLUES_LEDGER.md](function-lane/W109_WALL_CLUES_LEDGER.md). | NUM-L | M2 | BUG-FUNC-021 / KED-STAT-001 / W109 GRATIO identification |
| G3-02 — GAMMA (+ GAMMALN substrate) | W109 re-scoping (2026-07-11): the row was under-scoped — a fresh 156-row live sweep shows the POSITIVE side is `0/79` exact (up to `1370` ULP at large x) and negatives reach `810k` ULP; the recon corpus had only probed two negative points. Ruled out: current Lanczos log-domain kernel; `x87-EXP(published GAMMALN)` composition (6/79, errors grow with |lnΓ| — Excel uses a HIGHER-precision internal lgamma than published GAMMALN). Same internal extended-lgamma substrate implicated for COMBIN (G4-04 findings). **W109 run-2 identification (2026-07-11, supersedes the Cephes-small claim): GAMMALN `x>=11` IS the plain-double Cephes Stirling tail + UCRT-class log** — `136/139` bit-exact on a 361-row corpus incl. dense grids; the 3 residual rows are sub-ULP internal-CRT-log deltas; the boundary sits in `(10.25, 11.0]`. The `(0,11)` core is a **custom Microsoft rational** (accuracy <=3.5 ULP vs true, exact zeros at 1 and 2, real arithmetic at integer args, `-log(x)+poly` form below 0.5): Cephes-small, fdlibm, UCRT, R/SLATEC, Cody, DCDFLIB, AS245, GSL, NR and Boost are ALL ruled out bit-exactly under every staging (see W109_GAMMALN_IDENTIFICATION_20260711.md). **W109 session 6 (2026-07-18): the (0,11) core is STRUCTURALLY IDENTIFIED — the Cody & Hillstrom SPECFUN DLGAMA skeleton with Microsoft-RE-FIT coefficients** (zero-capture proof from existing data: downward-recurrence identity `excel(x)==double(excel(x+1)−log(x))` bit-exact 32/32 below 0.62 = Cody's PNT68 control flow; `GAMMALN(4)=CR(ln 6)` published verbatim = Cody's band-4 D4-anchor form; full band skeleton mapped, no other internal edges). Published Cody coefficients score b4 231/385 worst 3; ±1–3-ULP hill-climb stalls → decimal-conversion-error hypothesis FALSIFIED — a genuine re-fit on a flat GN manifold (b2 450/1247, b4 238/385 worst 2). b4 miss profile suggests re-fit interval [4,11] (Stirling switch moved from Cody's 12). Best composite forward model 1096/2850 (38.5%) worst 6 ULP. Batteries designed+queued: boundary pinning (1,793) + core recovery (9,642: peel ladders, adjacent-double clusters, recurrence partners, held-out sweep). 1967 tables obtained (user) + validated: verbatim RULED OUT — Excel = the 1967 FORM SET with retuned thresholds and a third Microsoft-refit n=7-class coefficient set. **Round-3 (2026-07-18): structure CLOSED — literal-0.7 threshold pinned to the double, NO Stirling switch at 12 (x≥8 = ONE formula: (x−0.5)log(x)−x+LS2PI + z·w with fdlibm's w1..w6 vector in a non-fdlibm staging, 99.83% worst 2), seam at 8.0 pinned, [4,8) = published SPECFUN P4/Q4 under x87-continuous (worst 1). KERNEL LANDED (commit 223cfa5, agent port bit-identical to reference on 17,003 rows): GAMMALN/GAMMALN.PRECISE 0/79 (worst 1,370) → held-out 316/400 = 79.0% (worst 5).** Open: b1 [0.7,1.5) + b2 [1.5,4) exact coefficients (provisional: 1967-n7/d and gn2/x87-spill per held-out head-to-head; b1 weakest ~31%), the internal-CRT-log ±1 class (7 of 10 x≥8 residuals), [4,8) exact spill pattern; then GAMMA = exp composition (+ sin reflection), COMBIN, G3-01 fractional-a re-race. | NUM-L | M2 | BUG-FUNC-027 C1 / W109_GAMMALN_IDENTIFICATION_20260711.md / smart-fuzzer/work/w109/G3-02-gamma |
| G3-03 — TREND, LINEST, LOGEST (FORECAST closed) | **W109 sweep (2026-07-12): FORECAST/FORECAST.LINEAR IDENTIFIED and PROMOTED** — Excel computes them via the simple centered kernel (forward sums -> means; fused `Σdx·dy`/`Σdx²` loop; publish `a + b·x` intercept form), NOT the LINEST pipeline; `65/65` bit-exact incl. adversarial (1e12 offsets, n=2, near-constant x); OxFunc rerouted off `trend_kernel` with pinned witnesses. SLOPE/INTERCEPT confirmed already bit-exact (share the kernel, `4/4` each). TREND stays on the least-squares pipeline and publishes different bits. **W109 sweep (2026-07-14): the PREDICTION path is bit-exact** — `TREND(x*)=a+b·x*` and `GROWTH(x*)=b·m^x*` reproduce Excel `0` ULP given the coefficients. The ONLY open stage is the LINEST **coefficient** kernel: Excel's slope is a **deterministic 2-ULP-LOW bias** vs any double-centered computation (and 3 ULP below the true value — Excel is LESS accurate than naive double, so extended/x87 move the WRONG way, +3). Confirmed mean-centered family (translation-invariant, scale-equivariant). It's an op-graph wall, BUT the Batch-D corpus is **non-discriminating** (Scxx=exact integer 10 masks the divide/variance path; 120×120 orderings all collapse to +2). Recipe: design-for-divergence capture (non-integer Scxx, cancellation-heavy Scxy, minimal 2/3-point sets, + LINEST ancillary SS/SE outputs to expose the QR R-factor). | NUM-L | M2 | G8 probe / W109 sweep |
| G3-04 — GROWTH | Exponential-regression drift around `11`-`13` ULP on the bounded witnesses. | NUM-L | M1 | recon G3-04 |
| G3-05 — CHISQ.TEST, CHITEST | **W109 sweep (2026-07-12): decomposition unblock proven** — `CHISQ.TEST(o,e) == CHIDIST(S, df)` BIT-EXACTLY for a specific stored double S (tail cancels in the comparison), so the internal statistic is directly measurable without the gamma substrate: the internal statistic is IDENTIFIED as the plain-double ROW-MAJOR `Σ(o-e)²/e` (offset 0 on the two injective-tail tables of a 4-table live set); the CHIDIST tail half remains on the G3-01 substrate, so ALL CHISQ.TEST drift is inherited. F.TEST decomposes the same way via FDIST (see G3-06). | NUM-L | M1 | recon G3-05 / W109 sweep / G3-05-answers-*.json |
| G3-06 — F.TEST, FTEST | **W109 sweep (2026-07-12): decomposed.** `F.TEST(a,b) == 2·FDIST(F, df_hi, df_lo)` BIT-EXACTLY (3 live sets, df to (5,6)); F = larger-var/smaller-var (unbiased, n-1 divisor), 2× exact. Statistic layer identified; one variance-accumulation ULP detail open; the tail is FDIST -> all drift inherited from the G3-01 incomplete-beta substrate. | NUM-S | M1 | recon G3-06 / W109 sweep / G3-06-answers-*.json |
| G3-07 — GAUSS | Standard-normal `Phi(z)-0.5` drift, `2` ULP on the stable witness; needs the erf/CDF substrate (Phase-5 adjacent). PHI is resolved out of this row (W109 2026-07-11: `RN53(RN64(x·x))` -> x87 EXP -> `RN53(RN64(e·RN(1/sqrt(2π))))` with a live-pinned subnormal publication flush; `764/764` answered rows, see the ruled-out ledger and `smart-fuzzer/work/w109/G3-07-phi`). | NUM-S | M1 | recon G3-07 / W109 PHI closure |

## G4 — Numeric Exactness: Elementary And Trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G4-02 — ATANH | **W109 (2026-07-12): region C AND region B PROMOTED — only a 6-row switch band remains.** Region C (`|x| >= ~1.05e-4`, incl. the entire catalog mid-small band and near-1 rows): naive `0.5·ln((1+x)/(1-x))` with the x87 CRT ln, **163/163** (binary64 ratio double-rounding load-bearing; not odd — `ATANH(-0.2)` is 1 ULP off `-ATANH(0.2)`; signed ratio removes the prior copysign divergence). Region B (`|x| <= ~9.0e-5`): Excel's x87 `fyl2xp1` ln1p pair `0.5·(ln1p(x)−ln1p(−x))`, extended temporaries + single store, **175/175** — promoted via `excel_atanh_small`; passthrough emergent; pair exactly odd. Ratio floor lowered `1.25e-4 → 1.05e-4`. OPEN: a narrow switch band at 3 distinct `|x|` values (`9.563e-5, 9.9996e-5, 1.0137e-4`; 6 rows) where Excel is `+-1` ULP from BOTH the x87 pair AND the SSE2 log1p pair — an internal-log1p switch that 3 points cannot disambiguate offline (overfit risk). Needs dense adjacent-double live probes across `[8e-5, 1.3e-4]` both signs to pin the exact switch and the band micro-path. | NUM-S | M3 | W109_ATANH_IDENTIFICATION_20260712.md / atanh.rs |
| G4-03 — ACOTH | **W109 (2026-07-12): two-regime x87 form PROMOTED — strict improvement (35→53/56, 0 regressions, +19 rows).** ACOTH IS exactly odd (`copysign(ACOTH(|x|), x)`) and mirrors ATANH: `|x| < ~3.5` uses the direct ratio `0.5·ln((|x|+1)/(|x|-1))` (x87 CRT ln); `|x| >= ~3.5` uses the reciprocal ln1p pair `0.5·(ln1p(1/|x|)−ln1p(−1/|x|))` via the x87 `fyl2xp1` pair (= `ATANH(1/|x|)`, reusing `excel_atanh_small`). Confirmed: `ACOTH(2)=ATANH(0.5)` bit-for-bit. The earlier "direct signed ratio 40/57 / ln1p 35/57" were both non-odd single-forms; the standalone `log1p(2/(x-1))` is ruled out at large `|x|` (`0/6`). OPEN residual: 3 pair-branch rows (`±5.0` at −1 ULP, `+8.1` at +2 ULP) and the exact switch double — need dense adjacent-double probes near `|x|∈[3,10]`. | NUM-S | M3 | recon G4-03 / W109 ACOTH racer / atanh.rs+acoth.rs |
| G4-04 — COMBIN, COMBINA, FACTDOUBLE, ERF.PRECISE, ERFC.PRECISE | **W109 sweep (2026-07-14): two DISTINCT substrates.** COMBIN = multiplicative product but **NOT bit-exact** (cycle-2 design-for-divergence capture CORRECTED the earlier "bit-exact <2^53" over-claim, which rested on a non-discriminating 7-point corpus where all forms agree below ~2^40): on 16 discriminating `(n,k)` with representable results, Excel matches OxFunc's multiply-first `(acc*(n-k+i))/i` only `6/16`, ratio-first `2/16`, exact-integer `6/16`, and NEITHER `8/16` — Excel sits `1`-`3` ULP BELOW the multiply-first product on larger `n,k`. Plain-double AND x87-prec64 product orderings all score `6/16` → a genuine op-graph residual (PERMUT x87-spill-product family, which IS closed at `702/702` — race COMBIN against that substrate). COMBINA = **`exp(gammaln)` substrate, NOT a product** — CONFIRMED: `COMBINA(20,7)=C(26,7)` returns `657799.9999999999`, 1 ULP BELOW the exact integer `657800` (impossible for a product); reduces to the **GAMMALN/x87 wall** (crack GAMMALN → COMBINA free). FACTDOUBLE bit-exact (7/7); ERF.PRECISE: **W109 2026-07-17 — NO coefficient tables exist: ERF/ERFC.PRECISE ARE the NSWC gratio a<1 branches themselves** (cross-view proof `ERF.PRECISE ≡ GAMMA.DIST(·,½,1)` / `ERFC.PRECISE ≡ CHIDIST(·,1)` 160/160×2; z<0.5 = the 190 DIRECT path `exp(½·ln z²)·g·(1−j)` with **g = 1+gam1(½) evaluated x87-EXTENDED, h pinned `0x3fc06eba8214db6c`**; erfc side = the a<1 CF with unsplit exp argument, proven by messy-grid regression slope +0.95; subnormal publication flush at the far tail; every published implementation + all rational/Padé/Taylor/constant micro-forms ruled out; best true-x87 model `check_erf190` 663/1218, ~92% within ±1 — residual = ONE staging op, recipe + untouched held-out in W109_G3-01_GRATIO_IDENTIFICATION_20260716.md). Recipe: capture `GAMMALN(n+k)/(k+1)/(n)` + `EXP` at the COMBINA arg-triples to formally reduce it to GAMMALN. `±1` ULP drift where OxFunc currently differs. PERMUT resolved out (W109 2026-07-11: ascending x87 spill-loop product, `702/702` live rows, see [`W109_PERMUT_COMBIN_FINDINGS_20260711.md`](function-lane/W109_PERMUT_COMBIN_FINDINGS_20260711.md)). COMBIN: `k -> min(k, n-k)` reduction CONFIRMED; all product-loop, factorial-ratio, reciprocal-multiply, and published-GAMMALN-composition kernels ruled out on a 505-row live corpus — leading hypothesis is an internal extended lgamma/exp substrate (Phase-5 lane). | NUM-S | M2 | BUG-FUNC-027 combinatorial group / recon G4-04 / W109 findings |
| G4-05 — CONVERT | Unit-conversion factor drift, `1` ULP at `CONVERT(1,"m","ft")`; `CONVERT(1,"in","m")` is an exact control. | NUM-S | M1 | recon G4-05 |


The former `G4-01` trig row is signed off and removed (2026-07-11). The W109
search identified the full legacy CRT chains: SIN = `FPREM1(x, FLDPI)` +
parity + `FSIN`; COS = `|x| < 2^-26 -> 1.0` else `FPREM1(|x|, FLDPI/2)` with
quadrant dispatch; TAN = π/2 chain with `-1/tan` (extended) on odd quadrants;
COT/CSC/SEC = double-rounded reciprocals of the published primaries. The
reduction constant is the 64-bit ROM `FLDPI` π — the entire source of the
large-argument drift. Validated `5425/5425` live rows (all six functions,
incl. held-out sweeps and a bit-resolution COS threshold ladder). See
[`W109_TRIG_IDENTIFICATION_20260711.md`](function-lane/W109_TRIG_IDENTIFICATION_20260711.md).
The GAMMA-reflection and Bessel inheritors (G3-02, BUG-FUNC-024) are now
unblocked: re-race their internal trig against `excel_sin`/`excel_cos`.

## G5 — Matrix Numeric And Shape

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G5-01 — MINVERSE | W109 (2026-07-13, **KERNEL LANDED**): algorithm = **Doolittle LU + partial pivot + division multiplier + sequential elimination + per-column unit-vector solves (forward L·y streaming-asc, division-form back-sub), plain double**. The shipping kernel until this landing was **Gauss-Jordan on `[A\|I]`** (`80/159` 3x3, `102/448` 4x4) — the prior "OxFunc already implements the identified algorithm" note was WRONG (it assumed the Python-Doolittle match implied the Rust shipped Doolittle; the Rust actually shipped ruled-out Gauss-Jordan). `inverse_kernel` in `matrix_family.rs` is now swapped to Doolittle and RE-VERIFIED end-to-end through the compiled surface (`eval_surface_value_call` via `matrix_local_eval`): **3x3 `80→150/159`, 4x4 `102→448/448` (perfect)** = +416 cells, `1502/1502` lib tests green, ZERO regressions (old-kernel passing cells are a strict subset). Ruled out for the residual this round: adjugate/cofactor `adj(A)/det` (`51/155`/`85/448`), Gauss-Jordan (`80/159`/`102/448`), the full **32-variant solve-ordering** sweep (fwd/back × stream/sum-then-sub × asc/desc × div/recip — all ≤`150`), and **x87 80-bit extended** registers-with-double-stores (best `110/159`, strictly worse — MINVERSE is plain SSE2 double, NOT a legacy x87 body). The `9` residual `+1`/`+2`-ULP misses are all on ill-conditioned 3x3 (small-determinant integer matrices e.g. [[2,-1,0],[-1,2,-1],[0,-1,2]] det 4, [[1,2,3],[4,5,6],[7,8,10]] det -3, near-identity 1e-8) where Excel lands 1 ULP off the exact representable value in a direction plain-double Doolittle does not — the residual DIRECTION flips (Excel further from exact on tridiag/integer, closer on near-identity), so it is a genuinely different op-graph for these cells, NOT one-extra/one-fewer rounding. Deferred as a targeted decoder probe; 4x4 has zero residual. | NUM-S | M3 | W109 4x4 harvest / G5-01-answers-m4b.json; matrix_family.rs:180 |

`MINVERSE(5)` and `MMULT(5,2)` are deliberately absent here. Nested `TYPE`
evidence proves that their function results remain `1x1` arrays; Excel's final-cell
scalar appearance belongs to the Category-1 worksheet publication/comparator seam.
They are now explicit `publication_shape` rows `CSC-0024`/`CSC-0025` under
`smart-fuzzer/corpus/context_sensitive_catalog/`, with downstream handoff `HO-FN-010`.

## G6 — Financial Exactness, Computation, And Solver

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G6-01 — PMT, PPMT (IPMT, CUMPRINC/CUMIPMT adjacent) | Annuity publication exactness drift. W108 Phase E supersedes the earlier forward-form conclusion: Excel uses the discount arrangement already present in OxFunc, `em=expm1(-n*log1p(r)); v=1+em; pmt=(pv+fv*v)*r/em`. The best tested historical Kahan/x87 helper candidate scores `2285/4040` exact (`56.6%`) and `92.9%` within `1` ULP on the adversarial PMT corpus; the `553`-row `n=1` isolation lane localizes its remaining error to the final rounding of `em`. The fresh mortgage PMT control is exact while first-period PPMT is `1` ULP, sharpening the adjacent recurrence/publication lane. W109 replay (2026-07-11): the full `5,154`-row Phase-E live corpus through current OxFunc scores `3117` exact, `1318` at `1` ULP, `542` at `2-16`, `177` above — PMT lanes are dominated by `<=4` ULP publication drift while IPMT/PPMT/CUM schedule rows reach catastrophic drift, including sign-of-zero/branch rows (`IPMT-139` publishes `-0` vs a finite Excel value; `ISO-EM-T0-109/122` publish tiny nonzero vs Excel exact `0`). Rollup: `smart-fuzzer/runs/w109-phase-e-replay/rollup.json`. Row STAYS OPEN. | NUM-L→NUM-S | M2 | BUG-FUNC-015 / KED-FIN-001 / W108 / recon G6-01 / W109 replay |
| G6-02 — ACCRINT | Residual `1` ULP on the now-pinned `us30360` triple-edge witness: issue `43565`, first interest `43647`, settlement `43905`, rate `0.05`, par `1000`, frequency `2`, basis `0`. | NUM-S | M1 | BUG-FUNC-030 / recon G6-02 |
| G6-03 — YIELD | W109 (2026-07-13): the dive-plan premise "forward kernel already bit-exact, YIELD is 100% schedule" is FALSE. The **forward PRICE kernel is not bit-exact**: `pcomp` computes `base^(off+k)` via `base.powf` (Rust powf = `exp·ln` even for integer exponents), but Excel uses the C-runtime `pow` integer special case = **binary exponentiation**. IDENTIFIED + held-out validated (25 live-Excel PRICE points / 5 bonds: **15/15 ident + 10/10 held-out** for binexp-integer + powf-fractional + coupons-first). Ruled out: naive repeated-multiply (5/10, breaks exp≥4), powf/x87·log integer (6/10), getPrice_ redemption-first order (2/5). Residual after the forward fix = **plateau publication**: corrected PRICE is flat at exactly 95 across a ~20-ULP yield band; Excel publishes a specific point (catalog `0x…9983`, par exactly `0.05`). COUPLED: fixing pcomp alone + current bisection regresses par 6→~40 ULP — land the forward fix AND the schedule together. Writeup: `work/w109/G6-solvers/YIELD_PRICE_FORWARD_KERNEL.md`. | NUM-L | M2 | BUG-FUNC-031 / recon G6-03 |
| G6-04 — ODDFYIELD | W109 (2026-07-14): **ODDFPRICE forward kernel FIXED** — it shared the `base.powf` integer-exponent bug; `excel_bond_pow` (binexp) added to `oddfprice_kernel` → ODDFPRICE now **bit-exact 5/5** on the live US-30/360 integer-exponent ladder (`oddfprice_us30360_integer_exponents_bit_exact_vs_excel`), 1505/1505 green, no regression. So ODDFYIELD's ~`3e5` ULP drift is now confirmed **pure schedule/publication** — shares the YIELD solver-VM (G6-03), iterate-publication wall. | NUM-L | M1 | BUG-FUNC-032 / recon G6-04 |
| G6-03b — PRICE | W109 (2026-07-14) **FIX LANDED**: PRICE drifted on **on-coupon** bonds (integer discount exponents) — `base.powf` uses `exp·ln` even for integers, but Excel uses the C-runtime `pow` integer special case = **binary exponentiation**. Added `excel_bond_pow` (binexp integer / powf fractional) and routed `price_kernel` through it via `pcomp_disc(..,binexp=true)`; YIELD's solver + DURATION keep the legacy `powf` path (decoupled — the YIELD forward fix is coupled to its unsolved schedule). Now **bit-exact on all 25 live-Excel points / 5 bonds** (`price_binexp_matches_excel_ladders`), regression `yield_unchanged_by_price_fix` pins YIELD; `1504/1504` lib green. **Cycle-3 sign-off sweep (2026-07-14) BLOCKED — found a material latent bug:** across 90 live-Excel PRICE points spanning all 5 bases × 6 bonds, OxFunc is bit-exact for bases 0 (30/360) and 4 (EU 30/360), near-exact for basis 1 (act/act, one `+1` ULP on 1 point) = `56/90`, but **materially wrong for basis 2 (Actual/360), off ~`0.035`-`0.042`, and basis 3 (Actual/365), off ~`0.007`-`0.014`** (dollars/cents, not rounding). PRE-EXISTING (fractional exponents → untouched by the binexp fix), never caught (no PRICE row tested act/360-365). Root cause is `pcomp`'s day-count for the Actual/360-365 bases (`dc` returns `360/freq`/`365/freq` for the coupon length `e` while `dd` uses actual days → wrong discount exponent `off=dsc/e`); shared by YIELD/DURATION. → **new sub-item G6-03d (below); the binexp/30-360 part remains M3-clean.** PRICEMAT confirmed already bit-exact; DURATION/MDURATION → G6-03c. | NUM-S | M3 | `bond_core_family.rs:excel_bond_pow` / work/w109/G6-solvers/YIELD_PRICE_FORWARD_KERNEL.md |
| G6-03d — PRICE / YIELD / DURATION (Actual/360 & Actual/365) | W109 cycle-3 (2026-07-14) NEW: `pcomp` computes a **materially wrong** price for **basis 2 (Actual/360)** (~`0.035` low) and **basis 3 (Actual/365)** (~`0.013` low) — e.g. `PRICE(2020-09-20,2025-01-01,0.06,0.03,103,2,2)` Excel `114.5887` vs OxFunc `114.5504`. The coupon-period length `e` uses the 30/360-style `360/freq`/`365/freq` while `dsc`/accrual use actual days, so the discount exponent `off=dsc/e` and the accrual fraction are inconsistent for the actual-day bases. Cycle-4 reverse-engineering (bond 44094 b2/b3) RULED OUT the simple fixes: **E=actual gives the act/act price** (114.6012, not the act/360 114.5887); money-market `exponent=days/(360|365 /freq)` gives 114.37/114.58; hybrid `off=DSC/Eactual` + accrual `A/(yb/freq)` gives 114.57/114.59 — none match. Note OxFunc's `dc` (E=`360/freq` for Actual360) faithfully mirrors the F# ExcelFinancialFunctions `coupDays`, so either EFF's PRICE is itself off for act/360-365 OR N/DSC differ — needs a systematic reverse-engineering of all four (N, DSC, E, A) for these bases. Shared by YIELD + DURATION. | NUM-L | M1 | W109 cycle-3/4 PRICE sweep / c3_price_out.json |
| G6-03c — DURATION, MDURATION | W109 (2026-07-14) NEW: DURATION drifts `1`-`2` ULP vs live Excel on on-coupon bonds — e.g. `DURATION(44013,44562,0.05,0.05,2,0)` OxFunc `…dbe8` vs Excel `…dbe9` (1 ULP), `…@0.08` OxFunc/binexp both `…4488` vs Excel `…448a` (2 ULP). The `excel_bond_pow` (binexp) discount fix is NECESSARY (fixes the 0.05 case) but NOT sufficient — a residual remains in the duration-specific arithmetic (the `Σ t·cash/disc` weighted sum, `t=(off+k)/freq`, or the `w/dirty` division), not the discount powers. Needs its own staging identification, then apply binexp + the residual fix together. `MDURATION = DURATION/(1+yld/freq)` inherits it. | NUM-S | M1 | W109 DURATION capture / work/w109/G6-solvers/dur_capture_out.json |
| G6-05 — RATE | Solver residual `586` ULP on the mortgage witness and `72` ULP on a one-period identity whose mathematical root is `0.1`. | NUM-L | M1 | BUG-FUNC-009 bit-parity / W103 / recon G6-05 |
| G6-06 — IRR | Irrational-root solver residuals remain: `80` ULP and `14096` ULP on the bounded witnesses. | NUM-L | M1 | BUG-FUNC-028 out-of-stream / recon G6-06 |
| G6-07 — CUMPRINC | The full-schedule control is exact; the half-schedule witness is `1` ULP, localizing the current evidence to boundary-sensitive accumulation. | NUM-S | M1 | recon G6-07 |

The former `G6-09` YIELDMAT row is signed off and removed (2026-07-11). The
W109 search identified x87 spill-loop arithmetic with the PUBLISHED formula's
association — `term1 = (1 + DIM/B·rate) - term2` with `term2` reused — not the
F#-style left chain OxFunc had ported. Validated `1250/1250` bit-exact on live
build `20131` (bases 2/3 sweep incl. held-out) and both former catalog
witnesses (bases 1/0) through the production day-count logic. See
[`W109_YIELDMAT_IDENTIFICATION_20260711.md`](function-lane/W109_YIELDMAT_IDENTIFICATION_20260711.md).

The former `G6-08` NPER row is signed off and removed (2026-07-11). The W109
search identified the same legacy x87 spill-loop signature as XNPV — every
assignment double-rounded, both logs the x87 worksheet `ln`, denominator on a
double-rounded `1+rate` — plus three newly pinned lanes: no epsilon small-rate
branch (tiny rates take the main path, `#DIV/0!` once `1+rate == 1`),
`NPER(0,0,..)` is `#DIV/0!`, and the zero-rate linear branch is double-rounded.
Validated `1286/1286` numeric + `7/7` error rows on live build `20131`. See
[`W109_NPER_IDENTIFICATION_20260711.md`](function-lane/W109_NPER_IDENTIFICATION_20260711.md).

The former `G6-11` XNPV row is signed off and removed (2026-07-11). The W109
calculation-graph search identified the full staging — `RN53(RN64(1+rate))`
base, the full worksheet POWER kernel per term (integer binexp dispatch
included), `RN53(RN64(value/pow))` term, forward per-step-stored x87
accumulation — plus a previously unknown guard (`rate <= 0`, including `-0.0`,
publishes `#NUM!`; OxFunc formerly accepted `(-1, 0]`). Validated
`1530/1530` numeric + `175/175` error rows on live build `20131`, including
held-out and metamorphic sweeps. See
[`W109_XNPV_IDENTIFICATION_20260711.md`](function-lane/W109_XNPV_IDENTIFICATION_20260711.md).

The former `G6-10` TBILLYIELD row is signed off and removed. A `2,156`-case
settlement × duration × price sweep first reproduced `308` one-ULP failures,
then reached `2156/2156` exact after changing the expression association from
`((100-pr)/pr*360)/days` to `((100-pr)/pr)*(360/days)`. See
[`CANDIDATE_CLOSURE_SWEEP_20260710.md`](function-lane/CANDIDATE_CLOSURE_SWEEP_20260710.md).

The former `YIELDDISC(44013,44562,95,100,0)` row was stale: the rate-first
formula repair already landed, the in-crate bit target is pinned, and a fresh
three-way replay on Excel 16.0 build 20131 is `all_bit_exact` (OxFunc = F# =
Excel). The row and bead `oxf-pzav` were retired on 2026-07-10.

## G7 — Comparison And Misc Semantics

No current open rows.

## G8 — Untriaged Inbox

No current open rows.

New smart-fuzzer `mixed_or_open` findings land here first, then move to G1-G7
or the context-sensitive catalog after triage.

## Pointers

- Category boundary and policy: [ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md)
- Context-sensitive Category 1 catalog: `smart-fuzzer/corpus/context_sensitive_catalog/`
- Severity vocabulary and comparison policy: `CHARTER.md` §4.1 and smart-fuzzer `Get-StandardSeverityClass`
- Transferable lessons: [OXFUNC_FIX_LEARNING_LOG.md](OXFUNC_FIX_LEARNING_LOG.md)
- Detailed bug evidence and history: `docs/bugs/streams/BUG-FUNC-*.md`
- Bit-exact Excel comparison plumbing: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`
- How to reproduce and repair a row (fixer quick-start): [OXFUNC_DEVIATION_FIXER_QUICKSTART.md](OXFUNC_DEVIATION_FIXER_QUICKSTART.md)
