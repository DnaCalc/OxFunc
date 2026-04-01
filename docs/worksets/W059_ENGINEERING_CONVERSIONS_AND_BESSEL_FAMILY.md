# WORKSET - Engineering Conversions And Bessel Family (W59)

## 1. Purpose
Promote the first ordinary successor packet from the normalized `W051` backlog by closing the engineering radix conversion family and the Bessel quartet for the current reference Excel baseline.

This packet closes:
1. `BESSELI`
2. `BESSELJ`
3. `BESSELK`
4. `BESSELY`
5. `BIN2DEC`
6. `BIN2HEX`
7. `BIN2OCT`
8. `DEC2BIN`
9. `DEC2HEX`
10. `DEC2OCT`
11. `HEX2BIN`
12. `HEX2DEC`
13. `HEX2OCT`
14. `OCT2BIN`
15. `OCT2DEC`
16. `OCT2HEX`

## 2. Dependencies
1. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
2. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
3. prior implementation/evidence lineage from:
   - `docs/function-lane/W16_BATCH41_ENGINEERING_RADIX_NOTES.md`
   - `docs/function-lane/W16_BATCH65_BESSEL_CONVERT_NOTES.md`
   - `crates/oxfunc_core/src/functions/engineering_radix_family.rs`
   - `crates/oxfunc_core/src/functions/bessel_convert_family.rs`
   - `formal/lean/OxFunc/Functions/EngineeringRadixFamily.lean`
   - `formal/lean/OxFunc/Functions/BesselConvertFamily.lean`

## 3. Scope
In scope:
1. native Excel replay for the declared current-baseline engineering-radix and Bessel slices,
2. closure-grade OxFunc runtime and dispatch evidence for all `16` rows,
3. Lean substrate/binding alignment for both subfamilies,
4. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
5. `W051` removal/update for the `16` rows.

Out of scope:
1. `CONVERT`,
2. locale/version sweeps beyond the declared current reference baseline,
3. widening beyond the current engineering radix and Bessel surfaces.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_ENGINEERING_CONVERSIONS_AND_BESSEL_FAMILY_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W59_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W59_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W59_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W59_EXECUTION_RECORD.md`,
7. `.tmp/w59-engineering-conversions-bessel-results.csv`,
8. updated `W051` and downstream snapshot/labeling surfaces.

## 5. Gate Criteria
`W59` is complete when:
1. native Excel replay exists and matches the seeded engineering-radix and Bessel scenarios,
2. targeted Rust tests pass for both subfamilies,
3. `lake build` passes,
4. the snapshot generator emits these `16` rows with real metadata rather than `catalog_only`,
5. the `16` rows are removed from active `W051` backlog counts.

## 6. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W059` scope
