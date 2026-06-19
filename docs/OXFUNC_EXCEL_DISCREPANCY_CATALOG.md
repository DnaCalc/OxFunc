# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-06-19`

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
| EXP (+ DEGREES/RADIANS/FACT/FACTDOUBLE audit) | overflow → `+Inf` vs Excel `#NUM!`; same `finite_or_num` pattern as the fixed SINH/COSH | STR | M1 | `oxf-vgxs` |
| COTH, CSCH, SECH | large-`\|x\|` → `NaN`/`Inf` vs Excel saturates (`±1` / `0`) — needs a saturation guard, NOT `finite_or_num` | STR | M1 | BUG-FUNC-027 C3.h |
| MOD | large-quotient → number vs Excel `#NUM!` (`MOD(1.005E14,1)`) | STR | M1 | BUG-FUNC-027 B1 |
| COS, SIN, TAN (+ COT/SEC/CSC) | very-large argument → number vs Excel `#NUM!` (threshold ~`2^48`, to pin) | STR | M1 | BUG-FUNC-027 B2 |
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
| BINOMDIST, NORMDIST, COMPLEX, DOLLARFR, SWITCH, IFS, ADDRESS | scalar-parameter array-lift gap (scalar error vs Excel spill) | STR | M3 | BUG-FUNC-018 |
| OP_* binary operators | array-lift value-surface + ordinary-broadcast gaps | STR | HO | BUG-FUNC-001/002 (HO-FN-005) |

## G3 — Numeric exactness: special & statistical functions

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMADIST, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, NORMSDIST, NORMSINV, TDIST, TINV, CONFIDENCE.T, Z.TEST | distribution scalar numeric drift (repair by numerical substrate, not per-case) | NUM-L | M2 | BUG-FUNC-021 / KED-STAT-001 / `oxf-simj` |
| GAMMA | negative-non-integer reflection drift (`~1290` ULP after cell-ref resweep) | NUM-L | M1 | BUG-FUNC-027 C1 |
| BESSELY | Bessel-Y scalar numeric drift | NUM-L | M1 | BUG-FUNC-024 / KED-BESSEL-001 / `oxf-xp6p` |

## G4 — Numeric exactness: elementary & trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| MOD | intermediate-truncation drift (up to `~9.5E10` ULP) | NUM-L | M1 | BUG-FUNC-027 C2 |
| TAN, SIN, COT, SEC, CSC | moderate-large argument-reduction drift (Cody-Waite vs extended-π; up to `~3.3E12` ULP) | NUM-L | M1 | BUG-FUNC-027 C3 |
| ATANH | near-`±1` precision (`~1.5E13` ULP); repair = `log1p` form | NUM-L | M1 | BUG-FUNC-027 C4 |
| ACOTH | large-argument series (`ACOTH(x)=ATANH(1/x)`); `~1.2E14` ULP band | NUM-L | M1 | BUG-FUNC-027 C5 |
| COMBIN, COMBINA, PERMUT, PHI, GAUSS, FACTDOUBLE, ERF.PRECISE, ERFC.PRECISE | `±1` ULP where OxFunc is analytic-exact and Excel is off — match-Excel | NUM-S | M1 | BUG-FUNC-027 (combinatorial group) |

## G5 — Numeric exactness: matrix

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| MINVERSE | multi-cell inversion low-bit drift (`1×1` publication already fixed) | NUM-L | M2 | BUG-FUNC-025 / KED-MATRIX-001 / `oxf-dzfk` |
| MMULT | matrix numeric / `scalar-vs-1×1` shape drift | NUM-L | M1 | BUG-FUNC-023 / `oxf-i45e` |

## G6 — Financial: exactness, computation & solver

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| PMT, PPMT (IPMT adjacent) | annuity publication exactness drift (`21` non-zero-rate mismatches); KED known-residual, hold until reopened | NUM-L | M3 | BUG-FUNC-015 / KED-FIN-001 / `oxf-fckb` |
| ACCRINT | periodic accrual returns exactly **half** of Excel (divide-by-frequency defect) | NUM-L | M1 | BUG-FUNC-030 |
| YIELD | root-finder `#NUM!` where Excel converges (`~0.0857`) | STR | M1 | BUG-FUNC-031 |
| ODDFPRICE, ODDFYIELD | odd-first-period `#NUM!` where Excel computes | STR | M1 | BUG-FUNC-032 |
| RATE | default-guess solver no-convergence (mortgage lane) → `#NUM!` vs small positive rate | STR | M3 | BUG-FUNC-009 |
| IRR | scalar error-code drift `#VALUE!` vs Excel `#NUM!` | STR | M1 | BUG-FUNC-028 (out-of-stream) |

## G7 — Comparison & misc semantics

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| operators, criteria families, SWITCH | numeric comparison tolerance lane (truncation-style vs round-to-nearest) | NUM-S | M3 | BUG-FUNC-004 (HO-FN-008) |

## G8 — Untriaged (no stream yet) — needs a severity/maturity probe

These are `mixed_or_open` surfaces from the smart-fuzzer status map: a genuine non-match
in the latest run, with no stream and no triaged severity. **First action is a witness +
severity probe**, then promote into G1–G7 (or the context-sensitive catalog).

| Cluster | Surfaces | Likely note |
|---------|----------|-------------|
| Regression / forecast | FORECAST, FORECAST.LINEAR, TREND, GROWTH, LINEST, LOGEST | single-point `#NUM!` vs value + per-cell ULP; likely one shared regression kernel |
| Percentile / quartile | PERCENTILE.EXC, PERCENTILE.INC, QUARTILE.EXC, QUARTILE.INC | interpolation lane |
| Paired-sample tests | CHISQ.TEST, CHITEST, F.TEST, FTEST | array-pair statistic |
| Finance (re-sweep) | NPV, NPER, CUMPRINC, XNPV, YIELDDISC, YIELDMAT | NPV/CUMPRINC/XNPV overlap recently-closed streams — confirm cleared by re-sweep before opening |
| Misc scalar | ACOT, CONVERT, GAUSS, PHI | scalar value/threshold |

---

## Pointers

- Category boundary & policy: [ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md)
- Context-sensitive (Category 1) catalog: `smart-fuzzer/corpus/context_sensitive_catalog/`
- Severity vocabulary & comparison policy: `CHARTER.md` §4.1; smart-fuzzer `Get-StandardSeverityClass`
- Transferable lessons: [OXFUNC_FIX_LEARNING_LOG.md](OXFUNC_FIX_LEARNING_LOG.md)
- Bit-exact Excel comparison plumbing: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`
