# W24 Batch 15 Runtime Requirements - Misc Ordinary Conversion Packet

Status: `active`
Workset: `W24`
Evidence ID: `W24-B15-MISC-ORDINARY-CONVERSION-20260318`

## 1. Packet Purpose
1. replay the ordinary `BAHTTEXT` / `CONVERT` / `PERCENTOF` closure packet,
2. pin the current host-baseline absence of `EUROCONVERT` and `RANDARRA`,
3. justify extraction of the outliers to `W025`.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W24_BATCH15_MISC_ORDINARY_CONVERSION_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host: native Excel via COM
3. output artifact: `.tmp/w24-batch15-misc-ordinary-conversion-results.csv`

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
2. the `BAHTTEXT` / `CONVERT` / `PERCENTOF` rows prove the ordinary triad closure slice,
3. the `EUROCONVERT` / `RANDARRA` `#NAME!` rows are extraction evidence, not ordinary closure claims,
4. no XLL seam limitation is expected for this native worksheet packet.
