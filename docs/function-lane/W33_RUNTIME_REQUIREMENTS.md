# W33 Runtime Requirements - Information Predicates And Forecast Compatibility

Status: `active`
Workset: `W33`
Evidence ID: `W33-INFO-FORECAST-20260319`

## 1. Packet Purpose
1. replay the admitted current-baseline packet for the information-predicate family and the `FORECAST` / `FORECAST.LINEAR` compatibility pair,
2. bind the newly promoted catalog members to a deterministic native worksheet artifact,
3. keep the forecast compatibility question explicit rather than assuming aliasing without evidence.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W33_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host: native Excel via COM
3. output artifact: `.tmp/w33-info-forecast-results.csv`

## 3. Output Columns
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

## 4. Acceptance Rule
1. every seeded row must emit `matches_expected = True`,
2. any failure is a semantic issue unless explicitly classified as an XLL or host-provider seam limitation,
3. no XLL seam limitation is expected for this native worksheet packet.
