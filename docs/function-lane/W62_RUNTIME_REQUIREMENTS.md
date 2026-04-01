# W62 Statistical Distributions And Compat B Runtime Requirements

Status: `active`
Owner lane: `OxFunc`
Workset: `W062`

## 1. Purpose
Define the runtime expectations for the native Excel baseline probe used to close `W062`.

## 2. Required Host Surface
1. desktop Excel available through COM automation,
2. local workbook creation allowed,
3. worksheet formula evaluation through `Formula2`,
4. the covered statistical-distribution and compatibility surface available on the installed host,
5. no XLL surface required for the native replay probe.

## 3. Source Inputs
1. `docs/function-lane/W62_SCENARIO_MANIFEST_SEED.csv`

## 4. Output Artifact
1. `.tmp/w62-statistical-distributions-compat-b-results.csv`

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
1. the replay corpus pins current-baseline publication for the `W062` statistical-distribution and residual compatibility wave,
2. numeric observation lanes use worksheet-side `ROUND(...)` where needed to stabilize current-baseline decimal comparison,
3. error-result lanes compare worksheet-published error text rather than the numeric `Value2` code,
4. alias rows are included directly so compatibility publication is verified in native Excel rather than inferred from local runtime equality.
