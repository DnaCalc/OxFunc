# W45 Wave A Runtime Requirements - Operator Arithmetic Family

Status: `active`
Workset: `W45`
Evidence ID: `W45-OP-ARITH-WAVEA-20260320`

## 1. Packet Purpose
1. bind the first executable non-`@` operator slice to a native Excel worksheet packet,
2. prove the current admitted arithmetic operator lanes against direct worksheet formulas,
3. capture spill-grid broadcast behavior rather than only anchor-cell scalar outcomes,
4. keep the rest of `W45` explicit rather than pretending the whole operator universe is already closed.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host: native Excel via COM
3. output artifact: `.tmp/w45-wavea-operator-arithmetic-results.csv`

## 3. Output Columns
1. `scenario_id`
2. `lane`
3. `formula`
4. `text`
5. `value2`
6. `has_spill`
7. `spill_rows`
8. `spill_cols`
9. `spill_text`
10. `expected_text`
11. `expected_value2`
12. `expected_error`
13. `expected_spill_text`
14. `expected_spill_rows`
15. `expected_spill_cols`
16. `tolerance`
17. `matches_expected`
18. `notes`

## 4. Acceptance Rule
1. every seeded row must emit `matches_expected = True`,
2. any failure is a semantic issue unless explicitly reclassified as a host or parser-seam limitation,
3. no XLL seam limitation is expected for this native worksheet packet.
