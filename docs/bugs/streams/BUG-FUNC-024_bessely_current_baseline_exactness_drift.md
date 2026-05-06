# BUG-FUNC-024: BESSELY current-baseline exactness drift

## Summary
- **Bug id**: `BUG-FUNC-024`
- **Opened**: `2026-04-30`
- **Status**: `open`
- **Owner workset**: `W089`
- **Bead**: `oxf-xp6p`
- **Split from**: `BUG-FUNC-023`

## Source Refs
- **Reported against ref**: `w089-comprehensive-seed-20260430-004`
- **Reproduced on ref**: `oxf-i45e-w089-repair-20260430-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `numeric_algorithm_exactness_gap`
- **Root cause summary**: `BESSELY` still follows the current local Bessel
  approximation path rather than Excel's current-baseline publication path for
  the affected scalar row. The row is not an array-admission problem and should
  not be repaired by a formula-specific lookup.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId oxf-i45e-w089-repair-20260430-001 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Representative row:

1. `=BESSELY(2.5,1)`: local `number:0x3fc2ad722ba3570c`,
   Excel `number:0x3fc2ad720e3ee754`.

## Repair Direction
1. Build a compact Excel probe grid over `x` and non-negative integer order,
   including values around the current branch boundaries and recurrence lanes.
2. Compare current local Bessel `Y` components against the Excel grid and
   high-accuracy public mathematical references.
3. Repair by substrate/algorithm only; do not patch this witness as a special
   case.
4. Keep the comparison policy as `exact_typed_bit_match_no_tolerance`.

## Evidence
1. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
2. `smart-fuzzer/runs/oxf-i45e-w089-repair-20260430-001/`
3. Parent stream: `docs/bugs/streams/BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md`
4. Bead: `oxf-xp6p`
5. W092 freshness replay:
   - `smart-fuzzer/runs/w092-scenario-engineering-cycle-001/` reproduced
     `=BESSELY(2.5,1)` with the same local and Excel numeric digests.
   - `smart-fuzzer/runs/w092-scenario-engineering-cycle-002/` records the same
     row as `known_residual` after the smart-fuzzer comparator was narrowed for
     this already-promoted exactness lane.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
