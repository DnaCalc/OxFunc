# W10 Ten-Function Mixed Seams Execution Record

Status: `in_progress-provisional`
Workset: `W10`
Evidence ID: `W10-TENMIX-SEED-20260308`
Lookup Follow-Up Evidence ID: `W10-LOOKUP-XLL-20260310`

## 1. Purpose
Track W10 execution status across runtime/formal scaffolding, scenario manifests, and classification side-notes.

## 2. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - `SUM` still depends on richer provenance-aware aggregate input modeling before full Excel closure can be claimed across all lanes.
   - `INDEX` still lacks nontrivial `area_num` semantics and broader reference-form edge coverage.
   - `INDIRECT` remains incomplete across broader ref-text semantics (for example structured, external, and richer workbook/sheet-address forms).
   - `SEQUENCE` still needs broader dynamic-array behavior closure beyond the current replayed shape/materialization lanes.
   - XLL verification-seam limits remain external and must not be mistaken for function-semantic closure.
6. function-phase-complete slices within W10:
   - `IF`
   - `ISNUMBER`
   - `MATCH`
   - `NOW`
   - `XLOOKUP`

## 3. Executed Scope
Execution date:
1. `2026-03-09`
2. `2026-03-10` (function-phase-complete promotion follow-up)
3. `2026-03-10` (lookup-family closure follow-up)

Function slices with landed scaffolding/runtime seeds:
1. `SUM`
2. `IF`
3. `INDEX`
4. `MATCH`
5. `ISNUMBER`
6. `NOW`
7. `XLOOKUP`
8. `INDIRECT`
9. `SEQUENCE`
10. `OP_ADD`

## 4. Output Artifacts
1. function contracts:
   - `docs/function-lane/FUNCTION_SLICE_SUM_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_IF_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_INDEX_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_MATCH_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_ISNUMBER_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_NOW_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_XLOOKUP_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_INDIRECT_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_SEQUENCE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_OP_ADD_CONTRACT_PRELIM.md`
2. scenario manifests:
   - `docs/function-lane/W10_S1_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S2_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S3_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S4_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`
3. runtime + formal modules:
   - Rust: `crates/oxfunc_core/src/functions/*`
   - Lean: `formal/lean/OxFunc/Functions/*`
4. side-note ledger:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md`
5. empirical outputs:
   - `.tmp/w10-scenarios-manifest.csv`
   - `.tmp/w10-results-default.csv`
   - `.tmp/w10-results-compat.csv`
   - `.tmp/w10-results-excel.csv`
   - `.tmp/w10-analysis-report.csv`
   - `.tmp/w10-analysis-summary.json`
   - `.tmp/w10-results-default.csv.run-metadata.json`
   - `.tmp/w10-results-compat.csv.run-metadata.json`
   - `.tmp/w10-artifacts/*`
6. replay tooling:
   - `tools/w10-probe/run-w10-suite.ps1`
   - `tools/w10-probe/analyze-w10-results.ps1`
   - `tools/w10-probe/new-w10-compat-template.ps1`
   - `tools/xll-addin/run-lookup-xll-bridge-suite.ps1`

## 5. Verification Runs
1. `cargo test -p oxfunc_core` -> pass (`204` tests).
2. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml` -> pass.
3. `./tools/xll-addin/sync-export-specs.ps1` -> pass.
4. `lake build` (from `formal/lean`) -> pass.
5. `powershell -File tools/w10-probe/run-w10-suite.ps1 -OutDir .tmp/lookup-pass` -> pass (`84` matched, `0` mismatched).
6. `powershell -File tools/xll-addin/run-lookup-xll-bridge-suite.ps1 -OutDir .tmp/lookup-pass` -> pass (`15` matched relation rows, including `2` explicit seam-divergence rows).

## 5. Gate Tracking
### G1 - Classification Closure
1. Status: `closed-provisional`.
2. Notes:
   - all ten slices have explicit class axes and profile rows in contract docs.

### G2 - Runtime/Formal Pairing Closure
1. Status: `closed-provisional`.
2. Notes:
   - each function now has a Rust module and Lean module.
   - multiple functions remain semantically incomplete (`INDEX`, `INDIRECT`, `SEQUENCE`, `SUM`) and are still scaffolding rather than closed implementations.

### G3 - Empirical Closure
1. Status: `closed-provisional`.
2. Notes:
   - default + `compat_template` Excel replays reran on `2026-03-10` after lookup-family expansion.
   - rows: `84` (`42` default + `42` compat_template).
   - expectation matched: `84`; mismatched: `0`.
   - execution failed unexpected: `0`.
   - dual-run requirement satisfied: `true`.
   - analyzer gate status: `green`.

### G4 - XLL Export Closure
1. Status: `closed-provisional`.
2. Notes:
   - export declarations are generated from `xll_export_specs` profile rules.
   - W10 functions are included in the generated export set with profile-derived U variants and admitted Q variants.
   - known XLL verification-seam limits remain external and are tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.

### G5 - Promotion Readiness
1. Status: `in_progress`.
2. Notes:
   - W10 has useful local scaffolding across contract, Rust, Lean, empirical replay, and export generation.
   - W10 does not satisfy packet-level implementation closure because several functions still carry known Excel-semantic gaps.
   - `IF`, `ISNUMBER`, `MATCH`, `NOW`, and `XLOOKUP` now satisfy current-phase function closure individually and may be reported as `function-phase-complete`.

## 6. XLL Verification-Seam Limitations
1. Relevant seam limits for the W10 packet are tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.
2. Material packet-level impacts:
   - reference-return and reference-resolved lookup-array lanes are not fully demonstrated through the XLL bridge,
   - registration-flag behavior remains outside ordinary profile-derived export generation,
   - XLL evidence must not be used to justify semantic closure for `INDIRECT` or `SEQUENCE`; for `XLOOKUP` it qualifies only the bridge evidence, not the core function-semantic claim.

## 7. Key Findings
1. strict runtime split remains workable for most non-interesting function shapes.
2. reference-return functions require explicit return policy axes and cannot be fully captured by values-only preparation.
3. volatile/provider seams are now visible as a first-class profile pressure point (`NOW`).
4. array payload modeling is now landed and removed the prior `SEQUENCE` shape-only blocker; `INDEX` still needs broader reference-form parity.
5. XLL codegen is now profile-derived over the catalog, removing per-function export-row hand curation.
6. empirical corrections from replay:
   - `MATCH(2,{1,2,3},1)` returns `2` (not `#N/A`).
   - `MATCH` approximate lanes on unsorted arrays follow Excel's older binary invalid-result behavior rather than the simpler bound logic first assumed.
   - `SEQUENCE(0)` returns `#CALC!` (not `#VALUE!`), and the runtime mapper now follows that worksheet lane.
   - `INDIRECT` with `a1_style=FALSE` follows R1C1 addressing when expression is valid (for example `R1C2`).
   - `XLOOKUP` return-array orientation/alignment matters; mismatched orientation yields `#VALUE!`.
   - `XLOOKUP(..., "nf")` returns the fallback before `#N/A` mapping on no-match.
   - `XLOOKUP` distinguishes true blank lookup values from literal empty string lookup values, and true blank return cells materialize as numeric zero.
   - `IF(FALSE,1)` defaults the omitted false branch to logical `FALSE`.
   - `ISNUMBER(1/0)` returns `FALSE`, and `ISNUMBER` classifies reference-fed numeric payloads as `TRUE`.
7. XLOOKUP reference-return behavior is empirically pinned in both run labels:
   - `CELL("address", XLOOKUP(...))` preserves address identity (`$C$1`, `$B$2`),
   - `SUM(XLOOKUP(...):XLOOKUP(...))` confirms returned references compose through `:` ranges.
8. lookup-family XLL bridge parity now has a dedicated manifest:
   - array-constant `MATCH`, `XMATCH`, and `XLOOKUP` rows matched built-in Excel in `LOOKUP_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv`,
   - reference-range `XLOOKUP` rows are carried as explicit `known_divergence_not_equal` seam-limit cases rather than semantic mismatches.
9. `NOW` is now evidenced across three current-phase seams:
   - provider/recalc baseline in W10,
   - ordinary user-facing volatile XLL registration behavior in W11,
   - caller-cell format-hinting in `TIME_FORMAT_HINT_EXECUTION_RECORD.md`.
