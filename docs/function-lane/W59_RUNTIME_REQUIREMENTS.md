# W59 Engineering Conversions And Bessel Runtime Requirements

Status: `active`
Owner lane: `OxFunc`
Workset: `W059`

## 1. Purpose
Define the runtime expectations for the native Excel baseline probe used to close `W059`.

## 2. Required Host Surface
1. desktop Excel available through COM automation,
2. local workbook creation allowed,
3. worksheet formula evaluation through `Formula2`,
4. the engineering radix conversion family and the Bessel quartet available on the installed host,
5. no XLL surface required for the native replay probe.

## 3. Source Inputs
1. `docs/function-lane/W59_SCENARIO_MANIFEST_SEED.csv`

## 4. Output Artifact
1. `.tmp/w59-engineering-conversions-bessel-results.csv`

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
1. engineering-radix rows pin the current baseline for truncation, fixed-width signed interpretation, padding, invalid-source rejection, and overflow rejection,
2. Bessel rows pin the current baseline for seeded numeric values, order truncation, sign parity, and `#NUM!` domain rejection lanes.
