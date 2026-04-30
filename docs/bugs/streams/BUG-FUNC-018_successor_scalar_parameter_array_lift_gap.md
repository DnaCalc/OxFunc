# BUG-FUNC-018: Successor scalar-parameter array-lift gap

## Summary
- **Bug id**: `BUG-FUNC-018`
- **Opened**: `2026-04-30`
- **Status**: `open`
- **Owner workset**: `W090`
- **Bead**: `oxf-b39r`

## Source Refs
- **Reported against ref**: `8b140b50bf7f07153f87ac197cf99c470cad9ae8`
- **Reproduced on ref**: current W090 successor sweep working tree
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`
- **Ref notes**: W090 successor tranches used live Excel COM on Excel `16.0`
  build `19929`, workbook Compatibility Version `2`, with exact typed
  equality and bit-exact numeric comparison. No numeric tolerance was allowed.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `test_gap`
- **Root cause summary**: many remaining value-only scalar parameter sites still
  reject inline array arguments with `#VALUE!` while current Excel spills
  elementwise or broadcast-shaped array results.

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
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
