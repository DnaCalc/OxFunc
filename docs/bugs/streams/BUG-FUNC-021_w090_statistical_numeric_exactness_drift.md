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

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [ ] root cause recorded
- [ ] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
