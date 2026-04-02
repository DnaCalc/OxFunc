# W22 Execution Record

Status: `complete-provisional`
Workset: `W22`
Evidence ID: `W22-CRITERIA-SHAPE-20260318`

## 1. Purpose
Record the focused criteria-family shape-hardening packet extracted from the `W17` residual inventory.

## 2. Scope
1. pin the current-baseline Excel behavior for mismatched criteria/target shapes,
2. implement the observed rule set in the criteria-family kernel,
3. replace the old generic open issue with an explicit split between `AVERAGEIF` and the `*IFS` family.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader locale/version sweeps remain orthogonal validation work.

## 4. Executed Scope
Execution date:
1. `2026-03-18`

Artifacts created or updated:
1. `docs/worksets/W022_CRITERIA_FAMILY_SHAPE_HARDENING.md`
2. `docs/function-lane/W22_CRITERIA_SHAPE_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W22_CRITERIA_RUNTIME_REQUIREMENTS.md`
4. `tools/w22-probe/run-w22-criteria-shape-baseline.ps1`
5. `crates/oxfunc_core/src/functions/criteria_family.rs`
6. `docs/function-lane/W16_BATCH51_CRITERIA_AGGREGATES_NOTES.md`
7. `docs/function-lane/FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md`
8. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
9. `docs/function-lane/W17_DEFERRED_LOW_INTEREST_INVENTORY.csv`
10. `docs/worksets/W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md`

## 5. Empirical Findings
From `.tmp/w22-criteria-shape-results.csv`:
1. `AVERAGEIF(A1:A3,1,B2)` -> `30`
2. `AVERAGEIF(A1:A4,1,B2:B3)` -> `35`
3. `AVERAGEIFS(B2, A1:A3, 1)` -> `#VALUE!`
4. `SUMIFS(B2, A1:A3, 1)` -> `#VALUE!`
5. `COUNTIFS(A1:A3,1,B2,">0")` -> `#VALUE!`
6. `MAXIFS(B2, A1:A3, 1)` -> `#VALUE!`
7. `MINIFS(B2, A1:A3, 1)` -> `#VALUE!`
8. `SUMIFS(B2:B4, A1:A3, 1)` -> `90`

Current-baseline interpretation:
1. `AVERAGEIF` anchors a mismatched `average_range` from its top-left reference over the criteria-range shape.
2. `COUNTIFS`, `SUMIFS`, `AVERAGEIFS`, `MAXIFS`, and `MINIFS` remain exact-shape functions on this baseline.

## 6. Implementation Result
1. `criteria_family.rs` now top-left anchors `AVERAGEIF` when its explicit `average_range` is a parseable A1-style reference and its direct resolved shape differs from the criteria range.
2. The rest of the family continues to require exact shape agreement.
3. Targeted Rust tests now cover both:
   - `AVERAGEIF` anchored mismatch behavior
   - exact-shape rejection for the non-anchoring `*IFS` members
4. Contract/conformance promotion is now explicit through `FUNCTION_SLICE_CRITERIA_FAMILY_CONTRACT_PRELIM.md` and `FDEF-041`.
5. The old generic `W17` criteria-family residual row is now reconciled out of the active deferred inventory.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w22-probe/run-w22-criteria-shape-baseline.ps1`
2. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml criteria_family`
4. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
5. `lake build`

## 8. Standing
1. the old `W16` / `W17` criteria-family open issue was too broad,
2. the current baseline split is now evidenced and implemented,
3. the seven-function `W22` criteria-family slice is now function-phase-complete for the current reference Excel baseline,
4. no criteria-family-specific XLL verification-seam limitation is presently known for the admitted slice beyond the generic bridge limits already tracked centrally,
5. the next criteria-family work should only reopen if a version/locale sweep or a broader range-shape case disproves the current baseline rule.

## 9. Pre-Closure Verification Checklist
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

## 10. Completion Claim Self-Audit
1. Step 1 Scope Re-Read: `pass`
   - all seven in-scope criteria-family functions now share an explicit family contract, exercised runtime implementation, and the narrowed empirical shape rule.
2. Step 2 Gate Criteria Re-Read: `pass`
   - `G1`, `G2`, and `G3` are now all satisfied.
3. Step 3 Silent Scope Reduction Check: `pass`
   - the workset still covers the original seven-function criteria-family shape question and did not silently narrow to only `AVERAGEIF`.
4. Step 4 "Looks Done But Is Not" Pattern Check: `pass`
   - no stubs were promoted as implementation,
   - the closure claim rests on native replay plus targeted Rust tests,
   - the family contract matches the exercised implementation and recorded empirical split,
   - Lean remains explicitly an alignment substrate rather than a fake full semantics proof,
   - no cross-repo handoff was needed because this packet did not change evaluator-facing seam clauses.
5. Step 5 Include Result: `pass`
   - W22 is complete for its declared current-baseline scope.
