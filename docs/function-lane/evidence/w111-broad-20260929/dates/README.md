# Date conversion and calendar observations

Status: `in_progress`; `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Owner: `oxf-mwue.28.7`, BUG-FUNC-056. Open lanes: shared numeric-text context, unsupported WEEKNUM selectors,
combined validation and canonical status reconciliation.

The public COM oracle is Excel 16.0 build 20430, 64-bit, Compatibility Version 2,
1900 date system. Channel is unverified. All numerical arguments use exact
binary64 bits through Value2. No binary internals were inspected. The three
discovery captures retain 35,528 observations, including contradictory repeated
WEEKNUM answers; `artifact-manifest.json` records source and retained hashes.

## Current numerical candidate

- DAY/MONTH/YEAR and HOUR/MINUTE/SECOND add `0.5 / 86400` to the serial before
  splitting its integer date and fractional time. This operation order differs
  from rounding `serial * 86400` or rounding the fractional seconds. Negative raw
  serials and rounded dates at or beyond 2,958,466 reject with `#NUM!`.
- For fractional time `f`, compute `h = f * 24`, `hour = floor(h)`,
  `m = (h - hour) * 60`, `minute = floor(m)` and
  `second = floor((m - minute) * 60) mod 60`. A first independent holdout
  separated the minute graph; subsequent controls separated the second graph.
  The revised numerical graph passed a fresh 72,930-row independent holdout.
- DAYS, EDATE, EOMONTH, WEEKNUM and ISOWEEKNUM use raw serial admission
  `0 <= serial < 2958466` followed by truncation. They do not inherit half-second
  date-part rounding. EDATE/EOMONTH reject target years outside 1900–9999, before
  constructing the target day. The existing Gregorian February clamp is retained,
  including EDATE(60, 0) = 59.
- WEEKDAY uses the rounded date-part serial. Its selector uses the date integer
  conversion below. WEEKNUM uses that selector conversion on its supported
  selector lane, with the separate raw serial rule.
- DATE arguments use a tolerant floor: if `u = ceil(x)`, admit `u` when `u - x`
  is at most `2049 / 8589934592` for positive `u`, or half that for nonpositive
  `u`; otherwise take the ordinary floor. Exact neighboring-bit probes distinguish
  this from plain truncation and decimal significant-digit rounding.
- The DATE year conversion first caps the rounded year at 10,000. Values fitting
  signed 32-bit range contribute their low 16 bits, otherwise zero bits. Interpret
  these as signed 16-bit, then add 1900 when that signed value is below 1900.
  Keep a resulting negative year until month normalization. This is an observed
  behavioral model, not an identification of unpublished implementation code.
  Months admit integers from -32,767 through 32,766. A day outside signed 16-bit
  range becomes 32,767. Normalize the month, require year 1900–9999, then add the
  day offset to the month-start serial. Applying the fictitious 1900 leap day at
  month start makes DATE(1900, 2, 30) = 61. Final serial must be 0–2,958,465.

## Exercised evidence

`w111_date_live_replay.rs` uses actual function dispatch and retained expected
results. It admits 6,777 boundary, 12,177 conversion, 11,242 integer, 33,992 first
holdout, 14,316 refinement and 72,930 second holdout rows: 151,434 exact numeric
comparisons. The other 5,332 rows use unsupported WEEKNUM
selectors and remain explicitly withheld. The test asserts each captured result;
it does not generate expected numerical answers from the candidate.

The 31 existing date-family Rust tests also pass. Lean DATE now has the same
finite-number preparation, integer-width and month-start calendar model.
DatePartsFamily supplies an executable binary64 split binding and DAYS truncation
model; DateWeekFamily supplies serial/target admission and integer weekday rules.
The focused Lean build passes eight jobs. These exercised models cover the
declared numerical slice; they do not establish missing locale/clock preparation.

`gen_date_heldout_20260929.py` freezes the three Rust source hashes before
generating a fresh, independent seed: 33,992 numerical rows and 173 typed cases.
The first frozen candidate matched 33,440/33,992 numeric observations. Its 552
failures comprise 551 DATE year-conversion cases and one MINUTE boundary. These
failures are retained without overwriting the initial candidate freeze. The
refined graph is described above. All failed observations remain retained;
`first-candidate-failed-replay.log` preserves the exercised failure report.

The typed holdout has 144 exact results, 22 semantic mismatches, and seven unary
empty-call formulas rejected by Excel's formula entry. The semantic mismatches
expose DATE/DAYS array lifting and function-specific logical/missing-argument
admission. `typed-first/` retains cases, both initial outcomes, and comparisons.
After typed controls, the candidate matched 1,275 qualified cases across the
first packet and 1,137-row refinement. Seven rejected unary empty-call formulas
and 28 unsupported WEEKNUM-selector cases are withheld.

The second independent typed packet had 743/780 matches. Its 37 failures exposed
whole-array error collapse in DAY/MONTH/YEAR/HOUR/MINUTE/SECOND. The repair maps
coercion and domain errors per cell; all 780 retained cases now replay exactly.
A fresh 720-case packet then matched all 360 direct arrays. Its 360 reference
arrays exposed a local harness gap assembling ranges from individual fixtures;
the resolver now assembles these exact fixtures and the actual-dispatch replay
matches all 720 cases. The four typed packets have 2,775 qualified exact outcomes
in total. The 35 withheld cases remain visible: seven formula-entry rejections
and 28 unsupported WEEKNUM selectors.

DATE, DAYS and all other tested date functions preserve array shape. DAYS scans
worksheet errors before text coercion. DATE/DAYS/WEEKDAY accept logicals and
prepare explicit missing slots as zero; EDATE/EOMONTH and WEEKNUM reject logicals
and missing required slots. WEEKNUM's optional missing selector defaults to one.

## Unsupported WEEKNUM observations

The first batch contains 29 contradictory duplicate-input pairs. A separately
shuffled batch repeats 945 distinct input tuples three times; 382 tuples have
more than one observed result, sometimes different numeric week numbers as well
as `#NUM!`. Inputs use ordinary finite nonzero/positive-zero numbers. These are
unresolved oracle observations, not permission to add a lookup table, infer a
binary implementation, or silently claim deterministic parity. Further direct
fixture/formula readback controls are retained in
`weeknum-direct-observations.json`: 1,008 observations, 84 distinct tuples, two
fresh Excel sessions, multithreaded calculation enabled/disabled, and ordinary,
full and rebuilt recalculation. Every input and formula is read back. Nine
tuples still produced different results; all involve unsupported -1 or 100
selectors. All 30 supported-selector tuples remained stable. The mechanism is
unknown; `weeknum-direct-analysis.json` retains every differing outcome and
configuration. This supports withholding the unstable lane, not fitting it.

## Scope and integration

Numerical repairs preserve current Rust signatures, metadata and FEC/F3E shapes.
The separate implicit text parser/context change is tracked by BUG-FUNC-057 and
HO-FN-022. XLL recreation of workbook date systems and host locale remains a
verification-seam obligation. This packet is qualified to the captured 1900/CV2
baseline and does not promote an entire function to phase completion.
