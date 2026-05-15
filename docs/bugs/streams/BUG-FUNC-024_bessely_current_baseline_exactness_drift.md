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

## 2026-05-10 W097 R-E Cell-Ref Re-Sweep

W097 R-E re-replayed the witness and a `93`-case `(x, n)` band around
it under cell-ref Excel input plumbing. Tranche record:
`smart-fuzzer/planning/W097-R-E-bessely-cell-ref-resweep.md`.

The witness `=BESSELY(2.5, 1)` reproduces bit-for-bit:

- local `0x3fc2ad722ba3570c`
- Excel `0x3fc2ad720e3ee754`
- ULP distance `493,121,464`

Per-`n` ULP histogram across the `(x, n)` band:

| `n` | rows | matches | drifts | ULP min  | ULP median | ULP max  |
| --: | ---: | ------: | -----: | -------: | ---------: | -------: |
|  `0`|  `20` |     `1` |   `19` | `7.3E3`  |    `6.1E5` | `4.3E6`  |
|  `1`|  `20` |     `0` |   `20` | `3.3E6`  |    `4.9E8` | `2.1E12` |
|  `2`|  `20` |     `0` |   `20` | `2.6E5`  |    `2.0E8` | `4.2E11` |
|  `3`|  `11` |     `0` |   `11` | `2.0E6`  |    `5.7E7` | `1.4E12` |
|  `5`|  `11` |     `0` |   `11` | `2.0E6`  |    `5.6E7` | `1.3E12` |
| `10`|  `11` |     `0` |   `11` | `1.1E6`  |    `7.6E8` | `5.8E11` |

Direction: the BESSELY drift surface is broad and large. The recorded
witness is roughly the median of the `n=1` band; the kernel is
approximately uniformly off across the entire `(x, n)` sample. The
drift grows with `|n|` and stays in the `10^6..10^12` ULP range across
most of the surface. This confirms the "substrate/algorithm only; do
not patch this witness as a special case" repair direction recorded
in this stream.

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
6. W097 R-E cell-ref re-replay:
   - `smart-fuzzer/runs/W097-R-E-bessely-cellref/` (`93`-case band, witness
     bit-for-bit reproduced).
   - Tranche record:
     `smart-fuzzer/planning/W097-R-E-bessely-cell-ref-resweep.md`.
   - Driver: `smart-fuzzer/tools/Run-BesselyResweep.ps1`.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
