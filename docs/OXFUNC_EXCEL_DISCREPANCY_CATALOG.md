# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-07-04` (W108 x87 transcendental core: EXP/LN/LOG10/LOG now bit-exact; POWER row added)
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

Open Category-2 rows: `27`

| Group | Current rows |
|-------|--------------|
| G1 error-code/domain guards | 0 |
| G2 structural kind/shape/admission | 0 |
| G3 special/statistical numeric exactness | 7 |
| G4 elementary/trig numeric exactness | 6 |
| G5 matrix numeric/shape | 2 |
| G6 financial exactness/solver | 12 |
| G7 comparison/misc semantics | 0 |
| G8 untriaged inbox | 0 |

W108 resolved (removed from tracking as bit-exact via the x87 backend): `EXP`, `LN`,
`LOG10`, `LOG(x, base)` — 64-bit Excel computes these with the legacy x87 CRT
transcendental chain (`87tran.asm`, CW `0x133F`), reproduced bit-for-bit by
[`crate::excel_numeric::x87`] on the reference x86-64 host (249/249 corpus + a fresh
396-row live sweep). These were never catalog rows (they were W108-A research findings);
noted here for provenance. Many small-ULP G3/G4 residuals whose kernels call `exp`/`ln`
internally may now be closable by routing those calls through the x87 backend.

## G1 — Error-Code And Argument-Domain Guards

No current open rows.

## G2 — Structural Kind, Shape, And Admission

No current open rows.

## G3 — Numeric Exactness: Special And Statistical Functions

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, TDIST, TINV (+ CONFIDENCE.T, Z.TEST unprobed) | Distribution scalar numeric drift, currently quantified at `1`-`28` ULP on representative live-Excel witnesses. | NUM-S | M2 | BUG-FUNC-021 / KED-STAT-001 / `oxf-simj` |
| GAMMA | Negative-non-integer reflection drift; latest representative reprobe records `182` ULP at `GAMMA(-1.00012)`. | NUM-L | M1 | BUG-FUNC-027 C1 |
| FORECAST, FORECAST.LINEAR, TREND, LINEST, LOGEST | Least-squares regression drift, currently `<=2` ULP. | NUM-S | M1 | G8 probe 2026-06-19 |
| GROWTH | Exponential-regression drift around `~11` ULP. | NUM-L | M1 | G8 probe `GROWTH({1,3,2,5},{1,2,3,4},{5})` |
| CHISQ.TEST, CHITEST | Chi-square test-statistic drift around `~8` ULP. | NUM-L | M1 | G8 probe `CHISQ.TEST({10,20,30},{12,18,30})` |
| F.TEST, FTEST | F-test statistic drift around `1` ULP. | NUM-S | M1 | G8 probe 2026-06-19 |
| GAUSS, PHI | Standard-normal `Phi(z)-0.5` / density drift, currently `<=2` ULP. | NUM-S | M1 | G8 probe 2026-06-19 |

## G4 — Numeric Exactness: Elementary And Trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| TAN, SIN, COT, SEC, CSC (+ COS `1` ULP) | Large-argument reduction drift; current cell-ref reprobes show small/moderate inputs exact and `10^5`-`10^6` inputs drifting by roughly `50`-`900` ULP. 2026-07-02: Excel `COS` also witnessed `1` ULP off at moderate args `49.214601836`/`149.214601836` (`SIN` exact there); `BESSELJ(50,0)`, `BESSELJ(150,0)` (`1` ULP) and `BESSELJ(50,2)` (`2` ULP) inherit exactly this and close with it — the Bessel substrate itself is signed off (BUG-FUNC-024). | NUM-L | M1 | BUG-FUNC-027 C3 |
| ATANH | Mid-small argument drift: `2`-`3` ULP at current witnesses such as `ATANH(0.1)` and `ATANH(0.2)`. | NUM-S | M1 | BUG-FUNC-027 C4 |
| ACOTH | Small residual drift: `1` ULP at current witnesses such as `ACOTH(5)` and `ACOTH(10)`. | NUM-S | M1 | BUG-FUNC-027 C5 |
| COMBIN, COMBINA, PERMUT, FACTDOUBLE, ERF.PRECISE, ERFC.PRECISE | `±1` ULP drift where OxFunc currently differs from Excel's published bits. | NUM-S | M1 | BUG-FUNC-027 combinatorial group |
| CONVERT | Unit-conversion factor drift, currently `1` ULP at `CONVERT(1,"m","ft")`. | NUM-S | M1 | G8 probe 2026-06-19 |
| POWER | Fractional-exponent, positive-base path drift. Root cause found (W108): Excel `POWER(x,y)` is `exp(y·ln x)` via the x87 exp/ln with f64 intermediates, NOT `powf` and NOT the fused x87 `x^y` chain. Live sweep (220 rows): `exp(y·ln x)` `86%` bit-exact (rest `1` ULP), current `powf` only `~5%` (up to `~125` ULP on large `\|y\|`). `exp(y·ln x)` strictly beats `powf` `84/90` head-to-head. Integer exponents use the validated `powi` publication (unaffected). Residual `14%` (all `1` ULP) is an unresolved intermediate-precision detail — a dedicated pass may reach bit-exact or land `exp(y·ln x)` as best-achievable. | NUM-L | M2 | BUG-FUNC-042 / W108 Phase-D |

## G5 — Matrix Numeric And Shape

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| MINVERSE | Multi-cell inversion low-bit drift, currently quantified at `<=37` ULP across small and ill-conditioned matrices. | NUM-S | M2 | BUG-FUNC-025 / KED-MATRIX-001 / `oxf-dzfk` |
| MMULT | Remaining scalar-vs-`1x1` publication/shape edge; numeric matrix multiplication itself is currently bit-exact on sampled matrix witnesses. | STR | M1 | BUG-FUNC-023 / `oxf-i45e` |

## G6 — Financial Exactness, Computation, And Solver

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| PMT, PPMT (IPMT, CUMPRINC adjacent) | Annuity publication exactness drift; PMT `8` ULP, PPMT `63` ULP on witnesses (PMT reaches `~5.5e8` ULP on tiny-rate/long-horizon). **Root cause resolved 2026-07-03 (W108)**: PMT uses Excel's `exp(n*log1p(r))`/`expm1` chain (FV/PV use `powi` and already match). Best-achievable pass (W108 epic `oxf-wpzw`): CR core + `log1p`-chain compositions collapses the drift to a **≤2-3 ULP residual** (bit-exact on ~70%, ≤1 ULP on ~95%, incl. realistic witnesses). Row STAYS OPEN — the residual is a low-tractability bug (Excel's bespoke internal `exp`/`log`, unreproducible without its binary); parity remains the goal, smaller drift is the improvement. | NUM-L→NUM-S | M2 | BUG-FUNC-015 / KED-FIN-001 / `oxf-fckb` / W108 |
| ACCRINT | Residual `1` ULP on the `us30360` triple-edge case: issue mid-period and settlement past first interest. | NUM-S | M1 | BUG-FUNC-030 |
| YIELD | Solver publication drift around `~19` ULP. | NUM-L | M2 | BUG-FUNC-031 |
| ODDFYIELD | Solver drift around `~3e5` ULP after the forward price path is aligned. | NUM-L | M1 | BUG-FUNC-032 |
| RATE | Solver residual around `~586` ULP on the mortgage-style default-guess witness. | NUM-L | M1 | BUG-FUNC-009 bit-parity / W103 |
| IRR | Irrational-root solver residuals remain, currently around `~80` ULP to `~14k` ULP on representative witnesses. | NUM-L | M1 | BUG-FUNC-028 out-of-stream |
| CUMPRINC | Full-schedule `type=0` numeric drift around `~6` ULP. | NUM-L | M1 | G8 probe `CUMPRINC(0.1,12,1000,1,12,0)` |
| NPER | Period-count drift around `1` ULP. | NUM-S | M1 | G8 probe 2026-06-19 |
| YIELDMAT | Yield-at-maturity drift around `1` ULP. | NUM-S | M1 | G8 probe 2026-06-19 |
| TBILLYIELD | Discounted-bill yield drift around `1` ULP on some settlements. | NUM-S | M1 | G2 resweep 2026-06-20 |
| XNPV | Discounted-cashflow summation/rounding drift around `~16` ULP at `XNPV(0.05,{-1000,500,600},{43831,44013,44562})`. Match Excel's discounting/summation order. | NUM-L | M1 | `oxf-jbi3` / W090 §4.3 |
| YIELDDISC | Discount-yield formula numeric drift around `~5` ULP at `YIELDDISC(44013,44562,95,100,0)`. | NUM-L | M1 | `oxf-pzav` / W090 §4.3 |

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
