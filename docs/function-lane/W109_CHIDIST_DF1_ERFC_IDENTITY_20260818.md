# CHIDIST(df=1) / Gamma(1/2, scale=2) published-ERF identity

Date: 2026-08-18
Lane: W109 inverse-problem decomposition
Reference: Excel 16.0 build 20228, x64, Value2

## Status axes

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial` (df=1 / Gamma(0.5,2) CDF route only)
- `target_completeness`: `target_partial`
- `integration_completeness`: `integrated` for the landed dispatch
- `open_lanes`: remaining G3-01 GRATIO/BRATIO body; ERF/ERFC.PRECISE body; inverses; CHISQ.TEST statistic for df≠1

## Method

Related worksheet functions were asked, through cell `Range.Value2` inputs,
which public identity Excel actually computes. No Excel binary was inspected.

## Identities that survived a 154-row nonnegative bank

The bank is powers of two from `2^-60` through `2^4`, a 0.05-step grid
through 4, and the usual chi-square critical values up to 100, plus 0.

| Identity | Exact |
|---|---:|
| `CHIDIST(x,1)` = `ERFC.PRECISE(SQRT(x/2))` | 154/154 |
| `CHIDIST(x,1)` = `CHISQ.DIST.RT(x,1)` | 154/154 |
| `GAMMA.DIST(x,0.5,2,TRUE)` = `ERF.PRECISE(SQRT(x/2))` | 154/154 |
| `CHISQ.DIST(x,1,TRUE)` = `GAMMA.DIST(x,0.5,2,TRUE)` | 154/154 |
| `CHIDIST(x,1)` = `ERFC.PRECISE(SQRT(x)/SQRT(2))` | 123/154 (refuted) |
| `CHIDIST(x,1)` = `ERFC.PRECISE(SQRT(x)*(1/SQRT(2)))` | 117/154 (refuted) |
| `CHIDIST(x,1)` = `1-GAMMA.DIST(x,0.5,2,TRUE)` | 92/154 (cancellation, not the graph) |

The surviving staging is divide-by-two first, then worksheet `SQRT`.
`ERF`/`ERFC` and the `.PRECISE` aliases were bit-identical on a separate
16-point probe of this build.

## What was landed

`chisq_dist_rt_kernel` (CHIDIST / CHISQ.DIST.RT) and the cumulative
`chisq_dist_kernel` / `gamma_dist_kernel(x, 0.5, 2, TRUE)` now call the
published ERF/ERFC kernels at `sqrt(x/2)`.

On the same 154 Excel rows, before the dispatch OxFunc CHIDIST matched
63/154 (max 30 ULP). The ERFC composition matched 88/154 (max 31 ULP).
The dispatch is the identified graph; it also improves the current residual.
The remaining misses are the still-open ERF/ERFC.PRECISE body.

## What this is not

- Not a full G3-01 or G4-04 closure.
- Not an ERF coefficient identification.
- Not a claim about `GAMMA.DIST` at other shapes/scales.
- Not a CHIINV / GAMMA.INV graph.
