# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-06-21`

## Purpose

OxFunc's goal is **bit-exact emulation of Excel for every in-scope function and
operator** (~507 surfaces). That target space is far too large to track in scattered
notes. This file is the **single live worklist** of every *open* OxFunc-vs-Excel
discrepancy that OxFunc can evaluate locally (Category 2, context-free — see
[ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md)).

Context-sensitive (Category 1) discrepancies — reference, spill, host, locale,
formula-binding — do **not** live here; they live in
`smart-fuzzer/corpus/context_sensitive_catalog/` and are evaluated downstream.

## Maintenance rules (keep this coherent)

1. **One catalog.** This is the only place open Category-2 discrepancy *status* is
   tracked. It supersedes the open-tracking role of
   [`KNOWN_EXACTNESS_DEVIATIONS.md`](KNOWN_EXACTNESS_DEVIATIONS.md) and the open rows of
   [`docs/bugs/BUG_STREAM_REGISTER.csv`](bugs/BUG_STREAM_REGISTER.csv). Detailed
   root-cause / evidence still lives in `docs/bugs/streams/BUG-FUNC-*.md`; this catalog
   points to them.
2. **A function may appear more than once** — once per distinct discrepancy type (e.g.
   a large array-lift gap *and* a small numeric drift are two rows).
3. **Remove rows when signed off.** When a discrepancy is fixed and Excel-verified, delete
   its row — do not accumulate fixed-case history here. Durable history is in git and the
   stream register; transferable lessons go to
   [`OXFUNC_FIX_LEARNING_LOG.md`](OXFUNC_FIX_LEARNING_LOG.md). We track the path ahead,
   not the steps behind.
4. **Add as you discover.** A newly-found discrepancy gets a row here immediately (often
   `M0 noted`); it does not need a full stream doc until it is being worked.

## Legend

**Severity** (worst known discrepancy for that row):
- `STR` — structural: wrong kind / error code / shape / array behavior. Not a closeness issue.
- `NUM-L` — numeric, **large**: materially wrong number or `> ~2` ULP drift.
- `NUM-S` — numeric, **small**: `≤ ~2` ULP (incl. "OxFunc analytic-exact, Excel ±1 ULP" — still a bug, repair direction is match-Excel).

**Maturity** (evidence + repair):
- `M0 noted` — discrepancy witnessed; not yet minimized; no repair.
- `M1 tested` — one or a few minimized reproducers / focused tests exist.
- `M2 repair-tried` — repair attempted or repair direction proven; not landed.
- `M3 fixed-unsigned` — fix landed locally + locally green; awaiting live-Excel sign-off.
- `HO downstream` — OxFunc-side handled; blocked on a downstream/seam acknowledgement.

---

## G1 — Error-code & argument-domain guards (currently empty)

Excel returns an error (or saturates) where OxFunc returns a number, or vice-versa.
All three G1 rows were resolved against live Excel 16.0 build 20026 on 2026-06-20:

- **MOD** (BUG-FUNC-027 B1) — **fixed**. Excel's `#NUM!` boundary is a precise, *d-independent*
  threshold on the **quotient**: `|n/d| >= 1125900000000` (bisected to the exact double
  `0x4270624de9b00000`). Guard added to `mod_kernel`; 11/11 bit-exact incl. both witnesses, the
  boundary (`2^40+2^34` ok / `2^40+2^35` `#NUM!`), and the quotient rule (`MOD(2^50,2^10)` ok,
  `MOD(2^51,2^10)` `#NUM!`).
- **ATAN2** (BUG-FUNC-027 B3) — **fixed**. Excel returns `#NUM!` exactly when `x != 0` and `y/x`
  overflows to `∞` (the earlier "no clean rule" reading was a denormal `Value2` artifact); the
  axis case `x == 0` stays valid. Guard added to `atan2_kernel`; bit-exact incl. the witness and
  the finite-vs-`∞` boundary.
- **ACOTH / ACOSH near 1** (BUG-FUNC-027 C5) — **stale harness artifact**: the formula-literal
  parser rounded `1+ULP` → `1.0`; with exact `Range.Value2` inputs OxFunc already matched Excel
  bit-for-bit (`ACOTH(1+ULP)=18.36840028483855`, `ACOSH(1+1e-15)=4.712…e-8`). Regression tests added.

(The separate `MOD` ~`9.5E10`-ULP intermediate-truncation drift was fixed bit-exact on 2026-06-21
via the exact IEEE-remainder form — the G4 `MOD` row is removed. `ACOTH(1.001)` is now bit-exact
too; the open `ACOTH` residual is the `ACOTH(5)`/`ACOTH(10)` 1-ULP point on the G4 row.)

## G2 — Coercion, array-lift & kind/shape (currently empty)

Local `#VALUE!` where Excel coerces a scalar, spills over an array, or propagates an error.
Every G2 row was signed off against live Excel 16.0 build 20026 on 2026-06-20.

- **Ordinary operators (`OP_*`) — BUG-FUNC-001/002 closed, HO-FN-005 resolved.** OxFunc's binary
  value operators broadcast bit-exact across a 21-case sweep (5 arithmetic, concat, all six
  comparisons; outer-product, scalar/array, same-shape, non-broadcastable `#N/A` padding,
  `#DIV/0!`, and per-cell + scalar error propagation). The former `HO` downstream block is gone:
  OxFml now dispatches operators straight to OxFunc's `OP_*` surface (`eval/mod.rs`
  `binary_operator_identity`) with **no local array fallback** — confirmed by reading the OxFml
  evaluator and running its green `evaluator_operator_array_arithmetic_*` test against current
  OxFunc. Regression tests: `surface_dispatch::tests::eval_surface_value_call_op_*`.
- **Scalar-coercion / array-lift / error-propagation (BUG-FUNC-028 closed).** The named conversion
  / text / date / engineering / `IS*` surfaces were re-probed on the OxFunc evaluation surface
  (typed-arg local-eval, the Category-2 path):
  - *Array-lift gap — already resolved (stale rows removed).* All named surfaces now lift over
    arrays bit-exact, including the aggregators that *consume* rather than broadcast
    (`GCD`/`LCM`/`MULTINOMIAL`/`ARRAYTOTEXT`): Row-1 (`CLEAN`…`UNICODE`) 23/23, Row-2 dates
    (`EOMONTH`…`YEARFRAC`) and Row-3 (`TBILL*`) 19/20, Row-4 (`IS*`) 4/4. The W090/W092
    array-support work fixed these; the catalog rows were never reconciled.
  - *Error-propagation — fixed (2026-06-20).* `DATEVALUE`/`TIMEVALUE` (+ siblings `DAYS360`/`DATEDIF`)
    and `ARRAYTOTEXT` now propagate a scalar error argument unchanged (`f(NA())` → `#N/A`, code
    preserved), while errors *inside* an array argument stay textified — 7/7 vs Excel.

## G3 — Numeric exactness: special & statistical functions

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, TDIST, TINV (+ CONFIDENCE.T, Z.TEST unprobed) | distribution scalar numeric drift — **quantified 2026-06-21** (live Excel, representative witnesses): `1`–`28` ULP (BETAINV 28, CHIDIST 17, TDIST 13, FDIST 9, FINV 3, HYPGEOMDIST 2, the rest 1). `NORMSDIST`/`NORMSINV`/`GAMMADIST`/`BETADIST` are **bit-exact** (removed from the row). Smaller than the prior NUM-L assumption; repair by the shared incomplete-beta/gamma substrate | NUM-S | M2 | BUG-FUNC-021 / KED-STAT-001 / `oxf-simj` |
| GAMMA | negative-non-integer reflection drift (`~1290` ULP after cell-ref resweep; re-probed `GAMMA(-1.00012)` = `182` ULP) | NUM-L | M1 | BUG-FUNC-027 C1 |
| BESSELY | **quantified + partially fixed 2026-06-21**: up to `~6.2E11` ULP (materially wrong — catastrophic band). Order-0 `x<8` fixed (Excel uses Numerical Recipes' *truncated* `2/π = 0.636619772`, not full-precision: `BESSELY(0.5,0)` `4.3E6→0`, `BESSELY(3,0)` `→1` ULP), plus a `bessj0` asymptotic coefficient typo. **Residual:** order-1 (`bessy1`), the `x≥8` asymptotic, and the order≥2 recurrence are still `10^8`–`10^11` ULP — Excel uses a **non-NR** method there (its `Y1(2.5)` and `J1(10)` differ from NR by `~1E-8`/`~5E-6`). Needs Excel's exact `x≥8`/order-1 Bessel algorithm | NUM-L | M1 | BUG-FUNC-024 / KED-BESSEL-001 / `oxf-xp6p` |
| BESSELJ (and likely BESSELI/BESSELK) for `x≥8` | newly found 2026-06-21: NR-exact for `x<8` but the `x≥8` asymptotic diverges from Excel (`BESSELJ(10,1)` `~8E11` ULP, `BESSELJ(15,3)` `~4E9`) — same non-NR `x≥8` method as BESSELY. BESSELI/BESSELK `x≥8` not yet probed | NUM-L | M0 | BUG-FUNC-024 (companion) |

**Bessel `x≥8` / order-1 diagnosis (2026-06-21).** Investigated the BESSELY/BESSELJ `x≥8` and
order-1 residual and ruled out the tractable causes: it is **not** a coefficient typo (the
`bessj1`/`bessy1` coefficients match Numerical Recipes exactly), **not** a structural bug
(`bessj1` is true-accurate at `x≥50` — `J1(50)` `4E-10`), and **not** a threshold issue (the
`x<8` rational extrapolated to `x=10` gives garbage `0.0432`, far from Excel `0.0434722`). The
root cause: NR's 5-term asymptotic is genuinely `~1E-6` inaccurate in the moderate-`x` band
(`8`–`~30`) for order 1, and **Excel uses a more-accurate *proprietary* method there** — Excel's
`J1(10)` is `~5E-7` off the true value while OxFunc's NR is `~6E-6` off; Excel's `Y1(2.5)` is also
more accurate than NR's. So OxFunc is *less* accurate than Excel here, and matching bit-for-bit
needs Excel's exact (more-precise) Bessel coefficients/method — the same hard tier as the solver
substrate, not a quick fix. The order-0 `x<8` constant fix (above) stands.
| FORECAST, FORECAST.LINEAR, TREND, LINEST, LOGEST | least-squares regression drift (`≤2` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| GROWTH | exponential-regression drift, `~11` ULP (`exp` amplifies the linear fit) | NUM-L | M1 | G8 probe `GROWTH({1,3,2,5},{1,2,3,4},{5})` |
| CHISQ.TEST, CHITEST | chi-square test-statistic drift, `~8` ULP | NUM-L | M1 | G8 probe `CHISQ.TEST({10,20,30},{12,18,30})` |
| F.TEST, FTEST | F-test statistic drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| GAUSS, PHI | standard-normal `Φ(z)-0.5` / density drift (`≤2` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |

## G4 — Numeric exactness: elementary & trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| TAN, SIN, COT, SEC, CSC | large-argument reduction drift. **Re-probed 2026-06-21** (cell-ref, live Excel b20026): exact for small/moderate args (`SIN(0.5)`, `TAN(1.2)`, `COT(-307.07)` all 0 ULP); `10^5`–`10^6` args drift `~50`–`900` ULP (`TAN(797601.58)` 719, `COS(797601.58)` 913, `SIN(961281.44)` 49), so the headline figure is **far below** the legacy `~3.3E12` (that band was near-pole amplification of the same reduction error). Root cause: Excel uses its own argument reduction that Rust std trig (which reduces more precisely) does not reproduce — repair requires reverse-engineering Excel's exact reduction. Re-ranked out of the catastrophic band. | NUM-L | M1 | BUG-FUNC-027 C3 |
| ATANH | catastrophic near-`±1` band fixed (2026-06-21, odd symmetry → bit-exact near `±1`). **Open residual:** `2`–`3` ULP at mid-small args (`ATANH(0.2)` = `…9849`, 3 ULP; `ATANH(0.1)`, 2 ULP) — Excel's own ATANH approximation is *less* accurate than correctly-rounded there; OxFunc (Rust `atanh`) sits at the true value. Not an ln-precision gap (Excel's `LN` is correctly-rounded). Matching needs Excel's exact ATANH routine. | NUM-S | M1 | BUG-FUNC-027 C4 |
| ACOTH | catastrophic large-`\|x\|` band fixed (2026-06-21, odd-symmetric `0.5*ln1p(2/(\|x\|-1))`). **Open residual:** `1` ULP at `ACOTH(5)`/`ACOTH(10)`. **ln-substrate probe (2026-06-21) is decisive:** a *validated* correctly-rounded double-double `ln` (matches CR `ln(2)`) gives the same `…984c` as ln1p — the true value — while Excel returns `…984d` (1 ULP **high**). So Excel's ACOTH is its own less-accurate routine, **not** `ATANH(1/x)` and **not** `LN((x+1)/(x-1))/2` (Excel's `LN` is correctly-rounded; Excel even maps the same real value to different doubles: `ATANH(0.2)=…9849` vs `ACOTH(5)=…984d`). A better/extended `ln` cannot match it — closure needs Excel's exact ACOTH routine. | NUM-S | M1 | BUG-FUNC-027 C5 |
| COMBIN, COMBINA, PERMUT, FACTDOUBLE, ERF.PRECISE, ERFC.PRECISE | `±1` ULP where OxFunc is analytic-exact and Excel is off — match-Excel | NUM-S | M1 | BUG-FUNC-027 (combinatorial group) |
| CONVERT | unit-conversion factor drift (`1` ULP, `CONVERT(1,"m","ft")`) | NUM-S | M1 | G8 probe 2026-06-19 |

## G5 — Numeric exactness: matrix

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| MINVERSE | multi-cell inversion low-bit drift (`1×1` publication already fixed) | NUM-L | M2 | BUG-FUNC-025 / KED-MATRIX-001 / `oxf-dzfk` |
| MMULT | matrix numeric / `scalar-vs-1×1` shape drift | NUM-L | M1 | BUG-FUNC-023 / `oxf-i45e` |

## G6 — Financial: exactness, computation & solver

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| PMT, PPMT (IPMT adjacent) | annuity publication exactness drift; re-confirmed vs live Excel 16.0 b20026 (2026-06-20): `PMT(0.05/12,360,200000)` 8 ULP, `PPMT(0.05/12,1,360,200000)` 63 ULP. Fix never landed; KED known-residual held for W103 | NUM-L | M1 | BUG-FUNC-015 / KED-FIN-001 / `oxf-fckb` |
| ACCRINT | **act/act leap-February residual fixed (2026-06-21)**: rewrote as a faithful F# `accrInt` port — the settlement-side fraction is normalised by the *canonical* `CoupDays(first-1p,first)`, so a leap-crossing period is never measured by its own actual length (the prior `~0.07%` error); plus a forward branch for settlement past `first_interest` (which F# itself rejects, but Excel computes, and OxFunc now matches). Bit-exact vs live Excel across a **24-case sweep** — all 5 bases, both settlement regimes, EOM dates, deep multi-period, quarterly/annual. Residual: **1 ULP** on the `us30360` triple-edge only (issue mid-period *and* settlement past `first_interest`) — operation order (constant-length bases want sum-then-divide). | NUM-S | M1 | BUG-FUNC-030 |
| YIELD | structural `#NUM!` fixed (2026-06-20): the root-finder probed negative candidate yields that `price_kernel` rejected via `rate(yld)`; now solves over `pcomp` directly. Residual `~19` ULP vs Excel (bisection vs Excel's solver) | NUM-L | M2 | BUG-FUNC-031 |
| ODDFYIELD | **ODDFPRICE now bit-exact across all five bases** (2026-06-20): replaced the single-period-length closed form with a faithful port of the ExcelFinancialFunctions `oddFPrice` two-branch algorithm (short `DFC<E`; long per-quasi-coupon-period `dci/nl` summation) — `all_bit_exact` vs live Excel 16.0 b20026 *and* the F# reference on the 10-case G6 three-way matrix (incl. the act/act, act/360, act/365 bases that were `10^10`–`10^12` ULP off). ODDFPRICE row removed. ODDFYIELD still diverges: it inverts the now-bit-exact price via a solver, but OxFunc bisects from 0 while Excel uses Newton-from-guess (`~3e5` ULP; F# also off). Needs the solver substrate shared with YIELD/RATE/IRR. | NUM-L | M1 | BUG-FUNC-032 |
| RATE | structural lane signed off (2026-06-20): default-guess mortgage root now converges and Excel returns a number, not `#NUM!`. Residual `~586` ULP vs Excel (`0.0041666445363460975` vs `0.004166644536345589`) — distinct numeric drift in the solver substrate | NUM-L | M1 | BUG-FUNC-009 (bit-parity) / W103 |
| IRR | structural error-code drift signed off (2026-06-20). Solver substrate, pass 1 (2026-06-20): added a Newton **rate-polish** of the solver seed (the gentle-NPV-slope cases were stopping at `|NPV|<1e-8`, ~`10^4`–`10^5` ULP from the root, outside the ±16-ULP publication plateau). **Representable-root cases now bit-exact**: `IRR({1,-2})`=`1.0` (was 114720 ULP), `IRR({-100,121})`, and the `{-10000,3000,4200,6800}` publication witness — OxFunc now *beats* the F# reference on the first two (F# 1 / 14571 ULP off). Residual: **irrational-root** cases where Excel's iteration-landing double differs from the |NPV|-minimal double — `IRR({-100,50,60})` `~80` ULP, mixed-5-flow `~14k` ULP, both ≈ F# now. Closing these needs Excel's *exact* iteration (guess / step / stop), the shared substrate with RATE/YIELD/ODDFYIELD (W103). | NUM-L | M1 | BUG-FUNC-028 (out-of-stream) |
| CUMPRINC | full-schedule (type 0) numeric drift `~6` ULP — distinct from the closed type=1 structural fix (BUG-FUNC-034) | NUM-L | M1 | G8 probe `CUMPRINC(0.1,12,1000,1,12,0)` |
| NPER | period-count drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| YIELDMAT | yield-at-maturity drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| TBILLYIELD | discounted-bill yield sub-ULP drift (`1` ULP on some settlements; array-lift itself correct) | NUM-S | M1 | G2 resweep 2026-06-20 |

**Solver-substrate diagnosis (2026-06-21).** ODDFYIELD / YIELD / RATE and IRR's irrational
roots are *not* a solver-quality problem — their forward functions (ODDFPRICE, PRICE, the
annuity balance, NPV) are already bit-exact, and Excel stops its root-finder **early** at an
*iteration-specific* double, not at a refined root. Evidence: at Excel's `RATE(360,-1073.64,
200000)` the balance residual is `+1.0e-8` (positive, far from 0) and monotone — Excel halted
when the step got small, ~48 ULP above the converged root; **ODDFYIELD is additionally
ill-conditioned** — a ~10³-ULP band of yields all map to the same price double (residual flat
at `-5.8e-10`), so no residual-min refinement can pick Excel's value. The residual-min/plateau
trick that made IRR's *representable* roots bit-exact does **not** generalise here. Matching
needs Excel's exact proprietary iteration (method / guess / derivative / stopping / op-order),
which differs from OxFunc, the F# reference, *and* the POI/LibreOffice "Excel-compatible"
reimplementations (POI's secant diverges for this RATE case; analytic Newton lands 48 ULP off,
and its iteration count can't reconcile with Excel's documented 20-iter/`1e-7` cap). This is
the hardest tier; it needs a dedicated reverse-engineering spike, not a better solver.

## G7 — Comparison & misc semantics (currently empty)

The numeric-comparison tolerance lane (operators, criteria/database families, `SWITCH`;
BUG-FUNC-004) was signed off against live Excel 16.0 build 20026 on 2026-06-20: the shared
truncation-style 15-significant-digit helper (`compare_excel_numbers`) matches Excel
bit-for-bit on the tolerant lanes (`=0.1+0.2=0.3`, the `((123456789012345*10)+5)/1E25`
boundary pair, `COUNTIF`/`SUMIF`/`SWITCH`) while the exact-match contrast families
(`MATCH`/`XMATCH`/`DELTA`) stay exact. Re-add a row only on a fresh witness.

## G8 — Untriaged inbox (currently empty)

New smart-fuzzer `mixed_or_open` findings land here first (a genuine non-match with no
stream and no triaged severity), then get a witness + severity probe and promote into
G1–G7 or the context-sensitive catalog.

**2026-06-19 drain.** The 28-surface backlog from the 2026-05-28 status map was probed
against live Excel 16.0 build 20026 (bit-level comparison) and fully triaged:

- **Promoted (numeric drift, now `M1`):** regression family FORECAST/FORECAST.LINEAR/
  TREND/LINEST/LOGEST and GROWTH → G3; CHISQ.TEST/CHITEST and F.TEST/FTEST → G3; GAUSS/PHI
  → G3; CONVERT → G4; CUMPRINC/NPER/YIELDMAT → G6 (YIELDDISC also promoted here, then
  closed bit-exact 2026-06-20).
- **Already triaged:** IRR (structural) → G6.
- **Routed to the context-sensitive catalog:** JIS, HYPERLINK, TRIMRANGE.
- **Cleared — bit-exact on the baseline witness:** PERCENTILE.EXC/.INC, QUARTILE.EXC/.INC,
  ACOT, NPV, XNPV. NPV/XNPV overlap closed BUG-FUNC-038/037 (stale run-data confirmed). The
  rest were not reproduced on a baseline witness; if the smart-fuzzer re-flags them the
  original edge input is needed, otherwise they stay clear.

No surfaces are currently awaiting triage.

---

## Pointers

- Category boundary & policy: [ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md)
- Context-sensitive (Category 1) catalog: `smart-fuzzer/corpus/context_sensitive_catalog/`
- Severity vocabulary & comparison policy: `CHARTER.md` §4.1; smart-fuzzer `Get-StandardSeverityClass`
- Transferable lessons: [OXFUNC_FIX_LEARNING_LOG.md](OXFUNC_FIX_LEARNING_LOG.md)
- Bit-exact Excel comparison plumbing: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`
