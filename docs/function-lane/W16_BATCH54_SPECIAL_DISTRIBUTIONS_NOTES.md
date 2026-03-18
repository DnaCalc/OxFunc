# W16 Batch 54 - Special Distributions

Status: `packet-evidenced`
Workset: `W16`

## Scope
1. `ERF`
2. `ERF.PRECISE`
3. `ERFC`
4. `ERFC.PRECISE`
5. `GAMMA`
6. `GAMMALN`
7. `GAMMALN.PRECISE`
8. `WEIBULL`
9. `WEIBULL.DIST`

## Native Excel Baseline
Pinned spot rows on the current reference Excel baseline:
1. `ERF(1) -> 0.8427007929497149`
2. `ERF(0,1) -> 0.8427007929497149`
3. `ERF(1,2) -> 0.15262147206923782`
4. `ERF.PRECISE(-1) -> -0.8427007929497149`
5. `ERFC(1) -> 0.15729920705028513`
6. `ERFC.PRECISE(-1) -> 1.8427007929497148`
7. `GAMMA(5) -> 24`
8. `GAMMA(0.5) -> 1.772453850905516`
9. `GAMMA(-0.5) -> -3.5449077018110318`
10. `GAMMA(-1) -> #NUM!`
11. `GAMMA(172) -> #NUM!`
12. `GAMMALN(5) -> 3.1780538303479458`
13. `GAMMALN(0.5) -> 0.5723649429247001`
14. `GAMMALN(0) -> #NUM!`
15. `WEIBULL(2,3,4,TRUE) -> 0.11750309741540463`
16. `WEIBULL.DIST(2,3,4,FALSE) -> 0.1654681692346117`
17. `WEIBULL.DIST(0,3,4,TRUE) -> 0`
18. `WEIBULL.DIST(0,3,4,FALSE) -> 0`
19. `WEIBULL.DIST(-1,3,4,TRUE) -> #NUM!`
20. `WEIBULL.DIST(2,0,4,TRUE) -> #NUM!`
21. `WEIBULL.DIST(2,3,0,TRUE) -> #NUM!`

## Current Implementation Notes
1. The Rust family file is self-contained and OxFunc-shaped: metadata, kernels, prepared evaluators, surface evaluators, error mapping, and unit tests all live in the same file.
2. `ERF` supports both the one-argument and interval forms; `ERF.PRECISE` is pinned to the one-argument form only.
3. `ERFC` and `ERFC.PRECISE` are treated as the same numeric kernel in the current baseline.
4. `GAMMA` uses a Lanczos-plus-reflection implementation so the admitted slice covers positive arguments and negative non-integers while still rejecting poles and overflow with `#NUM!`.
5. `GAMMALN` and `GAMMALN.PRECISE` are currently pinned only for positive inputs, matching the probed `#NUM!` lane at `0` and `-0.5`.
6. `WEIBULL` and `WEIBULL.DIST` share the same kernel; the last argument follows Excel's usual nonzero-is-TRUE flag coercion in the surface evaluator.
7. `W24` Batch 06 closed the previously open `WEIBULL.DIST(x=0,...,FALSE)` lane: Excel returns `0` for the pinned `alpha > 1`, `alpha = 1`, and `alpha < 1` density cases.
