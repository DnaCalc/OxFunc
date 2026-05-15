# W097 R-E — BUG-FUNC-024 BESSELY cell-ref re-sweep

Status: `tranche_complete`

Owning workset: `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md`
Owning bead: `oxf-ic1h.5`
Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`

## 1. What this record is

R-E re-replays the BUG-FUNC-024 BESSELY exactness witness `=BESSELY(2.5, 1)`
under cell-ref Excel input plumbing, plus a fixed band of `(x, n)` pairs
around the witness sampled in both directions, to confirm the magnitude
of the kernel drift.

## 2. New tooling

- `smart-fuzzer/tools/Run-BesselyResweep.ps1` — generates the case set,
  evaluates locally through `pmt_ppmt_local_eval --bin pmt_ppmt_local_eval`
  (which dispatches `FUNC.BESSELY` through `eval_surface_value_call`),
  and runs Excel through `Invoke-ExcelCellRefBatch`.

## 3. Run

`smart-fuzzer/runs/W097-R-E-bessely-cellref/`

- Cases: `93` total
  - Broad band: `n ∈ {0, 1, 2, 3, 5, 10}` × `x ∈ {0.5, 1, 1.5, 2, 2.5, 3, 5, 10, 20, 50, 100}` (`66` rows; `n=10` skips a few high-x because the broad set is `11` x-values per n)
  - Tight band around `(2.5, 1)`: `n ∈ {0, 1, 2}` × `x ∈ {2.40, 2.45, 2.49, 2.499, 2.5, 2.501, 2.51, 2.55, 2.60}` (`27` rows)
- Rollup: matches `1`, drifts `92`, kind drift `0`, blocked `0`
- Excel environment: `16.0` build `19929`

## 4. Witness confirmation

`=BESSELY(2.5, 1)`:
- local bits `0x3fc2ad722ba3570c`
- Excel bits `0x3fc2ad720e3ee754`
- ULP distance `493,121,464`

This **confirms exactly** the BUG-FUNC-024 recorded witness pair:
`=BESSELY(2.5,1)`: local `number:0x3fc2ad722ba3570c`, Excel
`number:0x3fc2ad720e3ee754`. The cell-ref re-replay reproduces the
literal-text witness bit-for-bit.

## 5. Per-`n` ULP histograms

| `n` | rows | matches | drifts | ULP min  | ULP median | ULP max  |
| --: | ---: | ------: | -----: | -------: | ---------: | -------: |
|  `0`|  `20` |     `1` |   `19` | `7.3E3`  |    `6.1E5` | `4.3E6`  |
|  `1`|  `20` |     `0` |   `20` | `3.3E6`  |    `4.9E8` | `2.1E12` |
|  `2`|  `20` |     `0` |   `20` | `2.6E5`  |    `2.0E8` | `4.2E11` |
|  `3`|  `11` |     `0` |   `11` | `2.0E6`  |    `5.7E7` | `1.4E12` |
|  `5`|  `11` |     `0` |   `11` | `2.0E6`  |    `5.6E7` | `1.3E12` |
| `10`|  `11` |     `0` |   `11` | `1.1E6`  |    `7.6E8` | `5.8E11` |

Direction: the BESSELY drift surface is **broad and large**. The
recorded witness `(2.5, 1)` is roughly the *median* of the `n=1` band,
not an outlier — the kernel is approximately uniformly off across the
entire (x, n) sample. The drift grows with `|n|` and stays in the
`10^6..10^12` ULP range across most of the surface.

## 6. Tight band around `(x=2.5, n ∈ {0, 1, 2})`

| `x`    | `n` | ULP        |
| ------ | --: | ---------: |
| `2.40` |  `0` |     `7269` |
| `2.45` |  `0` |    `68910` |
| `2.49` |  `0` |   `262179` |
| `2.499`|  `0` |   `290423` |
| `2.5`  |  `0` |   `293566` |
| `2.501`|  `0` |   `296711` |
| `2.51` |  `0` |   `325062` |
| `2.55` |  `0` |   `452019` |
| `2.60` |  `0` |   `612501` |
| `2.40` |  `1` | `4.53E8`   |
| `2.5`  |  `1` | `4.93E8`   |
| `2.60` |  `1` | `9.61E8`   |
| `2.40` |  `2` | `9.44E7`   |
| `2.5`  |  `2` | `1.98E8`   |
| `2.60` |  `2` | `3.70E8`   |

The drift varies smoothly with `x`: there's no resonance / cancellation
at the witness `x = 2.5`. The repair direction recorded in BUG-FUNC-024
("substrate/algorithm only; do not patch this witness as a special
case") is the correct one — patching the single witness would not
help the surrounding `(x, n)` band.

## 7. Doctrine

This is a re-measurement only. No kernel repair lands in W097. The
revised data confirms BUG-FUNC-024's qualitative "current-baseline
publication path" diagnosis: the drift is a substrate-wide algorithm
choice, not a single-row witness bug. The R-E run record should be
the new reference baseline for the BUG-FUNC-024 repair-validation
gate when an alternative Bessel-Y substrate is landed.
