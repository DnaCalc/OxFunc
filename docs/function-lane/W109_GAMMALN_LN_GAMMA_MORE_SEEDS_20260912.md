# W109 GAMMALN = LN(GAMMA) additional seeds (2026-09-12)

Live Excel 16.0 build 20326 / CV2 / 64-bit. Inputs via `Range.Value2`;
per-cell `Cells.Item` formulas; honest uint64 bit-distance.

Worksheet `GAMMALN(x)` equals worksheet `LN(GAMMA(x))` at further landed
GAMMA seeds beyond the earlier 1/5, 1/3, 1/7 (and 1/2 via the piecewise
kernel):

| x | GAMMALN bits | notes |
|---|---|---|
| 2/7 | `0x3ff25a9c12810ff0` | 4/7, 6/7 are not this graph |
| 3/7 | `0x3fe73e3996add6b2` | |
| 5/7 | `0x3fcf325cd39aec45` | |
| 3/8 | `0x3feb9e4d53fc3074` | 1/8, 5/8, 7/8 miss 1–10 ULP |
| 5/16 | `0x3ff0d8e1683cdcd1` | 1/16, 9/16 miss |
| 1/11 | `0x4002d0c317ddb0c0` | |
| 2/11 | `0x3ff9ff587e1ad8ce` | |
| 6/11 | `0x3fdf3ad24caad6dc` | other elevenths miss 1–24 ULP |
| 1/13 | `0x400433b1c2265eb7` | |
| 2/13 | `0x3ffcd17b700fc6c1` | |
| 1+1/13 | `0xbfa4549a42cb6665` | product-first GAMMA(14/13) |
| 2/17 | `0x4000a9daf888fcc2` | 1/17 is 1 ULP |
| 2.5 | `0x3fd2383e809a67e8` | half-integer product-first |

Production publishes `excel_log(gamma_kernel(x))` at those x bits.
`GAMMALN(x+1)=GAMMALN(x)+LN(x)` is 0/10 on the same seed set (max 27 ULP)
and is not the graph.

CHIINV / `CHISQ.INV.RT(p,2)` vs worksheet `-2*LN(p)` is 17/18 (p=0.8 is
1 ULP). Alternate LN associations do not recover the miss. Not landed.
