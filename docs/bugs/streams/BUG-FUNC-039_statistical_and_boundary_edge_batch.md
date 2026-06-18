# BUG-FUNC-039: Statistical and boundary edge batch from W102A review

## Summary
- **Bug id**: `BUG-FUNC-039`
- **Opened**: `2026-06-11`
- **Status**: `closed` (sub-lanes landed in `7a0003f`; GAMMA.INV p=0 regression fixed in `ba861ac`; all sub-lanes live-Excel verified 2026-06-19)
- **Owner workset**: `W102A`

## Live Excel Verification (2026-06-18)
OxFunc local value surface vs live Excel 16.0 build 20026:
- `MROUND(0,-2)` = `0` — MATCH.
- `GAMMA.INV(1,3,2)` = `#NUM!` — MATCH (p=1 rejected).
- `CONFIDENCE(0.05,1,2.5)` = `1.3859038243` — MATCH (size truncation).
- `CHISQ.DIST(0,1,FALSE)` = `#NUM!` — MATCH (divergent density).
- `GAMMA.INV(0,3,2)` = `0` — MATCH after the `ba861ac` corrective. The original
  W102A fix over-rejected p=0 with `#NUM!`; Excel accepts p=0 (inverse-CDF lower
  bound 0) and rejects only p=1. `ba861ac` admits p=0 → 0, keeps p=1 → `#NUM!`.
  Bead `oxf-99zz` closed.
- `CHISQ.DIST(5,1e10,TRUE)` = `0` (df=1e10 accepted) and `CHISQ.DIST(5,1e10+1,TRUE)`
  = `#NUM!` (df>1e10 rejected) — MATCH.
- `F.DIST(0,1,1,FALSE)` = `#NUM!` (divergent density) — MATCH.

All BUG-FUNC-039 sub-lanes now match live Excel 16.0 build 20026. The GAMMA.INV
near-one tail/cap behavior remains a separate W102B lane (`oxf-acdw.3.1`).

## Source Refs
- **Reported against ref**: review pass `2026-06-10` / branch `w100-w102-cleanup-pass`
- **Reproduced on ref**: working-tree tests in affected function modules
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed` (working-tree patch present; checkpoint not landed)
- **Ref notes**: this stream groups several small review-identified edge fixes
  that are each covered by local tests but still need register and checkpoint
  promotion.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `spec_mismatch`
- **Root cause summary**: A set of small boundary/error-publication gaps remained
  across statistical and rounding surfaces:
  1. `CHISQ.DIST`/`F.DIST` density at zero with divergent density returned
     `+inf` where Excel publishes `#NUM!`.
  2. Chi/F degrees-of-freedom boundary rejected `1e10` itself instead of only
     values greater than `1e10`.
  3. `GAMMA.INV` accepted `p=0`/`p=1`.
  4. `CONFIDENCE` used fractional sample size directly instead of truncating and
     rejecting `size < 1`.
  5. `MROUND(0, negative)` returned `#NUM!` even though zero has no sign and
     Excel returns `0`.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `unknown`
- **Spec vague or missing?**: `yes`
- **Code once correct and later regressed?**: `unknown`
- **Likely introduced in ref**: `unknown`
- **Explanation**: The local implementations followed reasonable mathematical
  defaults or broad boundary checks, but Excel has narrower publication and
  truncation rules for these edge lanes.

## Reproduction
1. `CHISQ.DIST(0,1,FALSE)` and `F.DIST(0,1,5,FALSE)` should publish `#NUM!`.
2. Degrees of freedom equal to `1e10` should be admitted; values above should
   publish `#NUM!`.
3. `GAMMA.INV(0,1,1)` and `GAMMA.INV(1,1,1)` should publish `#NUM!`.
4. `CONFIDENCE(0.05,2.5,100.9)` should match size `100`.
5. `MROUND(0,-3)` should publish `0`.

## Spec And Contract Relationship
- **Spec references**:
  1. `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
  2. `crates/oxfunc_core/src/functions/beta_gamma_stats_family.rs`
  3. `crates/oxfunc_core/src/functions/normal_log_family.rs`
  4. `crates/oxfunc_core/src/functions/mround.rs`
- **Spec state at intake**: `vague`
- **Notes**: This is a bounded review batch, not a claim that all statistical
  numeric exactness lanes are settled.

## Investigation Log
1. 2026-06-11: W102A working-tree patch added local code changes and focused
   tests for the five edge groups.
2. 2026-06-15: register reconciliation identified the code references but no
   canonical stream or register rows.

## Similar-Risk Scan
### Adjacent families to check
1. statistical distribution aliases
2. inverse distribution boundary paths
3. rounding functions with zero operands

### Check method
1. Focused Rust tests in the affected modules.
2. Follow-up: link exact live Excel evidence where available.

### Results
1. Working-tree tests cover the enumerated edge lanes.
2. Broader statistical numeric drift, including `GAMMA.INV` tail/cap behavior,
   remains under existing exactness/probe lanes and is not closed by this batch.

### Follow-on Openings
1. `GAMMA.INV` tail/cap behavior remains split from W102A; it needs live
   evidence and/or a numeric exactness repair lane before stronger claims.

## Fix Plan
1. Keep the five working-tree edge repairs isolated from broader numeric drift.
2. Run focused module tests and full `oxfunc_core` lib validation before status
   promotion.
3. Add or link live Excel evidence for any edge lacking durable probe artifacts.

## Validation
1. Working-tree tests:
   - `chisq_pdf_x0_df1_returns_num_not_inf`
   - `f_pdf_x0_d1lt2_returns_num_not_inf`
   - `df_boundary_at_1e10_is_admitted`
   - `gamma_inv_boundary_cases`
   - `confidence_size_truncation_and_sub1_rejection`
   - `mround_zero_number_with_negative_multiple_returns_zero`

## Linked Reports
1. `BUGREP-FUNC-024`

## Evidence
1. `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
2. `crates/oxfunc_core/src/functions/beta_gamma_stats_family.rs`
3. `crates/oxfunc_core/src/functions/normal_log_family.rs`
4. `crates/oxfunc_core/src/functions/mround.rs`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [ ] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] `GAMMA.INV` tail/cap behavior split to W102B or numeric exactness lane
- [ ] handoff filed if required
- [x] linked reports updated
