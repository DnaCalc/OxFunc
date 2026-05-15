# W097 R-F — BUG-FUNC-025 MINVERSE matrix cell-ref re-sweep

Status: `tranche_complete`

Owning workset: `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md`
Owning bead: `oxf-ic1h.6`
Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`

## 1. What this record is

R-F re-replays the BUG-FUNC-025 MINVERSE witness `=MINVERSE({1,2;3,4})`
plus a fixed band of 2x2 / 3x3 / 4x4 random and structured matrices
under cell-ref Excel input plumbing. Matrix elements are written via
`Range.Value2` (bit-exact f64 round-trip) and the formula is
`=INDEX(MINVERSE(<range>), r, c)` so each result cell is a scalar
bit-exact comparison.

## 2. New tooling

- `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/matrix_local_eval.rs`
  — minimal Rust binary that takes `args_typed`-style cases (with
  matrix args) and emits per-cell outcomes for matrix-returning OxFunc
  functions.
- `smart-fuzzer/tools/Run-MinverseResweep.ps1` — PowerShell driver that
  generates the case set, runs `matrix_local_eval`, and routes each
  result cell through `Invoke-ExcelCellRefBatch` (matrix-aware path
  that already lives in W097 R-B's `CellRefBatch.psm1`).
- `smart-fuzzer/tools/CellRefBatch.psm1` matrix path uses
  `INDEX(MINVERSE(<range>), r, c)` to extract each result cell as a
  scalar so array-returning functions do not spill into the
  error-type companion cell.

## 3. Run

`smart-fuzzer/runs/W097-R-F-minverse-cellref/`

- Cases: `45` matrices
  - 2x2: `1` BUG-FUNC-025 witness + `5` structured + `8` random = `14`
  - 3x3: `4` structured (identity, diag, tridiag, hilbert) +
    `8` random + `4` diag-dominant = `16`
  - 4x4: `3` structured (identity, diag, hilbert) +
    `8` random + `4` diag-dominant = `15`
- Cells compared: `440`
- Rollup: matches `217`, drifts `223`, kind drift `0`, blocked `0`
- Excel environment: `16.0` build `19929`

## 4. Witness confirmation (BUG-FUNC-025 MIV-0001)

`=MINVERSE({1,2;3,4})` cell-by-cell:

| (r, c) | local bits             | Excel bits             | ULP   |
| ------ | ---------------------- | ---------------------- | ----- |
| (0, 0) | `0xbffffffffffffffe`   | `0xbfffffffffffffff`   | `1`   |
| (0, 1) | `0x3feffffffffffffe`   | `0x3fefffffffffffff`   | `1`   |
| (1, 0) | `0x3ff7ffffffffffff`   | `0x3ff7ffffffffffff`   | `0` (match) |
| (1, 1) | `0xbfdfffffffffffff`   | `0xbfdffffffffffffe`   | `1`   |

This **reproduces exactly** the BUG-FUNC-025 recorded witness pair:
the local OxFunc result `[number:0xbffffffffffffffe | 0x3feffffffffffffe |
0x3ff7ffffffffffff | 0xbfdfffffffffffff]` versus the Excel result
`[number:0xbfffffffffffffff | 0x3fefffffffffffff | 0x3ff7ffffffffffff |
0xbfdffffffffffffe]`. Three of the four cells drift by exactly one ULP.

## 5. Per-kind summary

| Kind                  | matrices | cells | match | drift | max ULP   |
| --------------------- | -------: | ----: | ----: | ----: | --------: |
| identity (any size)   |       3  |   29  |   29  |    0  | `0`       |
| diagonal (any size)   |       3  |   29  |   29  |    0  | `0`       |
| rotation_45 (2x2)     |       1  |    4  |    3  |    1  | `1`       |
| near_singular (2x2)   |       1  |    4  |    4  |    0  | `0`       |
| negative_offdiag (2x2)|       1  |    4  |    4  |    0  | `0`       |
| tridiag (3x3)         |       1  |    9  |    2  |    7  | `1`       |
| hilbert_3 (3x3)       |       1  |    9  |    0  |    9  | `22`      |
| hilbert_4 (4x4)       |       1  |   16  |    0  |   16  | `352`     |
| random 2x2            |       8  |   32  |   25  |    7  | `1`       |
| random 3x3            |       8  |   72  |   29  |   43  | `5`       |
| random 4x4            |       8  | `128` |   30  |   98  | `2050`    |
| diag-dominant 3x3     |       4  |   36  |   18  |   18  | `2`       |
| diag-dominant 4x4     |       4  |   64  |   43  |   21  | `2`       |
| witness `{1,2;3,4}`   |       1  |    4  |    1  |    3  | `1`       |

Headline observations:

- **Identity and diagonal matrices**: bit-exact across all cells
  (n=2..4). Algorithm-choice impact is zero on these inputs.
- **Witness BUG-FUNC-025**: 1 ULP drift across 3 of 4 cells —
  exactly as recorded.
- **Random 4x4 random worst case**: a single 4x4 random matrix
  exposed a `2050` ULP outlier cell. Otherwise random-matrix drift
  stays in the `1..7` ULP range across most cells.
- **Hilbert matrices**: drift grows with size (`22` ULP for 3x3,
  `352` ULP for 4x4). These are well-known ill-conditioned matrices;
  the drift here reflects condition-number amplification of the
  Gauss-Jordan rounding path in the OxFunc kernel vs Excel's path,
  not a kernel bug.
- **Diagonally-dominant matrices**: predictable ~1-2 ULP drift.

## 6. Doctrine

This is a re-measurement only. No kernel repair lands in W097. The
revised data sustains the BUG-FUNC-025 diagnosis ("Gauss-Jordan
rounding path that differs from Excel by low bits on the seed 2x2
inverse") and adds a typical-magnitude band:

- Well-conditioned inputs: typically `0..7` ULP per result cell.
- Ill-conditioned inputs (Hilbert): `10..400` ULP per cell, growing
  with `n`.
- Outliers: rare, up to `~2000` ULP on one random 4x4 row.

When a future repair lands a different matrix-inversion substrate
(LU-solve, CroutLU, Cholesky) the R-F case set is the right
regression-validation gate — anything materially worse than the
above per-kind floor is a regression.
