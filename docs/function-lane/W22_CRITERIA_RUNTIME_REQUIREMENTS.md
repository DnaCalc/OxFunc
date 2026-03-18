# W22 Criteria Runtime Requirements

Status: `active`
Owner lane: `OxFunc`
Workset: `W22`

## 1. Purpose
Define the runtime expectations for the native Excel criteria-family shape probe.

## 2. Required Host Surface
1. desktop Excel available through COM automation,
2. local workbook creation allowed,
3. worksheet formula evaluation through `Formula2`,
4. no XLL surface required for this packet.

## 3. Source Inputs
1. `docs/function-lane/W22_CRITERIA_SHAPE_SCENARIO_MANIFEST_SEED.csv`

## 4. Output Artifact
1. `.tmp/w22-criteria-shape-results.csv`

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
1. `AVERAGEIF` rows are the anchoring-sensitive rows for this packet.
2. `COUNTIFS`, `SUMIFS`, `AVERAGEIFS`, `MAXIFS`, and `MINIFS` rows are exact-shape confirmation rows for the current baseline.
3. workbook/environment differences are not expected to dominate this packet because the tested formulas are pure worksheet calculations over local literals.
