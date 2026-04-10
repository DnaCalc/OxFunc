# BUG-FUNC-009: RATE default-guess solver no-convergence on mortgage-style lane

## Summary
- **Bug id**: `BUG-FUNC-009`
- **Opened**: 2026-04-10
- **Status**: `validated_local`
- **Owner workset**: `W081`

## Source Refs
- **Reported against ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Reproduced on ref**: `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: intake pinned the current committed local ref on 2026-04-10.
  Live Excel COM replay on 2026-04-10 showed that
  `RATE(360,-1073.64,200000)` has underlying value `0.004166644536345589`
  despite displaying as `0%` under General formatting, while the pre-fix local
  OxFunc omitted-guess path returned `NoConvergence` and the public surface
  therefore mapped the lane to `#NUM!`. The local `W081` correction is now on
  the working tree and validated, but not yet landed on a committed ref.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: the admitted `RATE` packet pinned only a seed
  inversion row and left the default-guess root-finding path undercharacterized
  for long-horizon, small-positive-root mortgage-style lanes. The current
  implementation can solve this row with nearby explicit guesses but fails the
  default omitted-guess path.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `yes`
- **Spec vague or missing?**: `no`
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown`
- **Explanation**: Excel's current-baseline behavior is clear once the
  underlying numeric value is inspected. The local `RATE` implementation did
  not regress from a previously pinned correct lane; instead, the earlier
  `W024`/`W029` evidence set admitted a representative seed inversion sample but
  never forced the omitted-guess solver through this mortgage-style case. The
  default `0.1` start therefore remained overtrusted.

## Reproduction
1. Live Excel COM replay on 2026-04-10 observed:
   - `=RATE(360,-1073.64,200000)`
   - displayed text under General: `0%`
   - underlying `Value2`: `0.004166644536345589`
2. Direct local OxFunc observation on the same committed ref:
   - `rate(360,-1073.64,200000,0,EndOfPeriod,None) -> NoConvergence`
   - surface mapping therefore publishes `#NUM!`
3. Bounded local guess scan:
   - `guess=None` -> `NoConvergence`
   - `guess=0.1` -> `NoConvergence`
   - `guess=0.01` -> `0.00416664453634561`
   - `guess=0.005` -> `0.00416664453634558`
   - `guess=0.004` -> `0.00416664453634550`
   - `guess=0.001` -> `0.00416664453634507`

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_SCENARIO_MANIFEST_SEED.csv`
  3. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`
- **Spec state at intake**: `correct but incomplete`
- **Notes**: the earlier financial time-value packet already admitted `RATE`
  for the current baseline, but only through a representative inversion seed.
  This bug reopens `RATE` until the omitted-guess robustness lane is pinned and
  repaired honestly.

## Investigation Log
1. 2026-04-10: user flagged `=RATE(360,-1073.64,200000)` for inspection.
2. 2026-04-10: live Excel COM replay showed the row is not zero; the underlying
   periodic rate is `0.004166644536345589`.
3. 2026-04-10: direct local OxFunc kernel replay returned `NoConvergence`,
   proving the local result is a real parity bug rather than a formatting
   misunderstanding.
4. 2026-04-10: bounded local guess scan showed explicit guesses near the root do
   converge, which narrows the likely failure mode to omitted/default-guess
   robustness rather than a total lack of a root.
5. 2026-04-10: corrected the local `RATE` kernel by adding a bounded
   bracket-and-bisection fallback around the existing secant path so the
   omitted/default-guess mortgage lane can recover the Excel root instead of
   failing with `NoConvergence`.
6. 2026-04-10: pinned the exact mortgage-style row in the existing W24
   financial-time-value witness manifest and reran the native worksheet probe.

## Similar-Risk Scan
### Adjacent families to check
1. other `RATE` rows with omitted guess on long-horizon amortization lanes
2. `RATE` rows whose true root is small positive and far from the default `0.1`
   seed
3. the existing admitted `RATE` seed inversion row from `W024`
4. adjacent finance root-finding surfaces only for contrast:
   - `IRR`
   - `XIRR`
   - `XNPV`

### Check method
1. live Excel COM replay for the exact mortgage-style row
2. direct local OxFunc kernel replay for omitted and explicit guesses
3. comparison with the existing `W024` / `W29` admitted seed evidence

### Results
1. the existing seed inversion row remains aligned in prior evidence:
   - `RATE(48,PMT(0.01,48,8000),8000) -> 0.00999999999999997`
2. the newly observed mortgage-style omitted-guess lane diverged locally before
   the `W081` repair:
   - Excel returns `0.004166644536345589`
   - OxFunc returns `#NUM!` through `NoConvergence`
3. the local `W081` correction now matches the mortgage-style lane through the
   omitted/default path and preserves the earlier admitted seed inversion row.
4. local explicit guesses near the root converged even before the repair, which
   correctly narrowed the problem to the default-guess / fallback strategy
   rather than a broader publication-format issue.
5. `IRR`, `XIRR`, and `XNPV` are not widened by this intake because they use
   different kernels; no new canonical stream is opened for them from this
   evidence alone.

### Follow-on Openings
1. `W081`

## Fix Plan
1. extend the adjacent omitted-guess scan beyond the first mortgage-style row
2. keep the admitted seed inversion row and the new mortgage-style witness
   aligned through any later solver adjustments
3. land the local correction on a committed ref and reconcile `W051` /
   finance truth surfaces honestly

## Validation
1. live Excel COM replay on 2026-04-10 for `=RATE(360,-1073.64,200000)`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib financial_time_value_family -- --nocapture`
3. direct local kernel replay of `rate(360,-1073.64,200000,0,EndOfPeriod,guess)`
   for omitted/default and nearby explicit guesses
4. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch11-financial-time-value-baseline.ps1`

## Linked Reports
1. `BUGREP-FUNC-013`

## Evidence
1. `crates/oxfunc_core/src/functions/financial_time_value_family.rs`
2. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
3. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`
5. `docs/worksets/W081_RATE_DEFAULT_GUESS_CONVERGENCE_REPAIR.md`

## Closure Checklist
- [x] local fix implemented on working tree
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] linked reports updated
- [ ] handoff filed if required
