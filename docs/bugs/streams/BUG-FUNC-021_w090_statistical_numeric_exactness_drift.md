# BUG-FUNC-021: W090 statistical numeric exactness drift

## Summary
- **Bug id**: `BUG-FUNC-021`
- **Opened**: `2026-04-30`
- **Status**: `open`
- **Owner workset**: `W090`
- **Bead**: `oxf-simj`

## Source Refs
- **Reported against ref**: W090 repair working tree after `BUG-FUNC-018` /
  `BUG-FUNC-019` / `BUG-FUNC-020` repairs
- **Reproduced on ref**: W090 repair working tree
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `unfixed`

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: after scalar parameter array-admission succeeds, a
  subset of compatibility-statistical and modern statistical functions still
  differ from live Excel by numeric bits under the no-tolerance comparison
  policy. This is scalar kernel exactness drift, not an array-lift shape or
  harness failure.

## Reproduction
Run the final W090 repair tranches:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w090-repair-final-compatibility-001 `
  -CaseSetPath smart-fuzzer\cache\array-support-successor-executable-tranches-v0.json `
  -CaseSetTrancheId w090-successor-compatibility

powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w090-repair-final-statistical-functions-001 `
  -CaseSetPath smart-fuzzer\cache\array-support-successor-executable-tranches-v0.json `
  -CaseSetTrancheId w090-successor-statistical-functions
```

Final residual counts:

1. `w090-repair-final-compatibility-001`: `36` unexpected mismatches.
2. `w090-repair-final-statistical-functions-001`: `5` unexpected mismatches.
3. All residual rows have `local_execution_status=ok` and
   `excel_execution_status=ok`.

Affected representative functions:

`BETADIST`, `BETAINV`, `CHIDIST`, `CHIINV`, `FDIST`, `FINV`, `GAMMADIST`,
`GAMMAINV`, `HYPGEOMDIST`, `NEGBINOMDIST`, `NORMSDIST`, `NORMSINV`, `TDIST`,
`TINV`, `PERCENTRANK`, `CONFIDENCE.T`, and `Z.TEST`.

The W089 comprehensive seed replay adds broader manifest-seed coverage for the
same no-tolerance statistical exactness lane:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w089-comprehensive-seed-20260430-004 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json
```

Run `w089-comprehensive-seed-20260430-004` executed `339` cases and left `48`
unexpected mismatches. The statistical subset overlaps this stream, including
beta/gamma/chi/f/t distribution and inverse routines, normal standard
compatibility aliases, `KURT`, `SKEW`, `SKEW.P`, `PERCENTRANK`,
`CONFIDENCE.T`, and `Z.TEST`.

## 2026-04-30 Partial Repair Pass
The `oxf-simj` repair pass made a bounded scalar-kernel exactness improvement
without changing the no-tolerance comparison policy.

Changes landed in this pass:

1. `BETA.DIST` / `BETADIST`: cumulative beta distribution now uses a finite
   binomial-form path for positive integer shape parameters before falling back
   to the generic regularized-beta implementation.
2. `PERCENTRANK`: significance handling now truncates to Excel's observed
   significant-digit result instead of rounding.
3. `BINOM.DIST.RANGE`: small finite ranges now sum direct binomial masses,
   while `BINOM.DIST` / `BINOMDIST` retain their prior log/exp path because the
   W090 replay showed those rows were already exact there.
4. `NEGBINOM.DIST`: the small finite cumulative witness now matches Excel bits;
   `NEGBINOMDIST` scalar mass remains one ULP high in the W089 seed replay.
5. `SKEW.P`: population skew now sums standardized third powers, matching the
   W089 seed witness. `SKEW` and `KURT` remain open ULP residuals.
6. `F.DIST.RT` / `FDIST` and `T.DIST.RT` / `TDIST`: right-tail formulas now call
   the direct beta tail rather than subtracting from one. This reduced
   cancellation risk but did not eliminate the retained exact-bit residual rows.
7. Inverse distribution bisection now stops at a tighter ULP-scale interval.
   This improved the earlier beta-inverse witness substantially but retained
   exact-bit residuals.

Post-repair replay results:

1. `smart-fuzzer/runs/oxf-simj-w090-repair-20260430-003/`: `139` cases,
   `102` exact typed bit matches, `37` unexpected mismatches, no harness
   blockers.
2. `smart-fuzzer/runs/oxf-simj-w089-seed-repair-20260430-003/`: `339` cases,
   `294` exact typed bit matches, `42` unexpected mismatches, `3` Excel harness
   blockers.

The W090 residual set remains OxFunc-owned scalar numeric drift for:
`BETAINV`, `CHIDIST`, `CHIINV`, `FDIST`, `FINV`, `GAMMADIST`, `GAMMAINV`,
`HYPGEOMDIST`, `NEGBINOMDIST`, `NORMSDIST`, `NORMSINV`, `TDIST`, `TINV`,
`CONFIDENCE.T`, and `Z.TEST`.

## Repair Direction
Do not hide this under array-support. Minimize each function to its scalar
formula, compare the scalar result against Excel Value2 with exact numeric
bits, and repair by statistical substrate family:

1. beta/gamma distribution and inverse routines,
2. chi/f/t compatibility aliases,
3. discrete compatibility aliases,
4. normal standard distribution aliases,
5. moment/test helpers (`CONFIDENCE.T`, `Z.TEST`, `PERCENTRANK`).

Keep the no-tolerance comparison policy. If a future scoped investigation
proves a row is an Excel-version/workbook-compatibility axis difference, split
that into a versioned evidence record rather than relaxing equality.

## 2026-05-10 W097 R-D Cell-Ref Re-Sweep

W097 R-D rebuilt the BUG-FUNC-021 evidence under cell-ref Excel input
plumbing using a new `stat_distribution_explorer` Rust binary plus the
`Run-StatDistributionExploration.ps1` driver. See tranche record
`smart-fuzzer/planning/W097-R-D-stat-distribution-cell-ref-resweep.md`.

Per-distribution ULP histogram (from `1,000,000` local cases, `800`
Excel-sampled candidates, seed `17`):

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

Highlights for repair sequencing:

- **`SKEW.P` and `PERCENTRANK`**: bit-exact in every sampled row.
  Drop from BUG-FUNC-021 scope when next visiting the fix plan.
- **`SKEW` and `KURT`**: near-zero ULP throughout. Effectively
  closed.
- **`GAMMADIST`**: highest match rate (`40%`); median drift `7` ULP.
- **`CHIDIST`** and **`CONFIDENCE.T`**: largest non-saturating
  median drift (`739` and `538` ULP). Repair priority: high.
- **`BETAINV` / `GAMMAINV` / `CHIDIST`**: saturating max ULP
  signals tail-precision loss; repair direction is `log`-domain or
  `1-p` formulation in the inverse path.
- **NEW HYPGEOMDIST domain sub-class**: `15` rows return `#NUM!`
  locally where Excel returns a finite probability, all with inputs
  inside Excel's documented domain. This is a kind-drift sub-class
  not previously surfaced by the literal-text harness; repair is to
  relax OxFunc's HYPGEOMDIST overflow / domain guard to match
  Excel's behaviour for moderate-large sample/population
  combinations.

## Evidence
1. `smart-fuzzer/runs/w090-repair-final-compatibility-001/`
2. `smart-fuzzer/runs/w090-repair-final-statistical-functions-001/`
3. `smart-fuzzer/runs/w090-successor-all-20260430-smart-wide-001/`
4. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
5. `smart-fuzzer/runs/oxf-simj-w090-repair-20260430-003/`
6. `smart-fuzzer/runs/oxf-simj-w089-seed-repair-20260430-003/`
7. Bead: `oxf-simj`
8. W092 freshness replays with known-residual classification:
   - `smart-fuzzer/runs/w092-scenario-statistical-cycle-001/`: `34` exact
     typed matches and `24` `known_residual` rows.
   - `smart-fuzzer/runs/w092-scenario-compatibility-cycle-001/`: `20` exact
     typed matches and `13` `known_residual` rows.
   - `smart-fuzzer/runs/w092-w090-successor-all-cycle-001/`: `102` exact
     typed matches and `37` `known_residual` rows over the W090 successor
     aggregate.
9. W097 R-D cell-ref re-replay:
   - `smart-fuzzer/runs/W097-R-D-stat-distribution-cellref/` (`1M` cases,
     seed `17`, `800` Excel-sampled, per-distribution ULP histogram).
   - Tranche record:
     `smart-fuzzer/planning/W097-R-D-stat-distribution-cell-ref-resweep.md`.
   - Local explorer source:
     `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/stat_distribution_explorer.rs`.
   - PowerShell driver:
     `smart-fuzzer/tools/Run-StatDistributionExploration.ps1`.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
