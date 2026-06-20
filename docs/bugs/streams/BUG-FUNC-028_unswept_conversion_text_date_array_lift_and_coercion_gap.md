# BUG-FUNC-028: Unswept conversion/text/date array-lift and scalar-coercion gap

## Summary
- **Bug id**: `BUG-FUNC-028`
- **Opened**: `2026-05-28`
- **Status**: `closed_signed_off` (2026-06-20)
- **Owner workset**: `W090` (array-support family; successor to BUG-FUNC-017/018)

## Resolution (2026-06-20, live Excel 16.0 build 20026)
Re-probed against Excel on the OxFunc evaluation surface (typed-arg local-eval — the
Category-2 path the original sweep used):

- **Array-lift / scalar-coercion class — already resolved (stale).** Every named surface now
  evaluates both the scalar and the `f({2;3})` array case bit-exact, including the surfaces
  that *consume* an array (aggregate to a scalar: `GCD`→4, `LCM`→12, `MULTINOMIAL`→10,
  `ARRAYTOTEXT`→"1, 2") versus the ones that *broadcast* (spill: `ISEVEN`, `QUOTIENT`, `ROMAN`,
  `BIN2OCT`, `IS*`, the date family, `TBILL*`, …). Tallies: Row-1 23/23, Row-2+Row-3 19/20,
  Row-4 4/4. The 2026-05-28 `#VALUE!` witnesses were fixed by the W090/W092 array-support work
  between then and now; this stream's rows were simply never reconciled. The lone non-match is
  `TBILLYIELD` at 1 ULP on one settlement — a numeric drift, not an array-admission failure,
  moved to the catalog G6 row.
- **Error-propagation sub-finding — fixed (2026-06-20).** `DATEVALUE`/`TIMEVALUE` (and their
  in-file siblings `DAYS360`/`DATEDIF`) collapsed any error argument to `#VALUE!` via
  `coerce_*.map_err(|_| Coercion)`; `ARRAYTOTEXT` stringified it to the text "#N/A". All now
  propagate the incoming error unchanged with its code preserved (`first_error_arg` /
  scalar-error guard), while errors *inside* an array argument stay textified
  (`ARRAYTOTEXT({1;#N/A})` → "1, #N/A"). 7/7 vs Excel; regression test
  `date_value_family::tests::error_args_propagate_unchanged_through_value_family`.

## Source Refs
- **Reported against ref**: working tree at run `unswept-structural-sweep-001`
- **Reproduced on ref**: run `unswept-structural-sweep-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: live Excel COM on Excel `16.0` build `20026`, workbook
  Compatibility Version `2`, exact typed equality with bit-exact numeric
  comparison via the cell-ref plumbing in `CellRefBatch.psm1`.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: a fresh swath of conversion / text / date /
  engineering surfaces — surfaced by the first structural sweep of the
  status-map `unswept` set — use scalar-only value preparation. They
  neither coerce a scalar input the way Excel does (e.g. number → text
  for `ASC`, number → parsed-number for `VALUE`) nor lift over an array
  argument (Excel spills elementwise, including arrays of per-element
  errors). This is the same root-cause family as `BUG-FUNC-017` and
  `BUG-FUNC-018`, on surfaces not previously exercised.

## Reproduction
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-UnsweptStructuralProbes.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId unswept-structural-sweep-001 `
  -CaseSetPath smart-fuzzer\cache\unswept-structural-probes-v0.json
```

Run `unswept-structural-sweep-001`: `812` cases, `580` exact typed bit
matches, `116` structural mismatches (`97` real across `46` surfaces,
`19` generator-invalid), `108` harness-blocked, `3` numeric_drift_gt1ulp,
`5` numeric_drift_1ulp.

### Cleanest witnesses (baseline scalar probe)
| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ASC(2)` | `#VALUE!` | `text:2` |
| `=DBCS(2)` | `#VALUE!` | `text:2` |
| `=DOLLAR(2)` | `#VALUE!` | `text:R2.00` |
| `=FIXED(2)` | `#VALUE!` | `text:2.00` |
| `=TEXT(2,2)` | `#VALUE!` | `text:2` |
| `=NUMBERVALUE(2)` | `#VALUE!` | `number:2` |
| `=VALUE(2)` | `#VALUE!` | `number:2` |

### Array-lift witnesses
| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ASC({2;3})` | `#VALUE!` | `array 2x1 [text:2 \| text:3]` |
| `=CLEAN({2;3})` | `#VALUE!` | `array 2x1 [text:2 \| text:3]` |
| `=DOLLAR({2;3})` | `#VALUE!` | `array 2x1 [text:R2.00 \| text:R3.00]` |
| `=EOMONTH({2;3},2)` | `#VALUE!` | `array 2x1 [date \| date]` |
| `=ARABIC({2;3})` | `#VALUE!` | `array 2x1 [#VALUE! \| #VALUE!]` |
| `=BIN2OCT({2;3})` | `#VALUE!` | `array 2x1 [#NUM! \| #NUM!]` |

## Candidate Surfaces
Structural mismatch on at least one probe, local outcome confirmed
genuine (execution_status=ok):
`ARABIC`, `ASC`, `BIN2OCT`, `CLEAN`, `DBCS`, `DECIMAL`, `DELTA`,
`DOLLAR`, `DOLLARDE`, `EOMONTH`, `FACTDOUBLE`, `FIXED`, `GCD`, `GESTEP`,
`ISEVEN`, `ISOWEEKNUM`, `LCM`, `LOG`, `MULTINOMIAL`, `NETWORKDAYS`,
`NETWORKDAYS.INTL`, `NOT`, `NUMBERVALUE`, `OCT2DEC`, `QUOTIENT`, `ROMAN`,
`SQRTPI`, `STANDARDIZE`, `TBILLEQ`, `TBILLPRICE`, `TBILLYIELD`, `TEXT`,
`UNICODE`, `VALUE`, `WEEKDAY`, `WEEKNUM`, `WORKDAY`, `WORKDAY.INTL`,
`YEARFRAC`.

This is a candidate list, not a per-function confirmed repair set. Repair
must, per surface, determine whether the divergence is a missing scalar
coercion, a missing array-lift, or an unimplemented kernel path, and
re-replay under `Run-ArraySupportTranche.ps1` before that surface is
closed.

### Second sweep additions (`unswept-structural-sweep-002`)
Array-lift gap confirmed on info predicates and date-value functions:
`ISERR`, `ISLOGICAL`, `ISNONTEXT`, `ISTEXT`, `ISODD`, `DATEVALUE`,
`TIMEVALUE`. Example: `=ISODD({2;3})` → local `#VALUE!`, Excel
`array 2x1 [FALSE|TRUE]`.

### Sub-finding: error-propagation kind drift (`#VALUE!` vs `#N/A`)
A distinct sub-class from the same sweep: some surfaces return local
`#VALUE!` (or stringify the error) where Excel propagates the incoming
error unchanged.

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=DATEVALUE(NA())` | `#VALUE!` | `#N/A` |
| `=TIMEVALUE(NA())` | `#VALUE!` | `#N/A` |
| `=ARRAYTOTEXT(NA())` | `text:#N/A` | `#N/A` |

Repair direction: propagate the incoming worksheet error rather than
re-classifying it to `#VALUE!` or stringifying it.

### Out of this stream (separate candidates)
- `IRR` scalar error-code drift (`#VALUE!` vs `#NUM!`).
- Regression family `GROWTH` / `TREND` / `LINEST` / `LOGEST`
  (single-point `#NUM!` vs value, plus per-cell ULP drift) — belongs to
  a regression-accuracy review, not this array-lift/coercion stream.
See `smart-fuzzer/planning/UNSWEPT_STRUCTURAL_SWEEP_FINDINGS_2026-05-28.md`
§8 for the full sweep-002 triage.

## Fix
Not yet fixed.

## Validation
Pending repair. Repair must re-run `unswept-structural-sweep-001` (or a
focused successor case set) and show the candidate surfaces moving to
`exact_typed_bit_match` / `match`.

## Similar-Risk Scan
- `BUG-FUNC-017` (math scalar array-lift, closed) and `BUG-FUNC-018`
  (successor scalar-parameter array-lift, validated_local) are the same
  family; the prepared-argument broadcast helper landed for those is the
  natural fix vehicle here.
- The `RefsVisibleInAdapter` and harness-blocked unswept surfaces
  (lookup/database/financial-date/ranking) are not covered by this
  stream — they need a reference-aware probe generator first
  (see the findings doc §7).

## Evidence
1. `smart-fuzzer/planning/UNSWEPT_STRUCTURAL_SWEEP_FINDINGS_2026-05-28.md`
2. `smart-fuzzer/tools/Build-UnsweptStructuralProbes.ps1`
3. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
4. ignored run artifacts under `smart-fuzzer/runs/unswept-structural-sweep-001/`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded (array-lift already-resolved; error-prop fixed 2026-06-20)
- [x] validation recorded (live Excel b20026 resweep: array-lift 46/47, error-prop 7/7)
- [x] root cause recorded per surface
- [x] similar-risk scan recorded (siblings `DAYS360`/`DATEDIF` fixed; `TBILLYIELD` 1-ULP → G6)
- [x] spec/matrix/contract updated if required (catalog G2 rows removed, OP_* retained)
- [ ] handoff filed if required
