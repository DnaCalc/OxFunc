# BUG-FUNC-025: MINVERSE matrix numeric exactness drift

## Summary
- **Bug id**: `BUG-FUNC-025`
- **Opened**: `2026-04-30`
- **Status**: `open`
- **Owner workset**: `W089`
- **Bead**: `oxf-dzfk`
- **Split from**: `BUG-FUNC-023`

## Source Refs
- **Reported against ref**: `w089-comprehensive-seed-20260430-004`
- **Reproduced on ref**: `oxf-i45e-w089-repair-20260430-001`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `matrix_numeric_algorithm_exactness_gap`
- **Root cause summary**: the matrix inversion kernel still takes a
  Gauss-Jordan rounding path that differs from Excel by low bits on the seed
  `2x2` inverse. The earlier scalar `1x1` publication note has been
  reclassified as an OxFml/DNA Calc publication-seam concern rather than an
  OxFunc matrix result-shape repair.

## Reproduction
Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId oxf-i45e-w089-repair-20260430-001 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Representative row:

1. `=MINVERSE({1,2;3,4})`: local
   `array:2x2:[number:0xbffffffffffffffe|number:0x3feffffffffffffe|number:0x3ff7ffffffffffff|number:0xbfdfffffffffffff]`,
   Excel
   `array:2x2:[number:0xbfffffffffffffff|number:0x3fefffffffffffff|number:0x3ff7ffffffffffff|number:0xbfdffffffffffffe]`.

## Repair Direction
1. Build a compact Excel probe grid over small `1x1`, `2x2`, and `3x3`
   matrices, including integer, fractional, pivoting, near-singular, and
   identity-adjacent lanes.
2. Compare Gauss-Jordan, LU-solve, and any selected public numerical method
   against the Excel grid before changing the kernel.
3. Repair by matrix algorithm/rounding path only; do not nudge individual
   witness cells.
4. Keep the comparison policy as `exact_typed_bit_match_no_tolerance`.

## Evidence
1. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
2. `smart-fuzzer/runs/oxf-i45e-w089-repair-20260430-001/`
3. Parent stream: `docs/bugs/streams/BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md`
4. Bead: `oxf-dzfk`
5. W092 freshness replay:
   - `smart-fuzzer/runs/w092-scenario-math-cycle-001/` reproduced
     `=MINVERSE({1,2;3,4})` as `known_residual`.
   - The same run also classified `=MINVERSE(5)` and `=MMULT(5,2)` as
     `adapter_or_seam_mismatch` under `HO-FN-010`, not as matrix-kernel repair
     targets.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
