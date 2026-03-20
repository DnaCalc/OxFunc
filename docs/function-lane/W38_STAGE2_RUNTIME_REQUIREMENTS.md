# W38 Stage 2 Runtime Requirements - `MAP`, `REDUCE`, `SCAN`

Status: `active`
Workset: `W38`
Evidence ID: `W38-MAP-REDUCE-SCAN-STAGE2-20260319`

## 1. Packet Purpose
1. replay the admitted current-baseline higher-order helper packet for `MAP`, `REDUCE`, and `SCAN`,
2. bind the first array-helper/callable semantics to a deterministic native worksheet artifact,
3. capture spill-shape evidence explicitly rather than flattening all helper results to single-cell text.

## 2. Inputs
1. scenario manifest:
   - `docs/function-lane/W38_STAGE2_MAP_REDUCE_SCAN_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host:
   - native Excel via COM
3. output artifact:
   - `.tmp/w38-map-reduce-scan-stage2-results.csv`
4. suite runner:
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
2. `set_err` rows are admission evidence, not runner failures,
3. rows with `inspect_range` must match both the scalar expectation and the spill-shape expectation when provided,
4. any unexpected mismatch is a semantic issue unless later reclassified as a host or publication seam limitation.
