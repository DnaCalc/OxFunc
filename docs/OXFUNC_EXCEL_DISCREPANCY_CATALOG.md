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

Open Category-2 rows: `19`

| Group | Current rows |
|-------|--------------|
| G1 error-code/domain guards | 0 |
| G2 structural kind/shape/admission | 0 |
| G3 special/statistical numeric exactness | 7 |
| G4 elementary/trig numeric exactness | 4 |
| G5 matrix numeric/shape | 1 |
| G6 financial exactness/solver | 7 |
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
| G3-01 — BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, TDIST, TINV (+ CONFIDENCE.T, Z.TEST unprobed) | Distribution scalar numeric drift, currently quantified at `1`-`28` ULP on representative live-Excel witnesses. | NUM-S | M2 | BUG-FUNC-021 / KED-STAT-001 / `oxf-simj` |
| G3-02 — GAMMA (+ GAMMALN substrate) | W109 re-scoping (2026-07-11): the row was under-scoped — a fresh 156-row live sweep shows the POSITIVE side is `0/79` exact (up to `1370` ULP at large x) and negatives reach `810k` ULP; the recon corpus had only probed two negative points. Ruled out: current Lanczos log-domain kernel; `x87-EXP(published GAMMALN)` composition (6/79, errors grow with |lnΓ| — Excel uses a HIGHER-precision internal lgamma than published GAMMALN). Same internal extended-lgamma substrate implicated for COMBIN (G4-04 findings). **W109 run-2 identification (2026-07-11, supersedes the Cephes-small claim): GAMMALN `x>=11` IS the plain-double Cephes Stirling tail + UCRT-class log** — `136/139` bit-exact on a 361-row corpus incl. dense grids; the 3 residual rows are sub-ULP internal-CRT-log deltas; the boundary sits in `(10.25, 11.0]`. The `(0,11)` core is a **custom Microsoft rational** (accuracy <=3.5 ULP vs true, exact zeros at 1 and 2, real arithmetic at integer args, `-log(x)+poly` form below 0.5): Cephes-small, fdlibm, UCRT, R/SLATEC, Cody, DCDFLIB, AS245, GSL, NR and Boost are ALL ruled out bit-exactly under every staging (see W109_GAMMALN_IDENTIFICATION_20260711.md). Remaining work (Phase-5b): bisect the boundary, identify the internal CRT log from Stirling bracketing constraints, recover the custom band coefficients via error-curve edge mapping + integer-relation fitting; then GAMMA = exp composition (+ sin reflection), COMBIN, G3-01 re-race. | NUM-L | M2 | BUG-FUNC-027 C1 / W109_GAMMALN_IDENTIFICATION_20260711.md / smart-fuzzer/work/w109/G3-02-gamma |
| G3-03 — TREND, LINEST, LOGEST (FORECAST closed) | **W109 sweep (2026-07-12): FORECAST/FORECAST.LINEAR IDENTIFIED and PROMOTED** — Excel computes them via the simple centered kernel (forward sums -> means; fused `Σdx·dy`/`Σdx²` loop; publish `a + b·x` intercept form), NOT the LINEST pipeline; `65/65` bit-exact incl. adversarial (1e12 offsets, n=2, near-constant x); OxFunc rerouted off `trend_kernel` with pinned witnesses. SLOPE/INTERCEPT confirmed already bit-exact (share the kernel, `4/4` each). TREND stays on the least-squares pipeline and publishes different bits (`7/12` under the simple kernel) — the LINEST normal-equations staging remains the open item for TREND/LINEST/LOGEST. | NUM-L | M1 | G8 probe / W109 sweep / G3-03-answers-*.json |
| G3-04 — GROWTH | Exponential-regression drift around `11`-`13` ULP on the bounded witnesses. | NUM-L | M1 | recon G3-04 |
| G3-05 — CHISQ.TEST, CHITEST | **W109 sweep (2026-07-12): decomposition unblock proven** — `CHISQ.TEST(o,e) == CHIDIST(S, df)` BIT-EXACTLY for a specific stored double S (tail cancels in the comparison), so the internal statistic is directly measurable without the gamma substrate: the internal statistic is IDENTIFIED as the plain-double ROW-MAJOR `Σ(o-e)²/e` (offset 0 on the two injective-tail tables of a 4-table live set); the CHIDIST tail half remains on the G3-01 substrate, so ALL CHISQ.TEST drift is inherited. F.TEST decomposes the same way via FDIST (see G3-06). | NUM-L | M1 | recon G3-05 / W109 sweep / G3-05-answers-*.json |
| G3-06 — F.TEST, FTEST | **W109 sweep (2026-07-12): decomposed.** `F.TEST(a,b) == 2·FDIST(F, df_hi, df_lo)` BIT-EXACTLY (3 live sets, df to (5,6)); F = larger-var/smaller-var (unbiased, n-1 divisor), 2× exact. Statistic layer identified; one variance-accumulation ULP detail open; the tail is FDIST -> all drift inherited from the G3-01 incomplete-beta substrate. | NUM-S | M1 | recon G3-06 / W109 sweep / G3-06-answers-*.json |
| G3-07 — GAUSS | Standard-normal `Phi(z)-0.5` drift, `2` ULP on the stable witness; needs the erf/CDF substrate (Phase-5 adjacent). PHI is resolved out of this row (W109 2026-07-11: `RN53(RN64(x·x))` -> x87 EXP -> `RN53(RN64(e·RN(1/sqrt(2π))))` with a live-pinned subnormal publication flush; `764/764` answered rows, see the ruled-out ledger and `smart-fuzzer/work/w109/G3-07-phi`). | NUM-S | M1 | recon G3-07 / W109 PHI closure |

## G4 — Numeric Exactness: Elementary And Trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G4-02 — ATANH | **W109 (2026-07-12): region C PROMOTED — premise overturned.** ATANH is NOT a custom rational; for `|x| >= ~1.25e-4` (the entire catalog mid-small band AND the near-1 rows) it is the naive `0.5·ln((1+x)/(1-x))` with the x87 CRT ln, **bit-exact 163/163** (the binary64 ratio's double-rounding is load-bearing; a higher-precision ratio fails). Promoted in `atanh.rs` (was `50/163`). Also found: **Excel ATANH is NOT odd in region C** — `ATANH(-0.2)` is 1 ULP off `-ATANH(0.2)`; the prior copysign-forced oddness was itself a divergence, now removed. OPEN: tiny-|x| region B (x87 `fyl2xp1` ln1p-pair) and a non-odd transition band near `1e-4` where neither form matches — retained platform path there (no regression). | NUM-S | M3 | W109_ATANH_IDENTIFICATION_20260712.md / atanh.rs |
| G4-03 — ACOTH | W109 (2026-07-12): with ATANH region C now identified, tested the inheritance `ACOTH(x)=ATANH(1/x)`: forming `1/x` first is WORSE (`18/57`, blows up near `|x|=1`), and the direct signed ratio `0.5·ln((x+1)/(x-1))` reaches only `40/57` (large `|x|` -> `1/x` small -> ATANH region B, +6 ULP). Excel does NOT compute ACOTH via a float `1/x`; its current `ln1p` form (`35/57`) is best. Genuinely on its own transcendental-staging path; inherits ATANH's open region-B difficulty. | NUM-S | M2 | recon G4-03 / W109 sweep |
| G4-04 — COMBIN, COMBINA, FACTDOUBLE, ERF.PRECISE, ERFC.PRECISE | `±1` ULP drift where OxFunc currently differs from Excel's published bits. PERMUT is resolved out of this row (W109 2026-07-11: ascending x87 spill-loop product, `702/702` live rows, see [`W109_PERMUT_COMBIN_FINDINGS_20260711.md`](function-lane/W109_PERMUT_COMBIN_FINDINGS_20260711.md)). COMBIN: `k -> min(k, n-k)` reduction CONFIRMED; all product-loop, factorial-ratio, reciprocal-multiply, and published-GAMMALN-composition kernels ruled out on a 505-row live corpus — leading hypothesis is an internal extended lgamma/exp substrate (Phase-5 lane). | NUM-S | M2 | BUG-FUNC-027 combinatorial group / recon G4-04 / W109 findings |
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
| G5-01 — MINVERSE | W109 (2026-07-12): algorithm = **Doolittle LU + partial pivot + division multiplier + sequential elimination + per-column unit solves, plain double** — generalizes to `150/159` on a combined corpus (51 + 108 fresh 3x3). A multi-agent workflow found a bottom-row reciprocal-multiply twist scoring `51/51` on the 51-row set, but it OVERFIT (held-out `147/159` < all-division `150/159`) — the enrichment corpus caught it before promotion. Crout/FMA/reciprocal-multiplier decisively ruled out. `9` residual `<=1`-ULP misses on ill-conditioned matrices = factorization-inner-loop accumulation detail (final axis). | NUM-S | M2 | W109 sweep / G5-01-answers-minverse*.json |

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
| G6-03 — YIELD | Solver publication drift: `19` ULP on the catalog witness and `6` ULP even on the par-price control. | NUM-L | M2 | BUG-FUNC-031 / recon G6-03 |
| G6-04 — ODDFYIELD | Solver drift around `3e5` ULP after the forward price path is aligned; two fresh witnesses are `307444`-`311909` ULP. | NUM-L | M1 | BUG-FUNC-032 / recon G6-04 |
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
