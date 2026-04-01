# W56 Grouped Aggregation Runtime Requirements

Status: `active`
Owner lane: `OxFunc`
Workset: `W56`

## 1. Purpose
Define the runtime expectations for the native Excel grouped-aggregation baseline probe.

## 2. Required Host Surface
1. desktop Excel available through COM automation,
2. local workbook creation allowed,
3. worksheet formula evaluation through `Formula2`,
4. dynamic-array worksheet functions `GROUPBY`, `PIVOTBY`, and `ARRAYTOTEXT` available on the installed host,
5. no XLL surface required for this packet.

## 3. Source Inputs
1. `docs/function-lane/W56_GROUPED_AGGREGATION_SCENARIO_MANIFEST_SEED.csv`

## 4. Output Artifact
1. `.tmp/w56-grouped-aggregation-results.csv`

## 5. Output Fields
1. `scenario_id`
2. `lane`
3. `formula`
4. `text`
5. `value2`
6. `expected_text`
7. `matches_expected`
8. `excel_version`
9. `excel_product_version`
10. `notes`

## 6. Interpretation Rule
1. `GROUPBY` rows pin the current local baseline for default grouping, callable-slot carriage, hierarchical subtotals, visible headers, filter/sort sensitivity, and tabular-subtotal rejection.
2. `PIVOTBY` rows pin the current local baseline for default pivoting, callable-slot carriage, visible header bands, filter/zero-totals sensitivity, and row/column-total sort behavior.
3. `ARRAYTOTEXT(...,1)` is the worksheet-observable projection used to stabilize dynamic-array results into scalar text for replay comparison.
