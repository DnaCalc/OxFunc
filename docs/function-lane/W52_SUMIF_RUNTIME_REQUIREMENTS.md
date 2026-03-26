# W52 SUMIF Runtime Requirements

Status: `active`
Owner lane: `OxFunc`
Workset: `W52`

## 1. Purpose
Define the runtime expectations for the native Excel `SUMIF` completion probe.

## 2. Required Host Surface
1. desktop Excel available through COM automation,
2. local workbook creation allowed,
3. worksheet formula evaluation through `Formula2`,
4. no XLL surface required for this packet.

## 3. Source Inputs
1. `docs/function-lane/W52_SUMIF_SCENARIO_MANIFEST_SEED.csv`

## 4. Output Artifact
1. `.tmp/w52-sumif-results.csv`

## 5. Output Fields
1. `scenario_id`
2. `lane`
3. `formula`
4. `text`
5. `value2`
6. `expected_text`
7. `matches_expected`
8. `notes`

## 6. Interpretation Rule
1. omitted `sum_range` rows confirm that `SUMIF` uses the criteria range directly.
2. anchored rows confirm that an explicit mismatched A1-style `sum_range` expands from its top-left reference over the criteria-range shape.
3. aggregation rows confirm numeric-only target summation and reached target-error propagation.
4. workbook/environment differences are not expected to dominate this packet because the tested formulas are pure worksheet calculations over local literals.
