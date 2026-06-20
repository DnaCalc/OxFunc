# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-06-20`

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

(The separate `MOD` ~`9.5E10`-ULP intermediate-truncation drift and `ACOTH(1.001)` precision drift
remain on the G4 numeric rows.)

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
| BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMADIST, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, NORMSDIST, NORMSINV, TDIST, TINV, CONFIDENCE.T, Z.TEST | distribution scalar numeric drift (repair by numerical substrate, not per-case) | NUM-L | M2 | BUG-FUNC-021 / KED-STAT-001 / `oxf-simj` |
| GAMMA | negative-non-integer reflection drift (`~1290` ULP after cell-ref resweep) | NUM-L | M1 | BUG-FUNC-027 C1 |
| BESSELY | Bessel-Y scalar numeric drift | NUM-L | M1 | BUG-FUNC-024 / KED-BESSEL-001 / `oxf-xp6p` |
| FORECAST, FORECAST.LINEAR, TREND, LINEST, LOGEST | least-squares regression drift (`≤2` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| GROWTH | exponential-regression drift, `~11` ULP (`exp` amplifies the linear fit) | NUM-L | M1 | G8 probe `GROWTH({1,3,2,5},{1,2,3,4},{5})` |
| CHISQ.TEST, CHITEST | chi-square test-statistic drift, `~8` ULP | NUM-L | M1 | G8 probe `CHISQ.TEST({10,20,30},{12,18,30})` |
| F.TEST, FTEST | F-test statistic drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| GAUSS, PHI | standard-normal `Φ(z)-0.5` / density drift (`≤2` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |

## G4 — Numeric exactness: elementary & trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| TAN, SIN, COT, SEC, CSC | moderate-large argument-reduction drift (Cody-Waite vs extended-π; up to `~3.3E12` ULP) | NUM-L | M1 | BUG-FUNC-027 C3 |
| ACOTH | catastrophic large-`\|x\|` band fixed (2026-06-21): the odd-symmetric `0.5*ln1p(2/(\|x\|-1))` form replaces the direct `0.5*ln((x+1)/(x-1))` ratio (`~1.2E14` ULP → bit-exact across the probed range incl. large + negative args; ATANH was the same odd-symmetry defect, now bit-exact, row removed). Residual: `1` ULP at scattered mid-range points (`ACOTH(5)`, `ACOTH(10)`) — Excel's x87 extended-precision `ln`, not matchable in IEEE double | NUM-S | M1 | BUG-FUNC-027 C5 |
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
| ACCRINT | half-value defect fixed (2026-06-20): odd first coupon now sums over quasi-coupon periods and `calc_method` matches Excel; 13/15 Excel-matrix cases bit-exact. Residual: `act/act` (basis 1/3) normal-period-length on later multi-coupon periods crossing a leap February (`~0.07%`), plus `act/360` sub-ULP | NUM-L | M2 | BUG-FUNC-030 |
| YIELD | structural `#NUM!` fixed (2026-06-20): the root-finder probed negative candidate yields that `price_kernel` rejected via `rate(yld)`; now solves over `pcomp` directly. Residual `~19` ULP vs Excel (bisection vs Excel's solver) | NUM-L | M2 | BUG-FUNC-031 |
| ODDFYIELD | **ODDFPRICE now bit-exact across all five bases** (2026-06-20): replaced the single-period-length closed form with a faithful port of the ExcelFinancialFunctions `oddFPrice` two-branch algorithm (short `DFC<E`; long per-quasi-coupon-period `dci/nl` summation) — `all_bit_exact` vs live Excel 16.0 b20026 *and* the F# reference on the 10-case G6 three-way matrix (incl. the act/act, act/360, act/365 bases that were `10^10`–`10^12` ULP off). ODDFPRICE row removed. ODDFYIELD still diverges: it inverts the now-bit-exact price via a solver, but OxFunc bisects from 0 while Excel uses Newton-from-guess (`~3e5` ULP; F# also off). Needs the solver substrate shared with YIELD/RATE/IRR. | NUM-L | M1 | BUG-FUNC-032 |
| RATE | structural lane signed off (2026-06-20): default-guess mortgage root now converges and Excel returns a number, not `#NUM!`. Residual `~586` ULP vs Excel (`0.0041666445363460975` vs `0.004166644536345589`) — distinct numeric drift in the solver substrate | NUM-L | M1 | BUG-FUNC-009 (bit-parity) / W103 |
| IRR | structural error-code drift signed off (2026-06-20). Solver substrate, pass 1 (2026-06-20): added a Newton **rate-polish** of the solver seed (the gentle-NPV-slope cases were stopping at `|NPV|<1e-8`, ~`10^4`–`10^5` ULP from the root, outside the ±16-ULP publication plateau). **Representable-root cases now bit-exact**: `IRR({1,-2})`=`1.0` (was 114720 ULP), `IRR({-100,121})`, and the `{-10000,3000,4200,6800}` publication witness — OxFunc now *beats* the F# reference on the first two (F# 1 / 14571 ULP off). Residual: **irrational-root** cases where Excel's iteration-landing double differs from the |NPV|-minimal double — `IRR({-100,50,60})` `~80` ULP, mixed-5-flow `~14k` ULP, both ≈ F# now. Closing these needs Excel's *exact* iteration (guess / step / stop), the shared substrate with RATE/YIELD/ODDFYIELD (W103). | NUM-L | M1 | BUG-FUNC-028 (out-of-stream) |
| CUMPRINC | full-schedule (type 0) numeric drift `~6` ULP — distinct from the closed type=1 structural fix (BUG-FUNC-034) | NUM-L | M1 | G8 probe `CUMPRINC(0.1,12,1000,1,12,0)` |
| NPER | period-count drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| YIELDMAT | yield-at-maturity drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| TBILLYIELD | discounted-bill yield sub-ULP drift (`1` ULP on some settlements; array-lift itself correct) | NUM-S | M1 | G2 resweep 2026-06-20 |

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
