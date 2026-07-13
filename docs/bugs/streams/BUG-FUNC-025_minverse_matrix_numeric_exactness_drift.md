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

## 2026-05-10 W097 R-F Cell-Ref Re-Sweep

W097 R-F replayed the witness and a `45`-matrix band of 2x2 / 3x3 /
4x4 random and structured matrices under cell-ref Excel input
plumbing. Each result cell is read scalar-by-scalar via
`INDEX(MINVERSE(<range>), r, c)`. Tranche record:
`smart-fuzzer/planning/W097-R-F-minverse-cell-ref-resweep.md`.

Witness `=MINVERSE({1,2;3,4})` reproduces bit-for-bit — three of the
four result cells drift by exactly one ULP, the `(1,0)` cell is
exact, matching the historical `BUG-FUNC-025` witness pair:

| (r, c) | local bits             | Excel bits             | ULP   |
| ------ | ---------------------- | ---------------------- | ----- |
| (0, 0) | `0xbffffffffffffffe`   | `0xbfffffffffffffff`   | `1`   |
| (0, 1) | `0x3feffffffffffffe`   | `0x3fefffffffffffff`   | `1`   |
| (1, 0) | `0x3ff7ffffffffffff`   | `0x3ff7ffffffffffff`   | `0`   |
| (1, 1) | `0xbfdfffffffffffff`   | `0xbfdffffffffffffe`   | `1`   |

Per-kind summary across `45` matrices / `440` cells: matches `217`,
drifts `223`, kind drift `0`, blocked `0`.

Highlights:

- **Identity and diagonal matrices** (any size): bit-exact across
  every cell. Algorithm-choice impact zero.
- **Random matrices** (well-conditioned): typically `0..7` ULP per
  cell. One `4x4` random outlier reached `2050` ULP.
- **Hilbert matrices**: drift grows with `n` — `22` ULP for `3x3`,
  `352` ULP for `4x4`. This reflects condition-number amplification
  of the Gauss-Jordan rounding-path delta, not a kernel bug.
- **Diagonally-dominant matrices**: ~`1..2` ULP per cell.

The R-F case set is the appropriate regression-validation gate when
a future repair lands a different matrix-inversion substrate
(LU-solve / Crout / Cholesky). Anything worse than the per-kind
floor recorded above is a regression.

## 2026-07-13 W109 Kernel Landed — Gauss-Jordan → Doolittle LU

The inversion kernel `inverse_kernel` in
`crates/oxfunc_core/src/functions/matrix_family.rs` has been replaced. It
previously ran **Gauss-Jordan on `[A|I]`** (the algorithm W109 had already
ruled out); it now runs the identified **Doolittle LU + partial pivot +
per-column unit-vector solve with division-form back-substitution, plain
binary64** (each multiply/subtract rounds separately — no FMA contraction;
the multiplier is a true division). The determinant kernel already carried
the same LU elimination, so the two matrix kernels are now consistent.

Verified end-to-end through the compiled surface (`eval_surface_value_call`
via `matrix_local_eval`), scored against cached live-Excel bits
(build 20131):

| Corpus | Gauss-Jordan (old) | Doolittle LU (new) |
| ------ | ------------------ | ------------------ |
| 3x3 (159 cells, 21 matrices) | `80` | `150` |
| 4x4 (448 cells, 28 matrices, `m4b`) | `102` | `448` — **perfect** |

Net **+416 cells** (`182 → 598` of `607`). The old-kernel passing set is a
strict subset of the new (0 regressions, confirmed cell-by-cell).
`cargo test -p oxfunc_core --lib` = `1502/1502` green; the 11 matrix unit
tests (singular→#NUM guard, seed inverse, non-square→#VALUE) all preserved —
the `EPS = 1e-12` singularity threshold is unchanged, so only the algorithm
changed.

**Residual (still open):** `9` cells on the 3x3 corpus (+ 2 of the 4 on the
`{1,2;3,4}` witness) drift by exactly `+1`/`+2` ULP. All are ill-conditioned
small-determinant cases (tridiag det 4, `[[1,2,3],[4,5,6],[7,8,10]]` det -3,
near-identity 1e-8) where Excel lands 1 ULP off the exact representable
value. Ruled out as explanations this round: the full 32-variant solve-ordering
sweep (all ≤150), and x87 80-bit extended registers (best 110, strictly worse
— MINVERSE is plain SSE2 double, not a legacy x87 body). The residual
*direction flips* (Excel further from exact on tridiag/integer, closer on
near-identity), so it is a genuinely different op-graph for those cells, not a
uniform extra/fewer rounding. Tracked as a targeted decoder probe in
`docs/function-lane/DISCREPANCY_RULED_OUT_LEDGER.csv`. **4x4 has zero residual.**

The W097 R-F per-kind drift floor recorded above no longer applies as-is: the
Gauss-Jordan condition-number amplification (Hilbert 4x4 → 352 ULP) is gone;
the new gate is the m4b 448/448 bit-exact result plus the 3x3 150/159.

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
6. W097 R-F cell-ref re-replay:
   - `smart-fuzzer/runs/W097-R-F-minverse-cellref/` (`45` matrices,
     `440` per-cell comparisons; witness reproduced bit-for-bit;
     per-kind drift floor recorded).
   - Tranche record:
     `smart-fuzzer/planning/W097-R-F-minverse-cell-ref-resweep.md`.
   - Driver: `smart-fuzzer/tools/Run-MinverseResweep.ps1`.
   - Local matrix evaluator:
     `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/matrix_local_eval.rs`.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
