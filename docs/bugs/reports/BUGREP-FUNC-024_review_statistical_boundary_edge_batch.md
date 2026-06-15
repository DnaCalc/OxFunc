# BUGREP-FUNC-024: Review finding on statistical and boundary edge batch

## Intake
- **Report id**: `BUGREP-FUNC-024`
- **Filed**: 2026-06-15
- **Source channel**: local review follow-up
- **Reporter/source**: June 2026 OxFunc cleanup review
- **Reported against ref**: `w100-w102-cleanup-pass working tree`
- **Reported against kind**: unknown
- **Reported against note**: code-review batch with focused working-tree tests
- **Canonical bug id**: `BUG-FUNC-039`
- **Status**: triaged

## Observed Symptom
Several small statistical and rounding boundary lanes diverged from Excel:
divergent densities published `+inf`, `1e10` df was rejected, `GAMMA.INV`
`p=0`/`p=1` boundary lanes were admitted, `CONFIDENCE` did not truncate size, and
`MROUND(0, negative)` rejected zero.

## Reproduction
1. See `BUG-FUNC-039`.
2. Expected: each edge follows the Excel-specific publication or truncation rule.
3. Actual before the working-tree patch: mathematical-default or broad-boundary
   behavior. `GAMMA.INV` tail/cap behavior is not part of this W102A report
   claim and remains split to probe/exactness follow-up.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: the defects are in OxFunc kernels.

## Links
1. `docs/bugs/streams/BUG-FUNC-039_statistical_and_boundary_edge_batch.md`
2. `crates/oxfunc_core/src/functions/chi_f_t_family.rs`
3. `crates/oxfunc_core/src/functions/beta_gamma_stats_family.rs`
4. `crates/oxfunc_core/src/functions/normal_log_family.rs`
5. `crates/oxfunc_core/src/functions/mround.rs`

## Triage Notes
This batch does not close broader statistical numeric exactness drift.
`GAMMA.INV` tail/cap behavior is explicitly outside the W102A checkpoint claim.
