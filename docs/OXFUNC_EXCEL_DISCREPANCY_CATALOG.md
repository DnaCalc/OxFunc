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

## G1 — Error-code & argument-domain guards (structural)

Excel returns an error (or saturates) where OxFunc returns a number, or vice-versa.

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| MOD | large-quotient → number vs Excel `#NUM!` (`MOD(1.005E14,1)`) | STR | M1 | BUG-FUNC-027 B1 |
| ATAN2 | `(tiny, huge-neg)` → `-π/2` vs Excel `#NUM!` (singleton; needs magnitude sweep) | STR | M0 | BUG-FUNC-027 B3 |
| ACOTH, ACOSH | near-1 argument-collapse: `ACOTH(1+ULP)` finite vs `#NUM!`; `ACOSH(1+1e-15)` non-zero vs `0` | STR | M0 | BUG-FUNC-027 C5 |

## G2 — Coercion, array-lift & kind/shape (structural)

Local `#VALUE!` where Excel coerces a scalar, spills over an array, or propagates an error.

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| CLEAN, GCD, LCM, QUOTIENT, ISEVEN, ISODD, NOT, LOG, SQRTPI, MULTINOMIAL, DECIMAL, DELTA, GESTEP, ARABIC, ROMAN, STANDARDIZE, OCT2DEC, BIN2OCT, UNICODE | scalar-numeric coercion gap (`ASC(2)`-style) and/or array-lift gap (`f({2;3})` → scalar `#VALUE!` vs Excel spill) | STR | M1 | BUG-FUNC-028 |
| EOMONTH, ISOWEEKNUM, WEEKDAY, WEEKNUM, NETWORKDAYS(.INTL), WORKDAY(.INTL), YEARFRAC | date-serial coercion / array-lift gap | STR | M1 | BUG-FUNC-028 |
| TBILLEQ, TBILLPRICE, TBILLYIELD | scalar coercion / array-lift gap | STR | M1 | BUG-FUNC-028 |
| ISERR, ISLOGICAL, ISNONTEXT, ISTEXT | array-lift gap (`IS*({2;3})` → `#VALUE!` vs Excel per-cell) | STR | M1 | BUG-FUNC-028 (sweep-002) |
| DATEVALUE, TIMEVALUE, ARRAYTOTEXT | error-propagation kind drift: `f(NA())` → `#VALUE!`/stringified vs Excel `#N/A` (the *parse* path is Category 1, locale) | STR | M1 | BUG-FUNC-028 (error-prop sub-finding) |
| OP_* binary operators | array-lift value-surface + ordinary-broadcast gaps | STR | HO | BUG-FUNC-001/002 (HO-FN-005) |

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
| MOD | intermediate-truncation drift (up to `~9.5E10` ULP) | NUM-L | M1 | BUG-FUNC-027 C2 |
| TAN, SIN, COT, SEC, CSC | moderate-large argument-reduction drift (Cody-Waite vs extended-π; up to `~3.3E12` ULP) | NUM-L | M1 | BUG-FUNC-027 C3 |
| ATANH | near-`±1` precision (`~1.5E13` ULP); repair = `log1p` form | NUM-L | M1 | BUG-FUNC-027 C4 |
| ACOTH | large-argument series (`ACOTH(x)=ATANH(1/x)`); `~1.2E14` ULP band | NUM-L | M1 | BUG-FUNC-027 C5 |
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
| ACCRINT | periodic accrual returns exactly **half** of Excel (divide-by-frequency defect) | NUM-L | M1 | BUG-FUNC-030 |
| YIELD | root-finder `#NUM!` where Excel converges (`~0.0857`) | STR | M1 | BUG-FUNC-031 |
| ODDFPRICE, ODDFYIELD | odd-first-period `#NUM!` where Excel computes | STR | M1 | BUG-FUNC-032 |
| RATE | structural lane signed off (2026-06-20): default-guess mortgage root now converges and Excel returns a number, not `#NUM!`. Residual `~586` ULP vs Excel (`0.0041666445363460975` vs `0.004166644536345589`) — distinct numeric drift in the solver substrate | NUM-L | M1 | BUG-FUNC-009 (bit-parity) / W103 |
| IRR | scalar error-code drift `#VALUE!` vs Excel `#NUM!` | STR | M1 | BUG-FUNC-028 (out-of-stream) |
| CUMPRINC | full-schedule (type 0) numeric drift `~6` ULP — distinct from the closed type=1 structural fix (BUG-FUNC-034) | NUM-L | M1 | G8 probe `CUMPRINC(0.1,12,1000,1,12,0)` |
| YIELDDISC | discounted-bill yield drift `~5` ULP | NUM-L | M1 | G8 probe `YIELDDISC(44013,44562,95,100,0)` |
| NPER | period-count drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |
| YIELDMAT | yield-at-maturity drift (`1` ULP) | NUM-S | M1 | G8 probe 2026-06-19 |

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
  → G3; CONVERT → G4; CUMPRINC/YIELDDISC/NPER/YIELDMAT → G6.
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
