# W10 Ten-Function Mixed Seams Execution Record

Status: `in_progress-provisional`
Workset: `W10`
Evidence ID: `W10-TENMIX-SEED-20260308`

## 1. Purpose
Track W10 execution status across runtime/formal scaffolding, scenario manifests, and classification side-notes.

## 2. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - `SUM` still lacks full direct-vs-range provenance semantics.
   - `INDEX` still lacks full array-payload extraction and nontrivial `area_num` semantics.
   - `MATCH` still rejects non-exact modes that Excel supports.
   - `XLOOKUP` still omits broader match/search-mode parity beyond the current seed lanes.
   - `INDIRECT` remains incomplete across R1C1 and broader ref-text semantics.
   - `SEQUENCE` remains shape-only and does not materialize payload values.
   - XLL verification-seam limits remain external and must not be mistaken for function-semantic closure.

## 3. Executed Scope
Execution date:
1. `2026-03-09`

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

## 5. Verification Runs
1. `cargo test -p oxfunc_core` -> pass (`120` tests).
2. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml` -> pass.
3. `./tools/xll-addin/sync-export-specs.ps1` -> pass.
4. `lake build` (from `formal/lean`) -> pass.

## 5. Gate Tracking
### G1 - Classification Closure
1. Status: `closed-provisional`.
2. Notes:
   - all ten slices have explicit class axes and profile rows in contract docs.

### G2 - Runtime/Formal Pairing Closure
1. Status: `closed-provisional`.
2. Notes:
   - each function now has a Rust module and Lean module.
   - multiple functions remain semantically incomplete (`SUM`, `INDEX`, `MATCH`, `XLOOKUP`, `INDIRECT`, `SEQUENCE`) and are still scaffolding rather than closed implementations.

### G3 - Empirical Closure
1. Status: `closed-provisional`.
2. Notes:
   - default + `compat_template` Excel replays executed on `2026-03-09`.
   - rows: `48` (`24` default + `24` compat_template).
   - expectation matched: `48`; mismatched: `0`.
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
   - W10 does not satisfy implementation closure because several functions still carry known Excel-semantic gaps.

## 6. XLL Verification-Seam Limitations
1. Relevant seam limits for the W10 packet are tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.
2. Material packet-level impacts:
   - reference-return and non-scalar payload lanes are not fully demonstrated through the XLL bridge,
   - registration-flag behavior remains outside ordinary profile-derived export generation,
   - XLL evidence must not be used to justify semantic closure for `XLOOKUP`, `INDIRECT`, or `SEQUENCE`.

## 7. Key Findings
1. strict runtime split remains workable for most non-interesting function shapes.
2. reference-return functions require explicit return policy axes and cannot be fully captured by values-only preparation.
3. volatile/provider seams are now visible as a first-class profile pressure point (`NOW`).
4. array payload modeling is the primary blocker for deeper `SEQUENCE`/`INDEX` closure.
5. XLL codegen is now profile-derived over the catalog, removing per-function export-row hand curation.
6. empirical corrections from replay:
   - `MATCH(2,{1,2,3},1)` returns `2` (not `#N/A`).
   - `SEQUENCE(0)` returns `#CALC!` (not `#VALUE!`).
   - `INDIRECT` with `a1_style=FALSE` follows R1C1 addressing when expression is valid (for example `R1C2`).
   - `XLOOKUP` return-array orientation/alignment matters; mismatched orientation yields `#VALUE!`.
7. XLOOKUP reference-return behavior is empirically pinned in both run labels:
   - `CELL("address", XLOOKUP(...))` preserves address identity (`$C$1`, `$B$2`),
   - `SUM(XLOOKUP(...):XLOOKUP(...))` confirms returned references compose through `:` ranges.
