# Unswept Structural Sweep Findings (2026-05-28)

Status: `triage_recorded`

## 1. What this is

First structural-axis sweep of the surfaces the status map
(`smart-fuzzer/planning/FUNCTION_STATUS_MAP.md`) classed `unswept`. The
goal was the structural bug class (CHARTER §4.1): does OxFunc agree with
Excel on kind / error / shape / array-lift / coercion, ahead of more
numeric-drift LSB witnesses.

Generator: `smart-fuzzer/tools/Build-UnsweptStructuralProbes.ps1`
(value-taking deterministic non-volatile unswept surfaces; arity from
the dimension inventory; six structural probes per surface — baseline
scalar, array-lift, error-NA, empty-text, text-number, logical).

Run: `unswept-structural-sweep-001` via
`smart-fuzzer/tools/Run-ArraySupportTranche.ps1`. Excel `16.0` build
`20026`, workbook compatibility `2`, bit-exact typed comparison.

## 2. Rollup

- cases: `812` over `137` surfaces (`11` category tranches)
- `match`: `580`
- `structural_mismatch`: `116`
- `numeric_drift_gt1ulp`: `3`
- `numeric_drift_1ulp`: `5`
- `harness_blocked_excel`: `108`
- severity sub-tags: `kind_drift=86`, `error_code_drift=27`, `array_element_drift=3`

Of the `116` structural mismatches: `97` are real OxFunc-vs-Excel
divergences across `46` distinct surfaces; `19` are generator-invalid
(LAMBDA-family / locale / host — see §5).

## 3. Primary finding: array-lift + scalar-coercion gap (→ BUG-FUNC-028)

The dominant real cluster is one root-cause family, the same as
`BUG-FUNC-017`/`BUG-FUNC-018`: conversion / text / date / engineering
surfaces use scalar-only value preparation and neither (a) coerce a
scalar input the way Excel does, nor (b) lift over an array argument.

Cleanest witnesses (baseline scalar probe — a plain number argument):

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ASC(2)` | `#VALUE!` | `text:2` |
| `=DBCS(2)` | `#VALUE!` | `text:2` |
| `=DOLLAR(2)` | `#VALUE!` | `text:R2.00` |
| `=FIXED(2)` | `#VALUE!` | `text:2.00` |
| `=TEXT(2,2)` | `#VALUE!` | `text:2` |
| `=NUMBERVALUE(2)` | `#VALUE!` | `number:2` |
| `=VALUE(2)` | `#VALUE!` | `number:2` |

Array-lift witnesses (Excel spills elementwise; OxFunc collapses to a
scalar `#VALUE!`):

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ASC({2;3})` | `#VALUE!` | `array 2x1 [text:2 | text:3]` |
| `=CLEAN({2;3})` | `#VALUE!` | `array 2x1 [text:2 | text:3]` |
| `=DOLLAR({2;3})` | `#VALUE!` | `array 2x1 [text:R2.00 | text:R3.00]` |
| `=FIXED({2;3})` | `#VALUE!` | `array 2x1 [text:2.00 | text:3.00]` |
| `=DOLLARDE({2;3},2)` | `#VALUE!` | `array 2x1 [2 | 3]` |
| `=EOMONTH({2;3},2)` | `#VALUE!` | `array 2x1 [date | date]` |
| `=ARABIC({2;3})` | `#VALUE!` | `array 2x1 [#VALUE! | #VALUE!]` |
| `=BIN2OCT({2;3})` | `#VALUE!` | `array 2x1 [#NUM! | #NUM!]` |
| `=DECIMAL({2;3},2)` | `#VALUE!` | `array 2x1 [#NUM! | #NUM!]` |

Note the last three: even where Excel produces an *error* per element, it
still spills an array of errors, while OxFunc returns a single scalar
error. That is an array-admission gap distinct from the value being right.

Candidate surfaces in this family (structural mismatch on ≥1 probe;
local outcomes confirmed genuine, execution_status=ok):
`ARABIC`, `ASC`, `BIN2OCT`, `CLEAN`, `DBCS`, `DECIMAL`, `DELTA`,
`DOLLAR`, `DOLLARDE`, `EOMONTH`, `FACTDOUBLE`, `FIXED`, `GCD`, `GESTEP`,
`ISEVEN`, `ISOWEEKNUM`, `LCM`, `LOG`, `MULTINOMIAL`, `NETWORKDAYS`,
`NETWORKDAYS.INTL`, `NOT`, `NUMBERVALUE`, `OCT2DEC`, `QUOTIENT`, `ROMAN`,
`SQRTPI`, `STANDARDIZE`, `TBILLEQ`, `TBILLPRICE`, `TBILLYIELD`, `TEXT`,
`UNICODE`, `VALUE`, `WEEKDAY`, `WEEKNUM`, `WORKDAY`, `WORKDAY.INTL`,
`YEARFRAC`.

These are candidates, not per-function confirmed repairs. Repair must
determine per function whether the root cause is a missing coercion, a
missing array-lift, or an unimplemented kernel, and must re-replay each
under `Run-ArraySupportTranche.ps1` before closure. Routed to
`BUG-FUNC-028`.

## 4. Secondary findings

1. **1x1 publication seam (TRIMRANGE).** `=TRIMRANGE(2)` → OxFunc
   `array 1x1 [2]`, Excel scalar `2`. Same publication-seam class as
   `BUG-FUNC-026` / `HO-FN-010` (TAKE). Recorded there, not a new stream.
2. **Array-element 1-ULP drift (ACOT and similar).** `=ACOT({2;3})`
   matches shape but each cell differs by `1` ULP. The severity helper
   currently tags whole-array element drift as `structural_mismatch`
   (`array_element_drift`); these are really numeric-drift-inside-array.
   Tracked as a comparator refinement follow-up, not a structural bug.
3. **Statistical quantile edges.** `QUARTILE.EXC/INC`,
   `PERCENTILE.EXC/INC` produced structural mismatches on some probes —
   likely the same array-lift gap; fold into `BUG-FUNC-028` confirmation
   or `BUG-FUNC-021` per-function as triage decides.

## 5. Generator-invalid (excluded; generator refined)

These are not OxFunc bugs — the probe was malformed for the surface:

1. **LAMBDA-family** (`LAMBDA`, `MAKEARRAY`, `BYROW`, `BYCOL`, `MAP`,
   `GROUPBY`): need a lambda/callable fixture; deferred per W089. Excel
   returns `#CALC!`/`#N/A` for the malformed probe.
2. **Locale/host** (`JIS` → Excel `#NAME?` in this build/locale;
   `HYPERLINK` → needs host).

`Build-UnsweptStructuralProbes.ps1` now excludes the LAMBDA-family and
locale/host surfaces so future runs do not re-surface these as noise.

## 6. Harness-blocked (108)

Functions where the baseline probe is structurally wrong for the surface
(needs more/typed args or a reference): financial date functions
(`CUMIPMT`, `RECEIVED`, `PRICEDISC`, …), ranking over a reference
(`RANK`, `RANK.EQ`, `RANK.AVG`), text position/count functions
(`MID`, `LEFT`, `LEN`, `REPLACE`, …), and special-syntax forms
(`LET`, `PIVOTBY`). These need a reference-aware / typed-arity generator
(follow-up; see §7). They are not counted as bugs.

## 7. Follow-ups

1. `BUG-FUNC-028` repair confirmation per surface in §3.
2. Reference-aware probe generator for the `RefsVisibleInAdapter` and
   harness-blocked surfaces (lookup/database/financial-date/ranking).
3. Operator-structural probes for the `22` unswept operators.
4. Comparator refinement: split array-element numeric drift out of
   `structural_mismatch` (see §4.2).
5. Re-run `Build-FunctionStatusMap.ps1` — the `137` swept surfaces move
   out of `unswept`.

## 8. Second sweep (`unswept-structural-sweep-002`)

Generator extended: multi-alias names split to the first alias
(`"FIND, FINDB"` → `FIND`), full `arity.min` filled (was capped at 3,
which produced under-arity calls), text-category functions get a text
arg0, and empty-profile / empty-determinism surfaces are attempted
(filtered by known-bad rather than required-good). RAND-family excluded.

Run: `450` cases over `75` previously-skipped surfaces (bond / financial
/ date / CONVERT / info-predicate / regression). Result: `76` match,
`26` structural_mismatch, `348` harness_blocked.

The high harness-blocked rate confirms that the complex multi-arg
financial/bond functions (`ACCRINT`, `COUPDAYS`, `CONVERT`, …) cannot be
tested with naive numeric fill — they need a typed-argument generator
(valid date serials, basis codes, unit strings). They land in
`harness_blocked`, flagged for that generator, not counted as bugs.

New real findings (extend the BUG-FUNC-028 family unless noted):

1. **Array-lift gap on info predicates and date-value functions.**
   `ISERR`, `ISLOGICAL`, `ISNONTEXT`, `ISTEXT`, `ISODD`, `DATEVALUE`,
   `TIMEVALUE` return a scalar where Excel spills an array over an array
   argument (e.g. `=ISODD({2;3})` → local `#VALUE!`, Excel
   `array 2x1 [FALSE|TRUE]`). Added to `BUG-FUNC-028`.
2. **Error-propagation kind drift (`#VALUE!` vs `#N/A`).** New sub-class:
   `=DATEVALUE(NA())`, `=TIMEVALUE(NA())` return local `#VALUE!` where
   Excel propagates `#N/A`; `=ARRAYTOTEXT(NA())` returns local text
   `"#N/A"` where Excel propagates the `#N/A` error. Recorded as a
   `BUG-FUNC-028` sub-finding (error propagation, distinct from array-lift).
3. **IRR scalar error-code drift.** `=IRR("")`, `=IRR("2")`, `=IRR(TRUE)`
   return local `#VALUE!` where Excel returns `#NUM!`. Candidate;
   needs its own confirmation (not folded into BUG-FUNC-028).
4. **Regression family (`GROWTH`, `TREND`, `LINEST`, `LOGEST`).** Two
   issues: (a) single-point degenerate input — `=GROWTH(2)`/`=TREND(2)`
   return local `#NUM!` where Excel returns the value; (b) array inputs
   match shape but drift a few ULP per cell. Candidate for a separate
   regression-accuracy review, not BUG-FUNC-028.
5. **RANDARRAY** leaked through (blank determinism); excluded going
   forward. Not a bug — stochastic.
