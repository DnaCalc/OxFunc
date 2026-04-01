# WORKSET - Math, Matrix, And Rounding Family (W067)

## 1. Purpose
Promote the ninth ordinary successor packet from the normalized `W051` backlog by closing the math, matrix, and rounding family for the current reference Excel baseline.

This packet closes:
1. `CEILING.MATH`
2. `CEILING.PRECISE`
3. `FLOOR`
4. `FLOOR.MATH`
5. `FLOOR.PRECISE`
6. `ISO.CEILING`
7. `MDETERM`
8. `MINVERSE`
9. `MMULT`
10. `MUNIT`
11. `SERIESSUM`
12. `SUMPRODUCT`
13. `SUMX2MY2`
14. `SUMX2PY2`
15. `SUMXMY2`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. prior implementation/evidence lineage from:
   - `docs/function-lane/W16_BATCH32_CEILING_FLOOR_NOTES.md`
   - `docs/function-lane/W16_BATCH45_MATRIX_FAMILY_NOTES.md`
   - `docs/function-lane/W16_BATCH47_SUMPRODUCT_NOTES.md`
   - `crates/oxfunc_core/src/functions/ceiling_floor_family.rs`
   - `crates/oxfunc_core/src/functions/matrix_family.rs`
   - `crates/oxfunc_core/src/functions/sumproduct_family.rs`
   - `formal/lean/OxFunc/Functions/CeilingFloorFamily.lean`
   - `formal/lean/OxFunc/Functions/MatrixFamily.lean`
   - `formal/lean/OxFunc/Functions/SumproductFamily.lean`

## 3. Scope
In scope:
1. current-baseline promotion of the already-evidenced rounding, matrix, and sumproduct-family rows into the ordinary-backlog closure program,
2. native Excel replay for all `15` normalized rows through a dedicated `W067` manifest and runner,
3. closure-grade OxFunc runtime and dispatch evidence for the admitted current-baseline slice,
4. Lean substrate/binding alignment confirmation for the existing rounding, matrix, and sumproduct formal artifacts,
5. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
6. `W051` removal/update for the `15` hidden snapshot entries and `15` normalized execution rows.

Out of scope:
1. legacy `CEILING`, which is provenance/evidence input but not part of the remaining hidden ordinary backlog,
2. locale/version sweeps beyond the declared current reference baseline,
3. workbook-version sweeps and historical-version sweeps,
4. reshaping or broad dynamic-array characterization beyond the admitted matrix spill lanes already pinned by the current baseline.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_MATH_MATRIX_AND_ROUNDING_FAMILY_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W67_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W67_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W67_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W67_EXECUTION_RECORD.md`,
7. `tools/w67-probe/run-w67-math-matrix-rounding-baseline.ps1`,
8. `.tmp/w67-math-matrix-rounding-results.csv`,
9. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`,
10. refreshed downstream `W051` / snapshot / policy / worklist counters.

## 5. Completion Gate
`W067` may be reported complete only when:
1. every covered row has at least one explicit native Excel replay lane in `W67_SCENARIO_MANIFEST_SEED.csv`,
2. native Excel replay matches the seeded `W67` manifest for all declared lanes,
3. the targeted Rust tests for the rounding, matrix, and sumproduct-family sources pass,
4. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml --all --check` passes,
5. `lake build` passes without introducing new active `W054` gaps,
6. all `15` `W067` hidden snapshot entries move out of `catalog_only` in the regenerated `W44` snapshot,
7. `W051` and downstream counts reconcile to the post-`W067` backlog (`3` hidden snapshot entries / `3` normalized execution rows).

## 6. Notes
1. `W067` is a promotion/reconciliation packet, not a fresh semantic-discovery packet: the current-baseline rounding, matrix, and sumproduct-family semantics already have local runtime and empirical evidence across the `W16` batches.
2. The matrix replay rows use deterministic string materialization of spilled outputs so the native Excel baseline can be compared without depending on host-grid formatting.
