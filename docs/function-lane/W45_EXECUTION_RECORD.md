# W45 Execution Record - Non-@ Operator Universe Closure Pass

Status: `complete`
Workset: `W45`
Evidence IDs:
1. `W45-OP-ARITH-WAVEA-20260320`
2. `W45-OP-CMP-WAVEB-20260320`
3. `W45-OP-REF-WAVEC-20260320`

## 1. Purpose
1. take ownership of the non-`@` operator universe as one named packet,
2. turn the old undeclared operator backlog into explicit slices,
3. close the packet with runtime, formal, export, and native Excel evidence for every row.

## 2. Scope
Artifacts created or updated in the packet:
1. `docs/worksets/W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`
2. `docs/function-lane/W45_NON_AT_OPERATOR_INVENTORY.csv`
3. `docs/function-lane/W45_SCOPE_RECONCILIATION.csv`
4. `docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`
7. `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`
8. `docs/function-lane/W45_WAVEA_RUNTIME_REQUIREMENTS.md`
9. `docs/function-lane/W45_WAVEB_OPERATOR_COMPARE_CONCAT_SCENARIO_MANIFEST_SEED.csv`
10. `docs/function-lane/W45_WAVEB_RUNTIME_REQUIREMENTS.md`
11. `docs/function-lane/W45_WAVEC_OPERATOR_REFERENCE_SCENARIO_MANIFEST_SEED.csv`
12. `docs/function-lane/W45_WAVEC_RUNTIME_REQUIREMENTS.md`
13. `docs/function-lane/W45_EXECUTION_RECORD.md`
14. `tools/w45-probe/run-w45-wavea-operator-arithmetic-baseline.ps1`
15. `tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
16. `tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`
17. `.tmp/w45-wavea-operator-arithmetic-results.csv`
18. `.tmp/w45-waveb-operator-compare-concat-results.csv`
19. `.tmp/w45-wavec-operator-reference-results.csv`
20. `crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
21. `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
22. `crates/oxfunc_core/src/functions/operator_reference_family.rs`
23. `crates/oxfunc_core/src/functions/mod.rs`
24. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
25. `crates/oxfunc_core/src/xll_export_specs.rs`
26. `tools/xll-addin/oxfunc_xll/export_specs.csv`
27. `formal/lean/OxFunc/Functions/OperatorArithmeticFamily.lean`
28. `formal/lean/OxFunc/Functions/OperatorCompareConcatFamily.lean`
29. `formal/lean/OxFunc/Functions/OperatorReferenceFamily.lean`
30. `formal/lean/OxFunc.lean`
31. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W45` scope after reconciliation

## 4. Packet Result
1. Wave A now covers:
   - `OP_UNARY_PLUS`
   - `OP_NEGATE`
   - `OP_ADD`
   - `OP_SUBTRACT`
   - `OP_MULTIPLY`
   - `OP_DIVIDE`
   - `OP_POWER`
   - `OP_PERCENT`
2. Wave B now covers:
   - `OP_CONCAT`
   - `OP_EQUAL`
   - `OP_NOT_EQUAL`
   - `OP_LESS_THAN`
   - `OP_LESS_EQUAL`
   - `OP_GREATER_THAN`
   - `OP_GREATER_EQUAL`
3. Wave C now covers:
   - `OP_RANGE_REF`
   - `OP_INTERSECTION_REF`
   - `OP_UNION_REF`
   - `OP_SPILL_REF`
   - `OP_TRIM_REF_LEADING`
   - `OP_TRIM_REF_TRAILING`
   - `OP_TRIM_REF_BOTH`
4. the library-context snapshot export now exposes the full current non-`@` operator surface to OxFml.

## 5. Main Findings
1. arithmetic wave:
   - unary plus accepts numeric text
   - unary negate coerces `TRUE` to `1` before negation
   - postfix percent scales by `1/100`
   - divide-by-zero surfaces `#DIV/0!`
   - `(-1)^0.5` surfaces `#NUM!`
2. compare/concat wave:
   - `="a"&1 -> "a1"`
   - `=1&TRUE -> "1TRUE"`
   - text equality is case-insensitive on the admitted slice
   - direct `1="1"` is `FALSE`; the operator does not numeric-coerce text on the admitted scalar slice
   - mixed-type ordering is now pinned for the seeded slice
   - blank cells compare as `0`, `""`, or `FALSE` depending on counterpart type
3. reference wave:
   - range operator preserves the rectangular span and normalizes reversed bounds
   - intersection returns the overlap area and surfaces `#NULL!` when empty
   - union forms the same parenthesized multi-area shape already consumed by `INDEX`
   - trim-ref family is now treated as structural reference-target normalization with native transparency evidence

## 6. Verification Runs
1. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml operator_arithmetic_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml operator_compare_concat_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml operator_reference_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
6. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
7. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
8. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavea-operator-arithmetic-baseline.ps1`
9. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
10. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`
11. `lake build`
12. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

## 7. Standing
1. `W45` is no longer a planning-only packet.
2. all `22` non-`@` operator rows are now reconciled as `done` in `W45`.
3. legacy CSE array-formula context remains an orthogonal seam topic tracked outside `W45`; it does not qualify the declared packet closure.
