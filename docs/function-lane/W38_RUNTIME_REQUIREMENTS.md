# W38 Runtime Requirements - Functional Lambda And Helper Family Stage 1

Status: `active`
Workset: `W38`
Evidence ID: `W38-LAMBDA-HELPER-STAGE1-20260319`

## 1. Packet Purpose
1. replay the admitted current-baseline Stage 1 helper/callable packet for `LET`, immediate `LAMBDA`, and the currently observable `ISOMITTED` lanes,
2. bind the first helper/callable family claims to a deterministic native worksheet artifact,
3. keep admission-time failures distinct from worksheet-surface evaluation results.

## 2. Inputs
1. scenario manifest:
   - `docs/function-lane/W38_SCENARIO_MANIFEST_SEED.csv`
2. worksheet host:
   - native Excel via COM
3. output artifact:
   - `.tmp/w38-lambda-helper-stage1-results.csv`
4. suite runner:
   - `tools/w38-probe/run-w38-suite.ps1`

## 3. Output Columns
1. `scenario_id`
2. `lane`
3. `formula`
4. `admission_status`
5. `stored_formula2`
6. `text`
7. `value2`
8. `expected_status`
9. `expected_text`
10. `expected_value2`
11. `tolerance`
12. `matches_expected`
13. `notes`

## 4. Acceptance Rule
1. every seeded row must emit `matches_expected = True`,
2. `set_err` rows are admission-behavior evidence, not runner failures,
3. any unexpected mismatch is a semantic issue unless explicitly classified later as a host or formula-admission seam limitation,
4. no XLL seam limitation is expected for this native worksheet packet.
