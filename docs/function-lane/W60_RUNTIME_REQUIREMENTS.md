# W60 Complex Number Family Runtime Requirements

Status: `active`
Owner lane: `OxFunc`
Workset: `W060`

## 1. Purpose
Define the runtime expectations for the native Excel baseline probe used to close `W060`.

## 2. Required Host Surface
1. desktop Excel available through COM automation,
2. local workbook creation allowed,
3. worksheet formula evaluation through `Formula2`,
4. the complex-number family available on the installed host,
5. no XLL surface required for the native replay probe.

## 3. Source Inputs
1. `docs/function-lane/W60_SCENARIO_MANIFEST_SEED.csv`

## 4. Output Artifact
1. `.tmp/w60-complex-family-results.csv`

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
1. the replay corpus pins current-baseline parsing, formatting, suffix preservation, numeric publication, mixed-suffix rejection, and domain/error behavior for the complex-number family,
2. numeric observation lanes use worksheet-side `ROUND(...)` to stabilize published current-baseline decimal comparison,
3. text-result lanes compare the direct worksheet-published complex string.
