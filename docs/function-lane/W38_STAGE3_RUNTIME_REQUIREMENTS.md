# W38 Runtime Requirements - Functional Lambda And Helper Family Stage 3

Status: `active`
Workset: `W38`
Evidence ID: `W38-BYROW-BYCOL-MAKEARRAY-DEFINED-NAMES-STAGE3-20260319`

## 1. Packet Purpose
1. replay the admitted current-baseline Stage 3 helper/callable packet for `BYROW`, `BYCOL`, `MAKEARRAY`, and workbook Defined Name callable preservation,
2. bind the remaining `W38` inventory members to a deterministic native worksheet artifact,
3. keep workbook-name setup explicit and reproducible rather than hiding it in prose.

## 2. Inputs
1. scenario manifest:
   - `docs/function-lane/W38_STAGE3_BYROW_BYCOL_MAKEARRAY_DEFINED_NAMES_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host:
   - native Excel via COM
3. workbook-name setup:
   - `MyAdder := LAMBDA(x,x+1)`
   - `CapAdd := LET(x,2,LAMBDA(y,x+y))`
4. output artifact:
   - `.tmp/w38-stage3-byrow-bycol-makearray-defined-names-results.csv`
5. suite runner:
   - `tools/w38-probe/run-w38-suite.ps1`

## 3. Output Columns
1. `scenario_id`
2. `lane`
3. `anchor_cell`
4. `formula`
5. `admission_status`
6. `stored_formula2`
7. `text`
8. `value2`
9. `inspect_range`
10. `spill_text`
11. `expected_status`
12. `expected_text`
13. `expected_value2`
14. `expected_spill_text`
15. `tolerance`
16. `matches_expected`
17. `notes`

## 4. Acceptance Rule
1. every seeded row must emit `matches_expected = True`,
2. `set_err` rows are admission-behavior evidence, not runner failures,
3. workbook-name setup is part of the deterministic packet, not an external seam limitation,
4. any unexpected mismatch is a semantic issue unless explicitly classified later as a host or formula-admission seam limitation,
5. no XLL seam limitation is expected for this native worksheet packet itself,
6. but helper-family worksheet parity is not yet reproducible through the XLL bridge because callable worksheet values and workbook Defined Name callable bindings are not yet transportable there.
