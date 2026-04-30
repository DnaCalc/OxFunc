# Known Exactness Deviations

Status: `active_register`
Last updated: `2026-04-30`

## Purpose
This register is the project-wide index of current OxFunc-vs-Excel exactness
residuals.

OxFunc still targets exact typed bit parity with Excel for the declared current
reference baseline. An entry here is not a waiver, not a pass, and not a
tolerance policy. It means:

1. the deviation has credible local-vs-Excel evidence,
2. the detailed evidence lives in a bug stream, bead, or smart-fuzzer run,
3. unrelated DNA Calc development may continue while the residual is tracked,
4. smart-fuzzer runs may classify matching rows as known residuals instead of
   rediscovering them as new unexpected mismatches,
5. the relevant functions remain `scope_partial` for exact current-baseline
   parity until the residual is repaired or explicitly reclassified.

## Operating Rules
1. Keep the comparison policy as `exact_typed_bit_match_no_tolerance`.
2. Do not green tests by patching formula-specific lookup tables or long
   failure lists.
3. Do not create heavy per-case prose for high-volume pass/fail runs. Record
   compact counts here and keep detailed failure packets under run artifacts.
4. Add a row here only after there is a canonical bug stream or bead reference.
5. If a future investigation proves a row is version/channel or workbook
   compatibility behavior, split it into a versioned evidence record instead of
   relaxing equality.
6. When a residual is repaired, move the entry to the resolved-history section
   or remove it only after the owning bug stream records the fix and validation.

## Entry Shape
Each active entry should record:

1. stable residual id,
2. owning bug stream and bead,
3. affected function family,
4. mismatch class,
5. compact evidence counts,
6. representative witnesses,
7. intended next action.

## Active Residuals

### KED-FIN-001: Financial Payment Publication Exactness
- **Status**: `blocked_deferred_repair`
- **Owner**: `BUG-FUNC-015`, bead `oxf-fckb`
- **Functions**: confirmed `PMT`, `PPMT`; adjacent shared-calculation review
  includes `IPMT`
- **Mismatch class**: financial time-value publication exactness and adjacent
  high-rate/long-horizon edge behavior
- **Current handling**: known residual; useful smart-fuzzer reference lane; do
  not repair until a focused convergence/reparation thread is explicitly
  reopened
- **Evidence**:
  1. `docs/bugs/streams/BUG-FUNC-015_pmt_ppmt_annuity_exactness_drift.md`
  2. `smart-fuzzer/tools/Run-PmtPpmtPilot.ps1`
  3. `smart-fuzzer/tools/pmt_ppmt_local_eval/`
  4. `smart-fuzzer/runs/w088-pmt-ppmt-pilot/` when present locally
  5. expanded run `expanded-finance-10m-20260428`
- **Compact counts**:
  1. W088 PMT/PPMT pilot: `28` cases, `7` exact matches, `21` numeric bit
     mismatches, `0` blocked.
  2. Expanded finance sample: `640` Excel-sampled cases, `536` exact matches,
     `102` expected known financial-exactness or formula-literal encoding
     deviations, and `2` adjacent high-rate/long-horizon `PPMT` rows where
     local returned `#NUM!` while Excel returned a tiny numeric value or zero.
- **Representative witnesses**:
  1. `=PMT(0.05/12,360,200000)`
  2. `=PPMT(0.05/12,1,360,200000)`
- **Next action**: characterize Excel's payment publication path over a small,
  theory-driven matrix before changing the shared annuity implementation.

### KED-STAT-001: Statistical Distribution Exactness Residuals
- **Status**: `open`
- **Owner**: `BUG-FUNC-021`, bead `oxf-simj`
- **Functions**: `BETAINV`, `CHIDIST`, `CHIINV`, `FDIST`, `FINV`,
  `GAMMADIST`, `GAMMAINV`, `HYPGEOMDIST`, `NEGBINOMDIST`, `NORMSDIST`,
  `NORMSINV`, `TDIST`, `TINV`, `CONFIDENCE.T`, `Z.TEST`
- **Mismatch class**: statistical scalar numeric algorithm/publication
  exactness after array-admission issues were repaired
- **Current handling**: known residual family; keep exact comparison; avoid
  case-specific patching; repair by substrate family
- **Evidence**:
  1. `docs/bugs/streams/BUG-FUNC-021_w090_statistical_numeric_exactness_drift.md`
  2. `smart-fuzzer/runs/w090-successor-all-20260430-smart-wide-001/`
  3. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
  4. `smart-fuzzer/runs/oxf-simj-w090-repair-20260430-003/`
  5. `smart-fuzzer/runs/oxf-simj-w089-seed-repair-20260430-003/`
- **Compact counts**:
  1. Pre-repair W090 successor-all replay: `139` cases, `98` exact matches,
     `41` unexpected mismatches.
  2. Post-partial-repair W090 successor-all replay: `139` cases, `102` exact
     matches, `37` unexpected mismatches, no harness blockers.
  3. Pre-repair W089 seed replay: `339` cases, `288` exact matches, `48`
     unexpected mismatches, `3` Excel harness blockers.
  4. Post-partial-repair W089 seed replay: `339` cases, `294` exact matches,
     `42` unexpected mismatches, `3` Excel harness blockers.
- **Representative witnesses**:
  1. `=BETA.INV(0.6,2,3)` / `=BETAINV(0.6,2,3)`
  2. `=CHIDIST(18.307,10)` / `=CHIINV(0.0500006,10)`
  3. `=F.DIST.RT(15.2069,6,4)` / `=FINV(0.01,6,4)`
  4. `=GAMMA.INV(0.5,3,2)` / `=GAMMAINV(0.5,3,2)`
  5. `=NORM.S.DIST(1.25,TRUE)` / `=NORMSINV(0.9)`
  6. `=T.DIST.RT(1.5,10)` / `=TINV(0.1,10)`
  7. `=Z.TEST({3;6;7;8;6},4,1.5)`
- **Partial repairs already landed**: `BETA.DIST` / `BETADIST` integer-shape
  CDF, `PERCENTRANK` significance truncation, `BINOM.DIST.RANGE` finite sum,
  `NEGBINOM.DIST` cumulative witness, and `SKEW.P` standardized-moment
  ordering.
- **Next action**: split follow-up repair work by numerical substrate:
  beta/gamma inverses, chi/f/t tails and inverses, discrete distributions,
  normal standard aliases, and `CONFIDENCE.T` / `Z.TEST`.

### KED-BESSEL-001: BESSELY Current-Baseline Exactness Residual
- **Status**: `open`
- **Owner**: `BUG-FUNC-024`, bead `oxf-xp6p`
- **Functions**: `BESSELY`
- **Mismatch class**: Bessel `Y` scalar numeric algorithm/publication
  exactness
- **Current handling**: known residual split from W089; repair by Bessel
  substrate only, not by formula-specific lookup
- **Evidence**:
  1. `docs/bugs/streams/BUG-FUNC-024_bessely_current_baseline_exactness_drift.md`
  2. `docs/bugs/streams/BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md`
  3. `smart-fuzzer/runs/oxf-i45e-w089-repair-20260430-001/`
  4. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
- **Representative witnesses**:
  1. `=BESSELY(2.5,1)`
- **Next action**: build a compact Excel probe grid for Bessel `Y` over
  order/x branch and recurrence lanes, then replace or adjust the substrate
  algorithm against that grid.

### KED-MATRIX-001: MINVERSE Matrix Numeric Exactness Residual
- **Status**: `open`
- **Owner**: `BUG-FUNC-025`, bead `oxf-dzfk`
- **Functions**: `MINVERSE`
- **Mismatch class**: matrix inversion low-bit numeric exactness
- **Current handling**: known residual split from W089; scalar `1x1`
  publication is repaired, but multi-cell inversion rounding remains open
- **Evidence**:
  1. `docs/bugs/streams/BUG-FUNC-025_minverse_matrix_numeric_exactness_drift.md`
  2. `docs/bugs/streams/BUG-FUNC-023_w089_non_statistical_exactness_and_matrix_shape_drift.md`
  3. `smart-fuzzer/runs/oxf-i45e-w089-repair-20260430-001/`
  4. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
- **Representative witnesses**:
  1. `=MINVERSE({1,2;3,4})`
- **Next action**: build a compact Excel probe grid for small matrices and
  compare candidate inversion algorithms before changing the kernel.

## Smart-Fuzzer Classification Guidance
Smart-fuzzer comparison output should keep three separate classes:

1. `exact_typed_bit_match`
2. `known_exactness_residual`
3. `unexpected_mismatch`

Known residuals should still be counted and reported. They should not fail a
broad discovery run as new information, but they must not be counted as passes.
If a known residual appears with a materially new formula shape, argument
region, error-vs-number difference, or affected function, promote that as new
evidence under the owning bug stream or open a new bug stream.

## Resolved History
1. `KED-MISC-001` records the mixed W089 non-statistical residual triage:
   `VDB`, `MINVERSE(5)`, and `MMULT(5,2)` were repaired under
   `BUG-FUNC-023`; `BESSELY(2.5,1)` and `MINVERSE({1,2;3,4})` remain active
   under dedicated entries above.
