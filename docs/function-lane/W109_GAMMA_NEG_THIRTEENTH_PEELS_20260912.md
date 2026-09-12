# GAMMA k/13-1 peels (2026-09-12)

Live Excel 16.0 build 20326 / CV2, Range.Value2 inject. Honest uint64 ULP.

Worksheet `GAMMA(x)=GAMMA(x+1)/x` is exact for 7/12 of `x=k/13-1`
(k=1,2,6,7,9,10,11). IEEE `GAMMA(1/13)/(1/13-1)` is 5 ULP from
`GAMMA(1/13-1)`. Those seven are published-bit seeds in (-1,0).
k=3,4,5,8,12 miss the worksheet peel by 1 ULP and stay generic.
