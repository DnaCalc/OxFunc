# BUG-FUNC-023: W089 non-statistical exactness and matrix shape drift

## Summary
- **Bug id**: `BUG-FUNC-023`
- **Opened**: `2026-04-30`
- **Status**: `resolved_by_partial_repair_and_split`
- **Owner workset**: `W089`
- **Bead**: `oxf-i45e`

## Source Refs
- **Reported against ref**: `w089-comprehensive-seed-20260430-004`
- **Reproduced on ref**: `w089-comprehensive-seed-20260430-004`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `working_tree_validated_by_oxf-i45e-w089-repair-20260430-001`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: the W089 comprehensive seed replay had combined four
  distinct lanes under one mixed bug stream:
  1. matrix 1x1 result publication was returning local arrays where Excel
     worksheet cells publish scalar values,
  2. `VDB` accumulated the declining-balance interval with `(book * factor) /
     life` where the Excel witness matches `book * (factor / life)`,
  3. `BESSELY` still has scalar numerical exactness drift,
  4. `MINVERSE` still has low-bit matrix-inversion exactness drift.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w089-comprehensive-seed-20260430-004 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Original confirmed representative rows:

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

## Repair Outcome
Superseded note, `2026-05-02`: follow-up nested `TYPE` probes showed that
Excel's worksheet-cell publication scalar is not the same as the function's
nested return value for array-returning functions. The earlier scalar collapse
for computed `1x1` matrix outputs has therefore been undone in OxFunc. The
direct-comparator scalar-vs-array observation remains a publication/comparator
seam issue, not a matrix function result-shape repair.

The `2026-04-30` repair pass landed the clear OxFunc-owned rows and split the
remaining exactness rows into focused successor streams.

Post-repair W089 replay:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId oxf-i45e-w089-repair-20260430-001 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Post-repair rollup:
1. `339` total cases.
2. `297` exact typed bit matches.
3. `39` unexpected mismatches.
4. `3` Excel harness blockers.
5. Excel environment: Excel `16.0`, build `19929`, workbook Compatibility
   Version `2`.

Rows repaired or reclassified under this stream:
1. `=VDB(2400,300,120,6,18)` now matches Excel exactly at
   `number:0x4078c4e5981baf06`.
2. `=MINVERSE(5)` final worksheet publication remains scalar in Excel, but
   OxFunc now preserves the internal `1x1` array result.
3. `=MMULT(5,2)` final worksheet publication remains scalar in Excel, but
   OxFunc now preserves the internal `1x1` array result.

Rows intentionally split to successor exactness streams:
1. `=BESSELY(2.5,1)` remains local `number:0x3fc2ad722ba3570c` versus Excel
   `number:0x3fc2ad720e3ee754`; successor `BUG-FUNC-024`, bead `oxf-xp6p`.
2. `=MINVERSE({1,2;3,4})` remains a `2x2` low-bit matrix drift; successor
   `BUG-FUNC-025`, bead `oxf-dzfk`.

## Evidence
1. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
2. `smart-fuzzer/runs/oxf-i45e-w089-repair-20260430-001/`
3. Successor stream: `docs/bugs/streams/BUG-FUNC-024_bessely_current_baseline_exactness_drift.md`
4. Successor stream: `docs/bugs/streams/BUG-FUNC-025_minverse_matrix_numeric_exactness_drift.md`
5. Bead: `oxf-i45e`

## Closure Checklist
- [x] fix landed or successor OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required: not required; no FEC/F3E or OxFml
  evaluator-facing seam change
