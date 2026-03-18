# W16 Batch 12 - Aggregate Arithmetic Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH12-PRODUCT-SUMSQ-20260315`

## Scope
1. `PRODUCT`
2. `SUMSQ`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch12-product-sumsq-probe.csv`

Pinned lanes:
1. `PRODUCT(2,3,4) -> 24`
2. `PRODUCT(TRUE,"2") -> 2`
3. `PRODUCT("x") -> #VALUE!`
4. `PRODUCT(F1:F3) -> 0`
5. `PRODUCT(G1:G2) -> 0`
6. `PRODUCT(G1:G3) -> #N/A`
7. `SUMSQ(2,3,4) -> 29`
8. `SUMSQ(TRUE,"2") -> 5`
9. `SUMSQ("x") -> #VALUE!`
10. `SUMSQ(F1:F3) -> 13`
11. `SUMSQ(G1:G2) -> 0`
12. `SUMSQ(G1:G3) -> #N/A`
13. a native Excel follow-up confirmed `PRODUCT` returns `0` rather than `1` when every surviving reference-derived item is ignored.
