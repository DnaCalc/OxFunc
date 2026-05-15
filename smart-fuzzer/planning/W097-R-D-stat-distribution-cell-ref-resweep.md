# W097 R-D — BUG-FUNC-021 statistical-distribution cell-ref re-sweep

Status: `tranche_complete`

Owning workset: `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md`
Owning bead: `oxf-ic1h.4`
Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`

## 1. What this record is

R-D builds a dedicated stat distribution explorer along the same shape
as `broad_scalar_explorer.rs` and runs it through the W097 R-B
`CellRefBatch.psm1` cell-ref Excel comparator to produce per-distribution
ULP histograms for the BUG-FUNC-021 surface. This replaces the
"approximate" row counts recorded against the W090 array-tranche
runners with bit-exact-input ULP measurements.

## 2. New tooling

- `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/stat_distribution_explorer.rs`
  — Rust binary that picks per-distribution argument bands, evaluates
  locally through `eval_surface_value_call`, and emits Excel candidate
  records with a tagged `args_typed` field so logical (e.g. GAMMADIST
  cumulative) and matrix (KURT/SKEW/SKEW.P/PERCENTRANK/Z.TEST) args
  travel alongside scalar args.
- `smart-fuzzer/tools/Run-StatDistributionExploration.ps1` — PowerShell
  driver that runs the explorer and routes candidates through
  `Invoke-ExcelCellRefBatch` for the bit-exact Excel comparison.
- `smart-fuzzer/tools/CellRefBatch.psm1` — extended to write boolean
  cell values directly via `Range.Value2 = $true/$false` so logical
  args round-trip through cells.

Distributions covered (BUG-FUNC-021 surface): `BETADIST`, `BETAINV`,
`CHIDIST`, `CHIINV`, `FDIST`, `FINV`, `GAMMADIST`, `GAMMAINV`,
`HYPGEOMDIST`, `NEGBINOMDIST`, `NORMSDIST`, `NORMSINV`, `TDIST`,
`TINV`, `CONFIDENCE.T`, `KURT`, `SKEW`, `SKEW.P`, `PERCENTRANK`,
`Z.TEST` (`20` total).

## 3. Run

`smart-fuzzer/runs/W097-R-D-stat-distribution-cellref/`

- `1,000,000` local cases, seed `17`, `800` Excel-sampled candidates
- Rollup: matches `110`, known stat drift `675`, unexpected `15`,
  blocked `0`
- Excel environment: `16.0` build `19929`, workbook compatibility `2`

## 4. Per-distribution ULP histograms

| Function       | total | match | drifts | ULP min | ULP median | ULP max         |
| -------------- | ----: | ----: | -----: | ------: | ---------: | --------------: |
| `BETADIST`     |  `28` |  `13` |   `28` |     `1` |       `16` | `4.5E3`         |
| `BETAINV`      |  `71` |   `4` |   `71` |     `1` |       `29` | `6.78E17` (sat) |
| `CHIDIST`      |  `41` |   `0` |   `41` |     `1` |      `739` | `1.38E19` (sat) |
| `CHIINV`       |  `14` |   `0` |   `14` |     `2` |       `16` | `4.10E6`        |
| `CONFIDENCE.T` |  `87` |   `1` |   `87` |     `1` |      `538` | `3.12E4`        |
| `FDIST`        |  `43` |   `8` |   `43` |     `1` |       `32` | `3.33E3`        |
| `FINV`         |  `80` |   `1` |   `80` |     `1` |       `95` | `3.48E7`        |
| `GAMMADIST`    |  `89` |  `36` |   `89` |     `1` |        `7` | `1.85E4`        |
| `GAMMAINV`     |  `82` |   `2` |   `82` |     `1` |       `31` | `1.45E18` (sat) |
| `HYPGEOMDIST`  |  `46` |   `0` |   `46` |     `2` |      `105` | `6.08E3`        |
| `KURT`         |   `9` |   `3` |    `9` |     `0` |        `0` | `32`            |
| `NEGBINOMDIST` | `138` |   `7` |  `138` |     `1` |       `20` | `237`           |
| `NORMSDIST`    |   `3` |   `1` |    `3` |     `3` |       `47` | `47`            |
| `NORMSINV`     |   `2` |   `1` |    `2` |    `21` |       `21` | `21`            |
| `PERCENTRANK`  |   `9` |   `9` |    `0` |       — |          — | —               |
| `SKEW`         |   `9` |   `5` |    `9` |     `0` |        `0` | `1`             |
| `SKEW.P`       |   `9` |   `9` |    `0` |       — |          — | —               |
| `TDIST`        |  `17` |   `6` |   `17` |     `2` |       `38` | `1.23E3`        |
| `TINV`         |  `14` |   `0` |   `14` |     `1` |       `13` | `189`           |
| `Z.TEST`       |   `9` |   `4` |    `9` |     `1` |      `167` | `9.07E2`        |

`(sat)` = `Get-UlpDistance` saturated by sign-magnitude-to-Int64
overflow when both operands cross zero or one is sub-normal-adjacent;
the actual drift is large (`>10^6` ULP) but not necessarily literally
the saturated value.

### 4.1 Per-subfamily observations

- **`SKEW.P` and `PERCENTRANK`**: bit-exact matches in every sampled
  row; these can be **dropped from BUG-FUNC-021 scope** as they have
  no measurable kernel drift in the new sample.
- **`SKEW` and `KURT`**: near-zero ULP throughout; small finite-tail
  outliers (`32` ULP for KURT). Effectively closed.
- **`GAMMADIST`**: `40%` exact match rate (`36/89`), median drift
  `7` ULP. Tightest of the inverse/distribution families.
- **`NEGBINOMDIST`**: largest sample count (`138`) with controlled
  tail (max `237` ULP). Repair priority: medium.
- **`CHIDIST`** and **`CONFIDENCE.T`**: largest median drift among
  non-saturating rows (`739` and `538` ULP respectively). Repair
  priority: high — these surfaces drift the most under the new
  measurement.
- **`BETAINV`, `GAMMAINV`, `CHIDIST`** (saturated max): contain rows
  where one side approaches `0` and the other is sub-normal-adjacent,
  giving very large ULP measurements. The repair direction is to
  switch to a `log`-domain or `1-p` formulation in the inverse path
  to recover absolute precision near `0` or `1`.

## 5. Unexpected mismatches (kind drift, NEW)

`15` rows of HYPGEOMDIST returning `error:Num` locally where Excel
returns a finite probability. All `15` rows have inputs that satisfy
Excel's documented HYPGEOMDIST domain
`0 ≤ sample_s ≤ MIN(number_sample, population_s)`,
`0 ≤ number_sample ≤ number_pop`,
`0 ≤ population_s ≤ number_pop`. Witnesses (a few of the 15):

- `=HYPGEOMDIST(492, 792, 644, 821)` → local `#NUM!`, excel finite
- `=HYPGEOMDIST(72, 252, 349, 393)` → local `#NUM!`, excel finite
- `=HYPGEOMDIST(8, 18, 57, 57)` → local `#NUM!`, excel finite

This is a **new HYPGEOMDIST domain-guard sub-class** within
BUG-FUNC-021 that the literal-text harness was not surfacing — the
repair direction is to relax OxFunc's HYPGEOMDIST overflow / domain
guard to match Excel's behaviour for moderate-large sample/population
combinations. Tracked as a follow-up sub-bullet of BUG-FUNC-021's
existing fix plan rather than a separate stream.

## 6. Doctrine

This is a re-measurement only. No kernel repair lands in W097. The
revised ULP histograms above replace the qualitative "5..48 unexpected
mismatches per W090 tranche" magnitudes recorded against
BUG-FUNC-021 with per-distribution drift bands that the existing
BUG-FUNC-021 fix plan (Section 4) can use to sequence its
substrate-by-substrate repair work.
