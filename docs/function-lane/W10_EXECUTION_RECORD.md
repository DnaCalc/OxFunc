# W10 Ten-Function Mixed Seams Execution Record

Status: `reopened`
Workset: `W10`
Evidence ID: `W10-TENMIX-SEED-20260308`
Follow-Up Evidence IDs:
1. `W10-LOOKUP-XLL-20260310`
2. `W10-CLOSEOUT-20260311`

## 1. Purpose
Track W10 execution status across runtime, formalization, empirical replay, and function-phase closure for the ten-function packet:
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

## 2. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - lookup-selection array-valued `lookup_value` lifting reopened on
     2026-04-08 for `MATCH`; the `MATCH` correction is landed on
     `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`. Fresh 2026-04-29 replay
     pinned the adjacent `XLOOKUP` multi-needle return shape, and the local
     `XLOOKUP` correction is validated in the working tree pending landed-ref
     promotion under `BUG-FUNC-006` / `W079`.
   - external XLL verification-seam limits remain and must not be mistaken for function-semantic gaps.
6. function-phase-complete slices within W10:
   - `SUM`
   - `IF`
   - `INDEX`
   - `ISNUMBER`
   - `NOW`
   - `INDIRECT`
   - `SEQUENCE`
   - `OP_ADD`

## 3. Executed Scope
Execution dates:
1. `2026-03-09`
2. `2026-03-10` (function-phase-complete promotion follow-up)
3. `2026-03-10` (lookup-family closure follow-up)
4. `2026-03-11` (SUM aggregate argument-structure closure follow-up)
5. `2026-03-11` (`INDEX` / `INDIRECT` / `SEQUENCE` closeout follow-up)

Landed artifacts:
1. function contracts for all ten W10 functions in `docs/function-lane/FUNCTION_SLICE_*_CONTRACT_PRELIM.md`
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
5. replay tooling:
   - `tools/w10-probe/run-w10-suite.ps1`
   - `tools/w10-probe/analyze-w10-results.ps1`
   - `tools/w10-probe/new-w10-compat-template.ps1`
   - `tools/xll-addin/run-lookup-xll-bridge-suite.ps1`

## 4. Verification Runs
1. `cargo test -p oxfunc_core` -> pass (`217` tests).
2. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml` -> pass.
3. `lake build` -> pass.
4. `powershell -File tools/w10-probe/run-w10-suite.ps1 -OutDir .tmp/w10-closeout` -> pass:
   - rows: `122`
   - matched: `122`
   - mismatched: `0`
   - failed_unexpected: `0`
   - dual_run_satisfied: `true`
5. `powershell -File tools/xll-addin/run-lookup-xll-bridge-suite.ps1 -OutDir .tmp/xll-housekeeping` -> pass:
   - rows: `17`
   - matched: `17`
   - mismatched: `0`
6. `powershell -File tools/function-lane-check/run-correlation-integrity-check.ps1` -> pass.

## 5. Gate Tracking
### G1 - Classification Closure
1. Status: `closed-provisional`.
2. Notes:
   - all ten slices have explicit semantic/profile fields in contract docs and runtime metadata.

### G2 - Runtime/Formal Pairing Closure
1. Status: `closed-provisional`.
2. Notes:
   - each function has Rust and Lean artifacts aligned to the current executable-semantic-model strategy.
   - `INDEX`, `INDIRECT`, and `SEQUENCE` no longer remain at seed-stub depth on the Lean side.

### G3 - Empirical Closure
1. Status: `closed-provisional`.
2. Notes:
   - dual-run replay is green across `122` observed rows (`61` default + `61` compat_template).
   - analyzer gate status: `green`.

### G4 - XLL Export Closure
1. Status: `closed-provisional`.
2. Notes:
   - export declarations remain generated from `xll_export_specs`.
   - W10 functions are included in the generated export set.
   - XLL reference-return and host-surface limitations remain external seam limits, not core function-semantic gaps.

### G5 - Promotion Readiness
1. Status: `closed-provisional`.
2. Notes:
   - all ten W10 functions now satisfy current-phase function closure individually.
   - packet-level integration remains partial only because XLL verification cannot reproduce all worksheet-boundary behavior.

## 6. XLL Verification-Seam Limitations
1. Relevant seam limits remain tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.
2. Material packet-level impacts:
   - lookup-family reference-return address and range-composition lanes are now demonstrated through the XLL bridge for the current manifest scope, but broader bridge coverage remains incomplete outside that scope,
   - registration-flag behavior remains a separate W11 evidence lane,
   - whole-row/whole-column worksheet-boundary effects for reference-returning functions remain host-surface observations rather than XLL parity requirements.

## 7. Key Findings
1. `SUM` required explicit OxFunc-side direct-scalar versus array-like classification so aggregate semantics remain representable without passing raw references into the SUM kernel.
2. `INDEX` required three concrete adjustments for closure:
   - explicit blank `row_num` / `col_num` positions behave as `0`,
   - same-sheet multi-area `area_num` selection happens before row/column slicing,
   - mixed-sheet multi-area references must be rejected rather than silently projected.
3. `INDIRECT` has a sharp omission seam on `a1_style`:
   - omitted second argument defaults to `TRUE`,
   - explicitly blank second argument behaves like `FALSE`.
4. `INDIRECT` now admits A1 cell/area, whole-column, whole-row, and absolute/relative R1C1 lanes for the current baseline.
5. `SEQUENCE` is not shape-only:
   - W10 now models materialized row-major payloads,
   - explicit blank `rows`, `columns`, `start`, and `step` positions all use Excel defaults in the observed baseline.
6. the older lookup-family packet remained green only for the previously
   admitted scalar lookup-value scope:
   - `MATCH` stayed green under the widened packet replay that was actually
     exercised,
   - `XLOOKUP` stayed green under the scalar lookup-value, blank lookup, and
     reference-return lanes that were actually exercised,
   - live Excel COM replay on 2026-04-08 later proved that array-valued
     lookup-value lifting was a missing current-baseline lane for `MATCH`; the
     local correction is now landed. Fresh replay on 2026-04-29 pinned the
     adjacent `XLOOKUP` multi-needle return shape, and the local `XLOOKUP`
     correction is validated pending landed-ref promotion.
7. `NOW` remains evidenced across provider/recalc, registration, and format-hint lanes, with format-hint application assigned to the engine boundary rather than the pure kernel.
