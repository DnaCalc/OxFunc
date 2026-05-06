# BUG-FUNC-018: Successor scalar-parameter array-lift gap

## Summary
- **Bug id**: `BUG-FUNC-018`
- **Opened**: `2026-04-30`
- **Status**: `validated_local`
- **Owner workset**: `W090`
- **Bead**: `oxf-b39r`

## Source Refs
- **Reported against ref**: `8b140b50bf7f07153f87ac197cf99c470cad9ae8`
- **Reproduced on ref**: W092 widened successor replay through
  `w092-array-successor-max20x60-logical-cycle-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: W092 repair working tree pending commit
- **Ref notes**: W090 successor tranches used live Excel COM on Excel `16.0`
  build `19929`, workbook Compatibility Version `2`, with exact typed
  equality and bit-exact numeric comparison. No numeric tolerance was allowed.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: many value-only scalar parameter sites rejected inline
  array arguments with `#VALUE!` while current Excel spills elementwise or
  broadcast-shaped array results. The local repair added an observed
  dispatch-level scalar-array lift for the W090 successor lanes and family
  fixes where generic scalarization would be semantically wrong.

## W092 Reopened Evidence
W092 widened the successor generator from the prior W090 default
`139`-case aggregate through two bounded caches:

1. `smart-fuzzer/cache/w092-array-successor-max2x6-v0.json`: `194` cases.
2. `smart-fuzzer/cache/w092-array-successor-max3x9-v0.json`: `219` cases.
3. `smart-fuzzer/cache/w092-array-successor-max4x12-v0.json`: `233` cases.
4. `smart-fuzzer/cache/w092-array-successor-max10x30-v0.json`: `251` cases.
5. `smart-fuzzer/cache/w092-array-successor-max100x300-v0.json`: `253`
   cases, matching the `max20x60` build and showing local generator
   saturation for the current manifest corpus.

Fresh live Excel replay `w092-array-successor-max4x12-all-cycle-001` found
`14` local scalar-error vs Excel spill-array mismatches that remain in this bug
class rather than the numeric residual lane. A focused logical replay
`w092-array-successor-max10x30-logical-cycle-001` added three distinct
`SWITCH` variants in the same class:

1. `=BINOMDIST(2,4,0.25,{FALSE,FALSE})`
2. `=NORMDIST(42,40,1.5,{TRUE,TRUE})`
3. `=COMPLEX(3,4,{"j","j"})`
4. `=COMPLEX(1,2,{"x","x"})`
5. `=DOLLARFR(,{16,16})`
6. `=SWITCH(2,1,"a",{2,2},"b","other")`
7. `=SWITCH(3,1,"a",{2,2},"b","other")`
8. `=SWITCH(3,1,"a",{2,2},"b")`
9. `=SWITCH("1",1,"a",{"1","1"},"b")`
10. `=SWITCH(TRUE,1,"a",{TRUE,TRUE},"b")`
11. `=IFS("2",{"hit","hit"})`
12. `=ADDRESS(3,2,{4,4},FALSE,"Alpha")`
13. `=ADDRESS(3,2,4,{FALSE,FALSE},"Alpha")`
14. `=ADDRESS(3,2,4,FALSE,{"Alpha","Alpha"})`
15. `=ADDRESS(3,2,{1,1},TRUE,"Quarter 1")`
16. `=ADDRESS(3,2,1,{TRUE,TRUE},"Quarter 1")`
17. `=ADDRESS(3,2,1,TRUE,{"Quarter 1","Quarter 1"})`

The bead `oxf-b39r` was reopened on `2026-05-04` for this widened-seed
evidence. The W090 closure evidence below remains useful for the narrower
tranche, and the W092 repair evidence below records the current widened-axis
working-tree validation.

## W092 Repair Evidence
The W092 repair widened the dispatch-level observed scalar-array lift table for
the newly observed scalar parameter positions and narrowed error-result
fallback to avoid spilling unselected `IFS` result arrays. The focused Rust
regression is:

```powershell
cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib observed_scalar_array_lift_covers_w092_reopened_successor_positions
```

Smart-fuzzer replay artifacts:

1. `smart-fuzzer/runs/w092-bug-func-018-repair-max4x12-all-003`
2. `smart-fuzzer/runs/w092-bug-func-018-repair-max20x60-logical-001`
3. `smart-fuzzer/runs/w092-bug-func-018-repair-max20x60-all-001`

Current plateau replay
`smart-fuzzer/runs/w092-bug-func-018-repair-max20x60-all-001` executed `253`
cases over Excel `16.0` build `19929`, workbook Compatibility Version `2`, and
reported:

1. `211` exact typed bit matches.
2. `42` known residual rows routed to existing exactness lanes.
3. `0` unexpected mismatches.

The W092 reopened scalar-error vs Excel spill-array formulas listed above now
classify as exact typed bit matches in the repair replay. The remaining
non-exact rows are not array-admission failures in this stream; they remain
classified under the existing statistical exactness lanes such as
`BUG-FUNC-021`.

## W090 Repair Outcome
The W090 repair pass fixed the array-admission class represented by this stream.
Final replay artifacts:

1. `smart-fuzzer/runs/w090-repair-final-compatibility-001`
2. `smart-fuzzer/runs/w090-repair-final-engineering-functions-001`
3. `smart-fuzzer/runs/w090-repair-final-financial-functions-001`
4. `smart-fuzzer/runs/w090-repair-final-logical-functions-001`
5. `smart-fuzzer/runs/w090-repair-final-lookup-and-reference-functions-001`
6. `smart-fuzzer/runs/w090-repair-final-math-and-trigonometry-functions-001`
7. `smart-fuzzer/runs/w090-repair-final-statistical-functions-001`
8. `smart-fuzzer/runs/w090-repair-final-text-functions-001`

Aggregate final replay:

1. `139` cases executed.
2. `98` exact typed bit matches.
3. `41` remaining unexpected mismatches.
4. `0` local harness blockers.

The remaining `41` mismatches are no longer local `#VALUE!` array-admission
failures. They are numeric exactness drift inside scalar statistical and
compatibility-statistical kernels after array lift succeeds, and are split to
`BUG-FUNC-021` / bead `oxf-simj`.

## Reproduction
Generate successor cases:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-ArraySupportExecutableTranches.ps1
```

Run the final successor tranches recorded in:

```text
smart-fuzzer/planning/ARRAY_SUPPORT_SUCCESSOR_SWEEP_20260430.md
```

Final aggregate for this bug class:

1. `127` local `#VALUE!` vs Excel array-result mismatches.
2. `69` unique surfaces.
3. Argument axes:
   - `one_array_arg:arg1`: `56`
   - `one_array_arg:arg2`: `44`
   - `one_array_arg:arg3`: `27`

Representative mismatch anchors:

1. `BETADIST({0.5,0.5},2,3)`: local `#VALUE!`, Excel `array:1x2`.
2. `CONFIDENCE.T({0.05,0.05},1.2,30)`: local `#VALUE!`, Excel `array:1x2`.
3. `DOLLARFR(1.125,{8,8})`: local `#VALUE!`, Excel `array:1x2`.
4. `DROP({1,2,3;4,5,6;7,8,9},{1,1},-1)`: local `#VALUE!`, Excel `array:1x2`.
5. `SWITCH(2,{1,1},"a",2,"b","other")`: local `#VALUE!`, Excel `array:1x2`.

## Affected Surface Sample
The successor sweep observed this class across compatibility statistical
aliases, modern statistical functions, engineering scalar functions, selected
logical functions, lookup/reference dynamic-array parameter sites, and smaller
financial/text/matrix/math lanes. The representative surface list includes:

`ADDRESS`, `BETADIST`, `BETAINV`, `BINOMDIST`, `CHIDIST`, `CHIINV`,
`COMPLEX`, `CONCATENATE`, `CONFIDENCE`, `CONFIDENCE.T`, `CRITBINOM`,
`DOLLARFR`, `DROP`, `EXPAND`, `EXPONDIST`, `FDIST`, `FINV`, `GAMMADIST`,
`GAMMAINV`, `HYPGEOMDIST`, `IFS`, `IMABS`, `IMAGINARY`, `IMARGUMENT`,
`IMCONJUGATE`, `IMCOS`, `IMCOSH`, `IMCOT`, `IMCSC`, `IMCSCH`, `IMDIV`,
`IMEXP`, `IMLN`, `IMLOG10`, `IMLOG2`, `IMPOWER`, `IMREAL`, `IMSEC`,
`IMSECH`, `IMSIN`, `IMSINH`, `IMSQRT`, `IMSUB`, `IMTAN`, `LOGINV`,
`LOGNORMDIST`, `MUNIT`, `NEGBINOMDIST`, `NORMDIST`, `NORMINV`,
`NORMSDIST`, `NORMSINV`, `PERCENTILE`, `PERCENTRANK`, `POISSON`,
`QUARTILE`, `SERIESSUM`, `SORT`, `SWITCH`, `TAKE`, `TDIST`, `TINV`,
`TOCOL`, `TOROW`, `TRIMMEAN`, `UNICHAR`, `WRAPCOLS`, `WRAPROWS`, and
`Z.TEST`.

## Repair Direction
Do not repair all `69` surfaces as one blind edit. Split by semantic family and
reuse the W090 exact comparison harness after each bounded patch:

1. compatibility/statistical alias scalar lift,
2. engineering complex scalar lift,
3. logical branch/selection scalar lift where the selected result is array
   valued,
4. dynamic-array reshape scalar-parameter broadcasting,
5. remaining small financial/text/math/matrix lanes.

## Evidence
1. `smart-fuzzer/tools/Build-ArraySupportExecutableTranches.ps1`
2. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
3. `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/array_tranche_local_eval.rs`
4. `smart-fuzzer/planning/ARRAY_SUPPORT_SUCCESSOR_SWEEP_20260430.md`
5. Ignored run artifacts under `smart-fuzzer/runs/w090-successor-*-final-*`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required

No handoff was required for this local OxFunc repair. Residual exact numeric
drift is tracked separately under `BUG-FUNC-021`.
