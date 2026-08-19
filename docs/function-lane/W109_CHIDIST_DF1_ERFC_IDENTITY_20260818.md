# CHIDIST(df=1) / Gamma(1/2, scale=2) published-ERF identity

Date: 2026-08-18
Lane: W109 inverse-problem decomposition
Reference: Excel 16.0 build 20228, x64, Value2

## Status axes

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial` (df=1 / Gamma(0.5,2) CDF route only)
- `target_completeness`: `target_partial`
- `integration_completeness`: `integrated` for the landed dispatch
- `open_lanes`: remaining G3-01 GRATIO/BRATIO body; ERF/ERFC.PRECISE body; inverses; even-df Poisson-series association (df=4/6 not exact); CHISQ.TEST statistic for df≠1

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

## Follow-up identities (2026-08-18, same build)

A second 68-row bank plus an 11-row CDF complement check:

| Identity | Exact |
|---|---:|
| `CHIDIST(x,2)` = `EXP(-x/2)` | 68/68 |
| `CHIDIST(x,2)` = `EXP(-(x/2))` | 68/68 |
| `CHISQ.DIST(x,2,TRUE)` = `1-EXP(-x/2)` | 9/11 (refuted as the CDF graph) |
| `GAMMA.DIST(x,0.5,1,TRUE)` = `ERF.PRECISE(SQRT(x))` | 68/68 |
| `GAMMA.DIST(x,0.5,4,TRUE)` = `ERF.PRECISE(SQRT(x/4))` | 68/68 |
| `CHIDIST(x,4)` = `EXP(-x/2)*(1+x/2)` | 50/68 (refuted) |
| `CHIDIST(x,6)` = next Poisson term | 46/68 (refuted) |

Landed: df=2 right-tail through `excel_exp(-(x/2))`; `GAMMA.DIST` CDF at shape `0.5` through `ERF.PRECISE(SQRT(x/beta))` for any positive finite scale.

## Follow-up identities (2026-08-19)

| Identity | Exact | Action |
|---|---:|---|
| `CHISQ.DIST(x,2,TRUE)` = `EXPON.DIST(x/2,1,TRUE)` | 85/85 | Landed |
| `CHISQ.DIST(x,2,TRUE)` = `GAMMA.DIST(x,1,2,TRUE)` | 85/85 | Same graph |
| `GAMMA.DIST(x,1,beta,TRUE)` = `EXPON.DIST(x/beta,1,TRUE)` | 85/85 dyadic; 11/11 for beta=3,5 | Landed (divide-first) |
| `EXPON.DIST(x,1/beta,TRUE)` | 1 ULP misses when 1/beta inexact | Refuted as the Gamma staging |
| `GAMMA`/`EXPON` PDF | 33/85 | Refuted |
| `CHIDIST(x,2)` = `POISSON.DIST(0,x/2,TRUE)` | 45/45 | Same as EXP |
| `CHIDIST(x,4)` = `POISSON.DIST(1,x/2,TRUE)` | 45/45 | Poisson CDF now dispatched (see 2026-08-19) |
| `CHIDIST(x,6)` = `POISSON.DIST(2,x/2,TRUE)` | 45/45 | Same |
| `CHIDIST(x,4)/EXP(-x/2)` = `1+x/2` | 70/85, leftover ±1–7 ULP | Series association still open |
| `ERFC.PRECISE(z)` = `CHIDIST(2*z*z,1)` | 27/27 | Implied Q is the published ERFC surface; no separate body |

General even-df rule observed: `CHIDIST(x, 2(k+1))` = `POISSON.DIST(k, x/2, TRUE)`.

## Poisson CDF landing (2026-08-19)

A 70-pair bank (`k ∈ {0,1,2,3,5}`, mixed `μ`) on live Excel 16.0 build 20228:

| Identity | Exact |
|---|---:|
| `POISSON.DIST` = `POISSON` alias | 70/70 |
| `POISSON.DIST(k,μ,TRUE)` = `CHIDIST(2*μ, 2(k+1))` | 70/70 |
| same with `μ*2` and `μ+μ` | 70/70 |
| `CHISQ.DIST.RT(2*μ, 2(k+1))` | 70/70 |
| `1-GAMMA.DIST(μ,k+1,1,TRUE)` | 45/70 (refuted) |
| sum of published `POISSON.DIST(j,μ,FALSE)` for `j=0..k` | 45/70 (refuted; only `k=0` is 14/14) |

Landed `poisson_dist_kernel` CDF as `EXP(-μ)` for `k=0` and GRATIO `Q(k+1, μ)` for `k≥1`, which is CHIDIST internals after the exact `2μ/2` recovery. The PMF path is unchanged.

## What this is not

- Not a full G3-01 or G4-04 closure.
- Not an ERF coefficient identification.
- Not a claim about `GAMMA.DIST` at other shapes/scales.
- Not a CHIINV / GAMMA.INV graph.
