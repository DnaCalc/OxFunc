# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-07-10` (live Excel build `20131`; stale YIELDDISC row removed;
MMULT/MINVERSE `1x1` publication observations moved to Category 1; TBILLYIELD
association repaired and signed off `2156/2156`)
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

Open Category-2 rows: `23`

| Group | Current rows |
|-------|--------------|
| G1 error-code/domain guards | 0 |
| G2 structural kind/shape/admission | 0 |
| G3 special/statistical numeric exactness | 7 |
| G4 elementary/trig numeric exactness | 5 |
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
| G3-01 — BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, TDIST, TINV (+ CONFIDENCE.T, Z.TEST unprobed) | Distribution scalar numeric drift, currently quantified at `1`-`28` ULP on representative live-Excel witnesses. | NUM-S | M2 | BUG-FUNC-021 / KED-STAT-001 / `oxf-simj` |
| G3-02 — GAMMA | Negative-non-integer reflection drift; latest representative reprobe records `182` ULP at `GAMMA(-1.00012)`. | NUM-L | M1 | BUG-FUNC-027 C1 |
| G3-03 — FORECAST, FORECAST.LINEAR, TREND, LINEST, LOGEST | Least-squares regression drift; fresh exact-line FORECAST is `5` ULP and a perturbed case is `2` ULP. | NUM-L | M1 | G8 probe / recon G3-03 |
| G3-04 — GROWTH | Exponential-regression drift around `11`-`13` ULP on the bounded witnesses. | NUM-L | M1 | recon G3-04 |
| G3-05 — CHISQ.TEST, CHITEST | Chi-square test-statistic drift around `7`-`8` ULP. | NUM-L | M1 | recon G3-05 |
| G3-06 — F.TEST, FTEST | F-test statistic drift around `1` ULP. | NUM-S | M1 | recon G3-06 |
| G3-07 — GAUSS, PHI | Standard-normal `Phi(z)-0.5` / density drift, currently `1`-`2` ULP on stable witnesses. | NUM-S | M1 | recon G3-07 |

## G4 — Numeric Exactness: Elementary And Trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G4-01 — TAN, SIN, COT, SEC, CSC (+ COS `1` ULP) | **Open numeric kernel; the separate `|x| >= 2^27 -> #NUM!` guard is aligned.** Fresh cell-ref replay on Excel 16.0 build 20131 records `TAN(797601.58)` `719` ULP, `SIN(961281.44)` `49` ULP, `SIN(100000)` / `TAN(100000)` `230` ULP, `COT(100000)` / `CSC(100000)` `351` ULP, and `SIN(134217727)` `5664` ULP; `COS(49.214601836)` and `COS(149.214601836)` remain `1` ULP. Some witnesses/functions are exact, so this is argument-dependent reduction/publication drift, not a blanket family failure. The small Bessel residuals noted by BUG-FUNC-024 inherit this trig lane. | NUM-L | M1 | BUG-FUNC-027 C3 / recon G4-01 |
| G4-02 — ATANH | Mid-small witnesses remain `2`-`3` ULP. The apparent x87-LN repair was rejected after an expanded 368-case sweep: it matched `297` but regressed `71`, including catastrophic tiny-input collapse and near-boundary drift. The restored odd-symmetric platform path matches `235/368`; search a piecewise historical kernel. | NUM-S | M2 | BUG-FUNC-027 C4 / candidate-closure sweep |
| G4-03 — ACOTH | Small residual drift: `1` ULP at `ACOTH(5)` and `ACOTH(10)`. | NUM-S | M1 | BUG-FUNC-027 C5 / recon G4-03 |
| G4-04 — COMBIN, COMBINA, PERMUT, FACTDOUBLE, ERF.PRECISE, ERFC.PRECISE | `±1` ULP drift where OxFunc currently differs from Excel's published bits. | NUM-S | M1 | BUG-FUNC-027 combinatorial group / recon G4-04 |
| G4-05 — CONVERT | Unit-conversion factor drift, `1` ULP at `CONVERT(1,"m","ft")`; `CONVERT(1,"in","m")` is an exact control. | NUM-S | M1 | recon G4-05 |

## G5 — Matrix Numeric And Shape

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G5-01 — MINVERSE | Multi-cell inversion low-bit drift, currently quantified at `<=37` ULP across small and ill-conditioned matrices; two fresh `2x2` cells are each `1` ULP. | NUM-S | M2 | BUG-FUNC-025 / KED-MATRIX-001 / `oxf-dzfk` / recon G5-01 |

`MINVERSE(5)` and `MMULT(5,2)` are deliberately absent here. Nested `TYPE`
evidence proves that their function results remain `1x1` arrays; Excel's final-cell
scalar appearance belongs to the Category-1 worksheet publication/comparator seam.
They are now explicit `publication_shape` rows `CSC-0024`/`CSC-0025` under
`smart-fuzzer/corpus/context_sensitive_catalog/`, with downstream handoff `HO-FN-010`.

## G6 — Financial Exactness, Computation, And Solver

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G6-01 — PMT, PPMT (IPMT, CUMPRINC/CUMIPMT adjacent) | Annuity publication exactness drift. W108 Phase E supersedes the earlier forward-form conclusion: Excel uses the discount arrangement already present in OxFunc, `em=expm1(-n*log1p(r)); v=1+em; pmt=(pv+fv*v)*r/em`. The best tested historical Kahan/x87 helper candidate scores `2285/4040` exact (`56.6%`) and `92.9%` within `1` ULP on the adversarial PMT corpus; the `553`-row `n=1` isolation lane localizes its remaining error to the final rounding of `em`. The fresh mortgage PMT control is exact while first-period PPMT is `1` ULP, sharpening the adjacent recurrence/publication lane. Row STAYS OPEN. | NUM-L→NUM-S | M2 | BUG-FUNC-015 / KED-FIN-001 / W108 / recon G6-01 |
| G6-02 — ACCRINT | Residual `1` ULP on the now-pinned `us30360` triple-edge witness: issue `43565`, first interest `43647`, settlement `43905`, rate `0.05`, par `1000`, frequency `2`, basis `0`. | NUM-S | M1 | BUG-FUNC-030 / recon G6-02 |
| G6-03 — YIELD | Solver publication drift: `19` ULP on the catalog witness and `6` ULP even on the par-price control. | NUM-L | M2 | BUG-FUNC-031 / recon G6-03 |
| G6-04 — ODDFYIELD | Solver drift around `3e5` ULP after the forward price path is aligned; two fresh witnesses are `307444`-`311909` ULP. | NUM-L | M1 | BUG-FUNC-032 / recon G6-04 |
| G6-05 — RATE | Solver residual `586` ULP on the mortgage witness and `72` ULP on a one-period identity whose mathematical root is `0.1`. | NUM-L | M1 | BUG-FUNC-009 bit-parity / W103 / recon G6-05 |
| G6-06 — IRR | Irrational-root solver residuals remain: `80` ULP and `14096` ULP on the bounded witnesses. | NUM-L | M1 | BUG-FUNC-028 out-of-stream / recon G6-06 |
| G6-07 — CUMPRINC | The full-schedule control is exact; the half-schedule witness is `1` ULP, localizing the current evidence to boundary-sensitive accumulation. | NUM-S | M1 | recon G6-07 |
| G6-08 — NPER | Period-count drift `1` ULP on `NPER-0000` (`0x405a1d41fa9d1c49` local vs Excel `...4a`); the control witness is exact. | NUM-S | M1 | in-crate `nper_exactness_audit` / W108 / recon G6-08 |
| G6-09 — YIELDMAT | Yield-at-maturity drift is `1` ULP for basis 1 and `2` ULP for the fresh basis-0 control. | NUM-S | M1 | recon G6-09 |
| G6-11 — XNPV | The fractional-year catalog witness is `16` ULP while a two-flow exact-year control is bit-exact, pointing to POWER/transcendental and summation staging rather than basic date admission. | NUM-L | M1 | `oxf-jbi3` / W090 §4.3 / recon G6-11 |

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
