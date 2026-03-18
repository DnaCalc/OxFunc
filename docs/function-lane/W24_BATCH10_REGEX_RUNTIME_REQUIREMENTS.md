# W24 Batch 10 Runtime Requirements - Regex Triad

Status: `active`
Workset: `W24`
Evidence ID: `W24-B10-REGEX-20260318`

## 1. Packet Purpose
1. replay the admitted current-baseline packet for `REGEXEXTRACT`, `REGEXREPLACE`, and `REGEXTEST`,
2. pin the bounded pure regex surface independently from `NUMBERVALUE` and `TRANSLATE`,
3. bind the trio to a deterministic worksheet artifact in `.tmp`.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W24_BATCH10_REGEX_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host: native Excel via COM
3. output artifact: `.tmp/w24-batch10-regex-results.csv`

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
2. any failure is a semantic issue unless explicitly classified as an external XLL seam limitation,
3. no XLL seam limitation is expected for this native worksheet packet.
