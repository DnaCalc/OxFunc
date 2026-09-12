# GAMMA k/d-1 peels in (-1,0) (2026-09-12)

Live Excel 16.0 build 20326 / CV2, Range.Value2 inject. Honest uint64 ULP.

Worksheet `GAMMA(x)=GAMMA(x+1)/x` is exact for:

- 7/12 of `x=k/13-1` (k=1,2,6,7,9,10,11). IEEE `GAMMA(1/13)/(1/13-1)`
  is 5 ULP from `GAMMA(1/13-1)`.
- 8/10 of `x=k/11-1` (k=2,3,5,6,7,8,9,10; k=1,4 miss 1 ULP).
- 5/16 of `x=k/17-1` (k=2,3,11,13,16).

Those residues are published-bit seeds. Misses stay generic.

Second peel into (-2,-1) from exact first-peel thirteenths is exact
for k=2,10,11 (3/7). k=1,6,7,9 miss 1-2 ULP.

Third peel into (-3,-2) from those three is exact 3/3.
