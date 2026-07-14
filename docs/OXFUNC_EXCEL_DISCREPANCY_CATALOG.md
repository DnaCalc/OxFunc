# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-07-11` (live Excel build `20131`; trig `G4-01`
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

Open Category-2 rows: `21`

| Group | Current rows |
|-------|--------------|
| G1 error-code/domain guards | 0 |
| G2 structural kind/shape/admission | 0 |
| G3 special/statistical numeric exactness | 7 |
| G4 elementary/trig numeric exactness | 4 |
| G5 matrix numeric/shape | 1 |
| G6 financial exactness/solver | 9 |
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
| G3-01 — BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, TDIST, TINV (+ CONFIDENCE.T, Z.TEST unprobed) | Distribution scalar numeric drift, `1`-`28` ULP. **W109 sweep (2026-07-14): shared substrate identified — ONE incomplete-γ/β continued-fraction kernel** (CHIDIST=`Q(df/2, x/2)` gamma; FDIST/TDIST=regularized incomplete beta `I`). Excel runs it in **x87 80-bit EXTENDED precision** (mpmath prec-64==prec-113 → the CF fully converges; residual is pure finite-precision accumulation, NOT an ITMAX/EPS cap). OxFunc's **plain-double CF** is the bug: it diverges `5`-`6224` ULP, worst at boundaries via the `1−P` complement (CHIDIST_1_10 gap `6224`), where Excel (extended `1−P` reconstruction) is `0`-`3` ULP. FIX DIRECTION: replace OxFunc's double CF with x87-extended accumulation + extended complement → collapses the `6224`/`56`/`50`-ULP catastrophes to Excel's sub-`20`-ULP regime; bit-exact then needs the x87 gser/gcf/betacf op-graph (wall). Excel-vs-truth: gamma `≤1`(+bias), beta-F `≤5`, beta-T `≤19` (sensitivity-weighted sawtooth, peaks mid-probability). *INV inherit via the solver-VM. Recipe: gser↔gcf branch sweep + CF-depth test + isolate the x87 exp/ln/loggamma prefactor. | NUM-L | M2 | BUG-FUNC-021 / KED-STAT-001 / W109 sweep |
| G3-02 — GAMMA (+ GAMMALN substrate) | W109 re-scoping (2026-07-11): the row was under-scoped — a fresh 156-row live sweep shows the POSITIVE side is `0/79` exact (up to `1370` ULP at large x) and negatives reach `810k` ULP; the recon corpus had only probed two negative points. Ruled out: current Lanczos log-domain kernel; `x87-EXP(published GAMMALN)` composition (6/79, errors grow with |lnΓ| — Excel uses a HIGHER-precision internal lgamma than published GAMMALN). Same internal extended-lgamma substrate implicated for COMBIN (G4-04 findings). **W109 run-2 identification (2026-07-11, supersedes the Cephes-small claim): GAMMALN `x>=11` IS the plain-double Cephes Stirling tail + UCRT-class log** — `136/139` bit-exact on a 361-row corpus incl. dense grids; the 3 residual rows are sub-ULP internal-CRT-log deltas; the boundary sits in `(10.25, 11.0]`. The `(0,11)` core is a **custom Microsoft rational** (accuracy <=3.5 ULP vs true, exact zeros at 1 and 2, real arithmetic at integer args, `-log(x)+poly` form below 0.5): Cephes-small, fdlibm, UCRT, R/SLATEC, Cody, DCDFLIB, AS245, GSL, NR and Boost are ALL ruled out bit-exactly under every staging (see W109_GAMMALN_IDENTIFICATION_20260711.md). Remaining work (Phase-5b): bisect the boundary, identify the internal CRT log from Stirling bracketing constraints, recover the custom band coefficients via error-curve edge mapping + integer-relation fitting; then GAMMA = exp composition (+ sin reflection), COMBIN, G3-01 re-race. | NUM-L | M2 | BUG-FUNC-027 C1 / W109_GAMMALN_IDENTIFICATION_20260711.md / smart-fuzzer/work/w109/G3-02-gamma |
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
| G4-04 — COMBIN, COMBINA, FACTDOUBLE, ERF.PRECISE, ERFC.PRECISE | **W109 sweep (2026-07-14): two DISTINCT substrates.** COMBIN = float multiplicative product, **bit-exact through 2^53** (all representable results exact; drifts only `±2` ULP past 2^53 where the result is unrepresentable anyway → effectively closed). COMBINA = **`exp(gammaln)` substrate, NOT a product** — proven: `COMBINA(20,7)=C(26,7)` returns `657799.9999999999`, 1 ULP BELOW the exactly-representable integer `657800` (impossible for any product); it reduces to the **GAMMALN/x87 wall** (crack GAMMALN → COMBINA free). FACTDOUBLE bit-exact (7/7); ERF.PRECISE needs the erf substrate. Recipe: capture `GAMMALN(n+k)/(k+1)/(n)` + `EXP` at the COMBINA arg-triples to formally reduce it to GAMMALN. `±1` ULP drift where OxFunc currently differs. PERMUT resolved out (W109 2026-07-11: ascending x87 spill-loop product, `702/702` live rows, see [`W109_PERMUT_COMBIN_FINDINGS_20260711.md`](function-lane/W109_PERMUT_COMBIN_FINDINGS_20260711.md)). COMBIN: `k -> min(k, n-k)` reduction CONFIRMED; all product-loop, factorial-ratio, reciprocal-multiply, and published-GAMMALN-composition kernels ruled out on a 505-row live corpus — leading hypothesis is an internal extended lgamma/exp substrate (Phase-5 lane). | NUM-S | M2 | BUG-FUNC-027 combinatorial group / recon G4-04 / W109 findings |
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
| G6-03b — PRICE | W109 (2026-07-14) **FIX LANDED**: PRICE drifted on **on-coupon** bonds (integer discount exponents) — `base.powf` uses `exp·ln` even for integers, but Excel uses the C-runtime `pow` integer special case = **binary exponentiation**. Added `excel_bond_pow` (binexp integer / powf fractional) and routed `price_kernel` through it via `pcomp_disc(..,binexp=true)`; YIELD's solver + DURATION keep the legacy `powf` path (decoupled — the YIELD forward fix is coupled to its unsolved schedule). Now **bit-exact on all 25 live-Excel points / 5 bonds** (`price_binexp_matches_excel_ladders`), regression `yield_unchanged_by_price_fix` pins YIELD; `1504/1504` lib green. Awaiting a broader live-Excel PRICE sign-off sweep before removal. PRICEMAT confirmed already bit-exact (interest-at-maturity single division, no discount powers); DURATION/MDURATION need binexp too but carry an additional residual — spun to G6-03c. | NUM-S | M3 | `bond_core_family.rs:excel_bond_pow` / work/w109/G6-solvers/YIELD_PRICE_FORWARD_KERNEL.md |
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
