# W24 Batch 03 Runtime Requirements - Text Delimiter Family

Status: `active`
Workset: `W24`
Evidence ID: `W24-B03-TEXT-DELIM-20260318`

## 1. Packet Purpose
1. replay the admitted current-baseline scalar packet for `TEXTAFTER` and `TEXTBEFORE`,
2. confirm that the integrated runtime still matches native Excel for the seeded delimiter lanes,
3. bind the batch to a deterministic worksheet artifact in `.tmp`.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W24_BATCH03_TEXT_DELIM_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host: native Excel via COM
3. output artifact: `.tmp/w24-batch03-text-delim-results.csv`

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
2. any failure is a semantic issue unless explicitly classified as an external XLL seam limitation,
3. no XLL seam limitation is expected for this native worksheet packet.
