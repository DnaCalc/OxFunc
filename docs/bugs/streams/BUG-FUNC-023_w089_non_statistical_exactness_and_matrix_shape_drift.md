# BUG-FUNC-023: W089 non-statistical exactness and matrix shape drift

## Summary
- **Bug id**: `BUG-FUNC-023`
- **Opened**: `2026-04-30`
- **Status**: `open`
- **Owner workset**: `W089`
- **Bead**: `oxf-i45e`

## Source Refs
- **Reported against ref**: `w089-comprehensive-seed-20260430-004`
- **Reproduced on ref**: `w089-comprehensive-seed-20260430-004`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: the W089 comprehensive seed replay found exact
  numeric or shape mismatches outside the known PMT/PPMT/IPMT lane and outside
  the W090 statistical exactness stream. These need minimization before repair
  because they may split into matrix publication shape, matrix arithmetic
  rounding, Bessel-family approximation, and depreciation-family exactness
  lanes.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w089-comprehensive-seed-20260430-004 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Confirmed representative rows:

1. `=BESSELY(2.5,1)`: local `number:0x3fc2ad722ba3570c`,
   Excel `number:0x3fc2ad720e3ee754`.
2. `=VDB(2400,300,120,6,18)`: local `number:0x4078c4e5981baf07`,
   Excel `number:0x4078c4e5981baf06`.
3. `=MINVERSE({1,2;3,4})`: `array:2x2` shape matches, but three numeric
   cells differ by low bits.
4. `=MINVERSE(5)`: local `array:1x1:[number:0x3fc999999999999a]`, Excel
   scalar `number:0x3fc999999999999a`.
5. `=MMULT(5,2)`: local `array:1x1:[number:0x4024000000000000]`, Excel
   scalar `number:0x4024000000000000`.

All representative rows have local and Excel execution status `ok`.

## Repair Direction
Minimize before changing implementation:

1. split matrix scalar-input publication shape (`MINVERSE(5)`, `MMULT(5,2)`)
   from matrix numeric bit drift (`MINVERSE({1,2;3,4})`),
2. compare `BESSELY(2.5,1)` against scalar Excel probes and current algorithm
   branches,
3. compare `VDB(2400,300,120,6,18)` against the depreciation substrate and
   adjacent period/factor lanes,
4. keep the no-tolerance comparison policy unless a future scoped
   investigation explicitly proves a version/workbook-compatibility axis.

## Evidence
1. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
2. Bead: `oxf-i45e`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
