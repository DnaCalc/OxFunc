# W52 Residual Seam Review - 2026-03-26

Purpose: record the conservative review pass over the remaining `oxfunc_function_corpus_passes_through_adapter` failures after standalone `SUMIF` closure.

Scope:
1. verify worksheet-value mismatches against live Excel,
2. classify each residual as fixture drift, OxFml seam behavior, or probable OxFunc parity drift,
3. avoid changing OxFunc function implementations or metadata profiles without confirmation.

## Live Excel Readback

Verified on `2026-03-26` through local Excel COM:

1. `=ASIN(0.5)` -> `0.5235987755982989`
2. `=ASINH(1)` -> `0.8813735870195429`
3. `=SIGN(-5)` -> `-1`
4. `=DATE(2026,3,25)` -> `46106`
5. `=DAY(46107)` -> `26`
6. `=PV(0.05,10,-100)` -> `772.1734929184813`
7. `=FV(0.05,10,-100)` -> `1257.789253554883`
8. `=PMT(0.05,10,1000)` -> `-129.50457496545667`
9. `=IFNA(A1,0)` with `A1 = #N/A` -> `0`
10. `=ISBLANK(A1)` on a blank `A1` -> `TRUE`
11. `=COUNTIF(C1:C5,">3")` with `1,2,4,5,6` -> `3`
12. `=AVERAGEIF(D1:D4,">2")` with `1,2,3,4` -> `3.5`

## Fixture Corrections Applied

The pinned OxFunc-side corpus in `crates/oxfunc_core/tests/fixtures/oxfunc_adapter_function_corpus.json` was corrected for the confirmed fixture-only rows:

1. `FN-ASIN-01`
2. `FN-DATE-01`
3. `FN-DAY-01`
4. `FN-EDATE-01`
5. `FN-IFNA-01`
6. `FN-PV-01`
7. `FN-FV-01`
8. `FN-LARGE-01`
9. `FN-SMALL-01`
10. `FN-COUNTIF-01`
11. `FN-AVERAGEIF-01`

Rationale:
1. the numeric/date rows above were pinned to direct Excel outputs,
2. `IFNA` is admitted as `RefsVisibleInAdapter`, so `["DirectScalar", "DirectScalar"]` was not a credible expectation,
3. `LARGE` remains `ValuesOnlyPreAdapter`, so an inline array constant should not expect `ReferenceVisible`,
4. `COUNTIF` and `AVERAGEIF` are on the criteria-family refs-visible substrate.

## Remaining Non-Fixture Issues

1. `ASINH` still differs from live Excel in the final low-order bit under the current strict string-equality seam comparison: Excel `0.8813735870195429` vs current OxFunc `0.881373587019543`.
2. `SIGN`, `PV`, and `FV` still fail through the adapter when the formula contains a negative literal. This currently looks like an OxFml unary-minus seam issue rather than an OxFunc function-definition issue.
3. `ISBLANK` still fails because the OxFml local resolver returns `UnresolvedReference` for an absent single-cell reference instead of treating it as a blank cell in the stand-in host.
4. `PMT` still appears to be a real low-order numeric parity drift on the OxFunc side and should be isolated separately before any finance-family code change.
