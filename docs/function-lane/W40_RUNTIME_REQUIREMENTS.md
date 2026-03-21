# W40 Runtime Requirements

Status: `active`
Packet: `W040`

## 1. Native Excel Baseline
The `W040` native packet runner must:
1. create a deterministic workbook with seeded sheets `Quarter 1`, `Beta`, and `Alpha`,
2. place seeded formula/value cells on `Alpha`,
3. evaluate the declared scenario formulas on `Alpha`,
4. write `.tmp/w40-reference-metadata-results.csv`.

## 2. Success Conditions
1. every seeded row in `docs/function-lane/W40_SCENARIO_MANIFEST_SEED.csv` must have an observed Excel text result,
2. `matches_expected` must be `TRUE` for all declared rows,
3. the packet must make the host/grid seam explicit for:
   - stored formula visibility,
   - current-sheet identity,
   - workbook sheet order and span counting,
4. the packet must pin the exact OxFunc callback/query surface as:
   - `query_formula_text(reference)`,
   - `query_sheet_index(CurrentSheet | Reference | SheetNameText)`,
   - `query_sheet_count(Workbook | Reference)`,
5. the packet must not overclaim in-core pure runtime support where host metadata is still required.

## 3. Current Slice Boundary
This packet does not yet require:
1. full host-side consumer wiring for the three callback queries,
2. XLL bridge parity for formula-storage or workbook-sheet-topology lanes,
3. full cross-version or cross-locale sweeps.
