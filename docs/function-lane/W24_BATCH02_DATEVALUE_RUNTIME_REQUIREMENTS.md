# W24 Batch 02 - Date Value Family Runtime Requirements

## 1. Purpose
Define the runtime expectations for the native Excel date-value family baseline used by `W24` Batch 02.

## 2. Inputs
1. scenario manifest:
   - `docs/function-lane/W24_BATCH02_DATEVALUE_SCENARIO_MANIFEST_SEED.csv`
2. runner:
   - `tools/w24-probe/run-w24-batch02-datevalue-baseline.ps1`
3. output artifact:
   - `.tmp/w24-batch02-datevalue-results.csv`

## 3. Environment
1. local Excel COM host
2. default workbook baseline
3. locale `en-US`

## 4. Expected Output Shape
CSV columns:
1. `scenario_id`
2. `lane`
3. `formula`
4. `text`
5. `value2`
6. `expected_text`
7. `expected_value2`
8. `tolerance`
9. `matches_expected`
10. `notes`

## 5. Acceptance Rule
1. rows with `expected_text` compare on the worksheet text surface
2. rows with `expected_value2` compare numerically with the supplied tolerance
3. every seeded row must emit `matches_expected=True`

## 6. Explain Notes
1. this packet intentionally closes the admitted `en-US` host-profile text subset, not all locale text-date parsing
2. broader locale/version replay remains orthogonal validation work
