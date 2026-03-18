# W24 Batch 04 Runtime Requirements - Array Text Split Family

Status: `active`
Workset: `W24`
Evidence ID: `W24-B04-ARRAY-TEXT-SPLIT-20260318`

## 1. Packet Purpose
1. replay the admitted current-baseline packet for `ARRAYTOTEXT` and `TEXTSPLIT`,
2. capture `TEXTSPLIT` arrays through scalar `ARRAYTOTEXT(TEXTSPLIT(...),1)` witnesses,
3. bind the family to a deterministic worksheet artifact in `.tmp`.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W24_BATCH04_ARRAY_TEXT_SPLIT_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host: native Excel via COM
3. output artifact: `.tmp/w24-batch04-array-text-split-results.csv`

## 3. Output Columns
1. `scenario_id`
2. `lane`
3. `formula`
4. `text`
5. `value2`
6. `expected_text`
7. `matches_expected`
8. `notes`

## 4. Acceptance Rule
1. every seeded row must emit `matches_expected = True`,
2. the `TEXTSPLIT` witness path in this packet is scalarized on purpose through `ARRAYTOTEXT(...,1)`,
3. no XLL seam limitation is expected for this native worksheet packet.
