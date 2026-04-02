# W52 Execution Record

Status: `complete-provisional`
Workset: `W52`
Evidence ID: `W52-SUMIF-BL-20260326`

## 1. Purpose
Record the standalone `SUMIF` closure packet extracted from the missing criteria-family member exposed by the OxFml adapter corpus.

## 2. Scope
1. pin the current-baseline Excel behavior for `SUMIF`-specific target-range lanes,
2. align the shared criteria-family runtime and Lean substrate to include `SUMIF`,
3. promote the contract/evidence/export artifacts so `SUMIF` is no longer catalog-only.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader locale/version sweeps remain orthogonal validation work.

## 4. Executed Scope
Execution date:
1. `2026-03-26`

Artifacts created or updated:
1. `docs/HISTORY.md`
2. `docs/function-lane/W52_SUMIF_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W52_SUMIF_RUNTIME_REQUIREMENTS.md`
4. `tools/w52-probe/run-w52-sumif-baseline.ps1`
5. `docs/function-lane/W52_EXECUTION_RECORD.md`
6. `crates/oxfunc_core/src/functions/criteria_family.rs`
7. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
8. `crates/oxfunc_core/src/xll_export_specs.rs`
9. `formal/lean/OxFunc/Functions/CriteriaFamily.lean`
10. `docs/function-lane/FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md`
11. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
12. `docs/function-lane/FUNCTION_SLICE_CORRELATION_LEDGER.csv`
13. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
14. `tools/xll-addin/oxfunc_xll/export_specs.csv`
15. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 5. Empirical Findings
From `.tmp/w52-sumif-results.csv`:
1. `SUMIF(A1:A3,">1")` -> `5`
2. `SUMIF(A1:A3,1,B2)` -> `90`
3. `SUMIF(A1:A4,1,B2:B3)` -> `140`
4. `SUMIF(A1:A3,1,B1:B3)` -> `10`
5. `SUMIF(A1:A3,2,B1:B3)` -> `#DIV/0!`

Current-baseline interpretation:
1. omitted `sum_range` uses the criteria range directly,
2. an explicit mismatched A1-style `sum_range` anchors from its top-left reference over the criteria-range shape,
3. target aggregation is numeric-only,
4. a reached target-side worksheet error propagates.

## 6. Implementation Result
1. `criteria_family.rs` now exposes `SUMIF` metadata and runtime evaluation on the shared criteria-family substrate.
2. `SUMIF` follows the single-criteria target-range rule rather than the exact-shape `*IFS` rule:
   - omitted `sum_range` uses the criteria range,
   - an explicit mismatched A1-style `sum_range` anchors from the referenced top-left cell.
3. Targeted Rust tests now cover:
   - omitted `sum_range`,
   - anchored mismatched `sum_range`,
   - shared family metadata shape.
4. The shared Lean criteria-family metadata/alignment now includes `SUMIF`.
5. Generated export and snapshot artifacts now expose `SUMIF` as a real admitted function instead of a catalog-only gap.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w52-probe/run-w52-sumif-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml criteria_family -- --nocapture`
3. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
4. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`
5. `cargo test --manifest-path ..\OxFml\crates\oxfml_core\Cargo.toml --test oxfunc_catalog_snapshot_export_tests -- --nocapture`
6. `lake build`

## 8. Cross-Repo Impact Assessment
1. reviewed inbound observations: `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
2. `SUMIF` does not introduce a new evaluator-facing clause shape:
   - the first argument remains `ReferenceVisible` at adapter entry under the existing criteria-family refs-visible admission rule,
   - the new work only closes a missing OxFunc runtime/catalog/export hole behind that already-admitted seam.
3. no new cross-repo handoff packet is required for this packet.

## 9. Standing
1. the old state was not an honest implementation because `SUMIF` existed in the catalog and snapshot but not in the runtime/export/formal artifact chain,
2. `SUMIF` is now function-phase-complete for the current reference Excel baseline on the shared criteria-family substrate,
3. no known `SUMIF` semantic gap remains in the admitted current-baseline slice,
4. no criteria-family-specific XLL verification-seam limitation is presently known for `SUMIF` beyond the generic bridge limits already tracked centrally.

## 10. Pre-Closure Verification Checklist
1. Function contract rows complete and promoted for all in-scope functions? `yes`
2. Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? `yes`
3. Rust implementation and required tests pass for all in-scope functions? `yes`
4. At least one deterministic replay artifact exists per in-scope function behavior? `yes`
5. Evidence links complete and reproducible? `yes`
6. Version scope explicit on both axes (Excel app version/channel + workbook Compatibility Version)? `yes`
7. Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? `yes`
8. XLL verification-seam limitations documented in seam-level and function-level records where material? `yes`
9. Cross-repo impact assessed and handoff filed if FEC/F3E boundary or evaluator-facing clauses affected? `yes`
10. No known semantic gap remains in declared scope? `yes`
11. Completion language audit passed (no premature "done"/"complete" per AGENTS.md anti-premature-completion rules)? `yes`
12. `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? `yes`
13. blocker provenance updated where relevant? `yes`

## 11. Completion Claim Self-Audit
1. Step 1 Scope Re-Read: `pass`
   - the packet still covers the declared `SUMIF` runtime/formal/evidence/export closure and did not silently narrow to metadata-only work.
2. Step 2 Gate Criteria Re-Read: `pass`
   - `G1`, `G2`, and `G3` are all satisfied.
3. Step 3 Silent Scope Reduction Check: `pass`
   - the packet did not borrow W22 closure language or silently pretend that prior seven-function criteria-family evidence already covered `SUMIF`.
4. Step 4 "Looks Done But Is Not" Pattern Check: `pass`
   - no scaffolding is being reported as implementation,
   - native Excel replay exists for the `SUMIF`-specific lanes,
   - the contract, Lean metadata, Rust runtime, and published export artifacts are now aligned,
   - no unacknowledged cross-repo handoff is being used as fake closure.
5. Step 5 Include Result: `pass`
   - `W52` is complete for its declared current-baseline scope.
