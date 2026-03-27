# W38 Execution Record - Functional Lambda And Helper Family

Status: `in_progress`
Workset: `W38`
Evidence IDs:
1. `W38-LAMBDA-HELPER-STAGE1-20260319`
2. `W38-MAP-REDUCE-SCAN-STAGE2-20260319`
3. `W38-BYROW-BYCOL-MAKEARRAY-DEFINED-NAMES-STAGE3-20260319`

## 1. Purpose
1. open `W38` with the first admitted current-baseline worksheet slice for the helper/callable family:
   - `LET`,
   - immediate-invoked `LAMBDA`,
   - the admitted direct `ISOMITTED` lanes.

## 2. Scope
Artifacts created or updated:
1. `docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md`
2. `docs/function-lane/FUNCTION_SLICE_FUNCTIONAL_LAMBDA_AND_HELPER_STAGE1_CONTRACT_PRELIM.md`
3. `docs/function-lane/W38_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W38_RUNTIME_REQUIREMENTS.md`
5. `docs/function-lane/W38_EXECUTION_RECORD.md`
6. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
7. `tools/w38-probe/run-w38-lambda-helper-stage1-baseline.ps1`
8. `docs/function-lane/FUNCTION_SLICE_FUNCTIONAL_LAMBDA_HELPERS_STAGE2_MAP_REDUCE_SCAN_CONTRACT_PRELIM.md`
9. `docs/function-lane/W38_STAGE2_MAP_REDUCE_SCAN_SCENARIO_MANIFEST_SEED.csv`
10. `docs/function-lane/W38_STAGE2_RUNTIME_REQUIREMENTS.md`
11. `tools/w38-probe/run-w38-map-reduce-scan-stage2-baseline.ps1`
12. `docs/function-lane/FUNCTION_SLICE_FUNCTIONAL_LAMBDA_HELPERS_STAGE3_BYROW_BYCOL_MAKEARRAY_DEFINED_NAMES_CONTRACT_PRELIM.md`
13. `docs/function-lane/W38_STAGE3_BYROW_BYCOL_MAKEARRAY_DEFINED_NAMES_SCENARIO_MANIFEST_SEED.csv`
14. `docs/function-lane/W38_STAGE3_RUNTIME_REQUIREMENTS.md`
15. `tools/w38-probe/run-w38-stage3-byrow-bycol-makearray-defined-names-baseline.ps1`
16. `tools/w38-probe/run-w38-suite.ps1`
17. `crates/oxfunc_core/src/functions/callable_helpers.rs`
18. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
19. `crates/oxfunc_core/src/xll_export_specs.rs`
20. `tools/xll-addin/oxfunc_xll/export_specs.csv`
21. `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
22. `formal/lean/OxFunc/Functions/FunctionalLambdaHelpers.lean`
23. `formal/lean/OxFunc.lean`
24. `crates/oxfunc_core/src/functions/callable_stage1_prepared.rs`

## 3. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - typed callable metadata now exists in OxFunc core, and executable callable invocation plus worksheet-surface evaluators now exist for admitted direct-invocation, `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` lanes
   - helper-family worksheet surfaces are now wired into core dispatch/export/catalog admission for the admitted higher-order slice when a prepared callable value is supplied
   - direct Stage 1 runtime semantics for `LET`, immediate `LAMBDA`, and the currently observed direct `ISOMITTED` lanes now exist in core as a prepared-expression substrate, and the OxFml adapter now exercises the admitted helper slice end-to-end; the remaining residual is narrowed to bind/admission validation edges rather than missing runtime semantics
   - Lean executable substrate alignment now exists for the admitted higher-order helper slice, callable publication, omission, and Defined Name callable preservation seeds, but it does not attempt to duplicate OxFml-owned parser/binder formation behavior
   - final callable carrier remains an open cross-repo seam topic
   - UDF/interoperable callable transport remains out of first-pass scope and deferred

## 4. Empirical Findings
From `.tmp/w38-lambda-helper-stage1-results.csv`:
1. all `18` seeded Stage 1 rows matched expected behavior on the current reference baseline.
2. `LET` supports sequential bindings, nested bindings, and array-valued bindings in the admitted slice.
3. Duplicate local `LET` names are rejected at formula-admission time (`set_err`) rather than producing a later worksheet error.
4. `LAMBDA` supports immediate single- and multi-argument invocation in the admitted slice.
5. A bare uninvoked lambda evaluates to `#CALC!` on the worksheet surface.
6. Direct arity mismatch on lambda invocation returns `#VALUE!`.
7. Duplicate lambda parameter names are rejected at formula-admission time.
8. `LET` bindings are lexically captured by immediately invoked lambdas in the admitted slice.
9. `ISOMITTED` returns `FALSE` for present arguments in the seeded direct lanes.
10. `ISOMITTED` returns `TRUE` for an explicit omitted placeholder in the seeded direct lambda lane `LAMBDA(a,b,ISOMITTED(b))(1,)`.
11. Direct lambda under-application is a different lane and does not expose an omitted-argument channel; the call fails with `#VALUE!` before `ISOMITTED` becomes useful.

From `.tmp/w38-map-reduce-scan-stage2-results.csv`:
11. all `14` seeded Stage 2 rows matched expected behavior on the current reference baseline.
12. `MAP` spills array results on the worksheet surface in the admitted slice.
13. For the seeded mismatched-array lane, `MAP` materializes `#N/A` for the missing partner element instead of failing the whole formula.
14. `REDUCE` folds to a scalar over the admitted array-constant lanes.
15. `SCAN` spills intermediate accumulations, and the visible spill excludes the initial accumulator as a separate leading element.
16. Runtime lambda arity mismatch inside `MAP`, `REDUCE`, and `SCAN` returns `#VALUE!`.
17. Malformed helper lambda declaration with an extra body argument is rejected at formula admission.
19. In the seeded `MAP` and `REDUCE` lanes, present helper arguments are not omitted.

From `.tmp/w38-stage3-byrow-bycol-makearray-defined-names-results.csv`:
20. all `15` seeded Stage 3 rows matched expected behavior on the current reference baseline.
21. `BYROW` returns one scalar result per source row in the admitted slice.
22. `BYCOL` returns one scalar result per source column in the admitted slice.
23. `BYROW` and `BYCOL` return `#CALC!` when the supplied lambda body yields a non-scalar result in the seeded lanes.
24. `BYROW` runtime lambda arity mismatch returns `#VALUE!`.
25. malformed `BYCOL` lambda declaration with an extra body argument is rejected at formula admission.
26. `MAKEARRAY` uses 1-based generated row and column coordinates in the seeded slice.
27. `MAKEARRAY` runtime lambda arity mismatch returns `#VALUE!`.
28. in the seeded `MAKEARRAY` lane, generated coordinate arguments are present rather than omitted.
29. workbook Defined Names preserve callable lambda values in the admitted slice.
30. direct invocation through a Defined Name callable works.
31. `MAP` can invoke a workbook Defined Name callable.
32. lexical capture preserved inside a workbook Defined Name callable remains visible in the seeded slice.
33. bare publication of a workbook Defined Name callable yields `#CALC!`.
34. OxFunc core now carries a typed callable value placeholder instead of a bare lambda string placeholder.
35. That typed carrier currently preserves:
   - callable token
   - origin kind
   - arity shape
   - capture mode
   - invocation-contract reference
36. The typed carrier now has an executable companion substrate in `crates/oxfunc_core/src/functions/callable_helpers.rs`.
37. That substrate currently supports:
   - direct token-driven callable invocation with arity enforcement,
   - token-driven `MAP` over the admitted prepared-array slice,
   - token-driven `REDUCE` over the admitted prepared-array slice,
   - token-driven `SCAN` over the admitted prepared-array slice,
   - token-driven `BYROW` over the admitted prepared-array slice with scalar-result enforcement,
   - token-driven `BYCOL` over the admitted prepared-array slice with scalar-result enforcement,
   - token-driven `MAKEARRAY` over generated 1-based row/column coordinates in the admitted slice,
   - and direct Defined Name callable invocation through the same typed boundary.
38. The new substrate also now has worksheet-surface evaluator functions for the admitted higher-order helper slice when the callable value is already prepared as an `EvalValue::Lambda`.
39. These worksheet-surface evaluators are now wired into core dispatch/export/catalog admission for the admitted higher-order helper slice when a prepared callable value is supplied.
40. The current XLL bridge does not yet carry callable worksheet values or workbook Defined Name callable bindings end-to-end.
41. For `W38`, that is recorded as an external seam limitation rather than a packet-open semantic lane.
42. The native Excel `W38` suite runner now replays all three empirical stages in one step.
43. Lean now has an executable callable/helper substrate in `formal/lean/OxFunc/Functions/FunctionalLambdaHelpers.lean`.
44. That substrate aligns the admitted helper metadata plus seeded executable cases for:
   - `ISOMITTED`,
   - direct callable arity mismatch,
   - `MAP`,
   - `REDUCE`,
   - `SCAN`,
   - `BYROW`,
   - `BYCOL`,
   - `MAKEARRAY`,
   - bare callable publication to `#CALC!`,
   - and workbook Defined Name callable invocation/preservation on the admitted slice.
45. OxFunc core now also has a separate Stage 1 prepared-expression runtime substrate in `crates/oxfunc_core/src/functions/callable_stage1_prepared.rs`.
46. That substrate executes the admitted direct worksheet-semantics slice for:
   - `LET`,
   - immediate `LAMBDA`,
   - direct `ISOMITTED`,
   - lexical capture,
   - duplicate-name rejection,
   - direct arity mismatch,
   - and bare callable publication to `#CALC!`.
47. The current remaining gap for that Stage 1 slice is no longer missing runtime semantics in OxFunc core; it is the remaining bind/admission formation behavior that remains OxFml-owned at the seam.
48. The current OxFml adapter wave now exercises admitted direct/helper/higher-order callable lanes end-to-end, so the remaining seam pressure is narrowed to bind-time validation vocabulary and final callable-carrier tightening.

## 5. Current Packet Result
1. `W38` now has a real Stage 1 native packet and a first honest contract for the helper/callable family.
2. `W38` now also has a Stage 2 higher-order helper packet for `MAP`, `REDUCE`, and `SCAN`.
3. `W38` now also has a Stage 3 packet for `BYROW`, `BYCOL`, `MAKEARRAY`, and workbook Defined Name callable preservation.
4. All nine inventory members now have seeded native baseline evidence on the admitted current-baseline slice.
5. The packet is not function-phase-complete for the nine-member inventory.
6. The current result is a meaningful kickoff that pins:
   - admission-time helper errors,
   - immediate lambda invocation behavior,
   - lexical capture across `LET` and `LAMBDA`,
   - the admitted direct `ISOMITTED` behavior, including explicit omitted-placeholder visibility distinct from arity under-application,
   - first higher-order helper invocation and spill-shape behavior for `MAP`, `REDUCE`, and `SCAN`,
   - row-wise and column-wise scalarity requirements for `BYROW` and `BYCOL`,
   - coordinate-driven helper invocation for `MAKEARRAY`,
   - workbook Defined Name callable preservation on the Excel-supported surface,
   - and the first typed callable-value substrate step inside OxFunc core.
7. OxFunc core now also has worksheet-surface evaluator functions wired through dispatch/export/catalog admission for the admitted token-driven higher-order-helper slice.
8. OxFunc core now also has a direct Stage 1 prepared-expression runtime substrate for `LET`, immediate `LAMBDA`, and the currently observed direct `ISOMITTED` lanes.
9. The remaining worksheet-facing open lane is no longer raw runtime support; it is the remaining bind/admission formation behavior that sits on the OxFml side of the seam.
10. The latest OxFml adapter evidence narrows that remaining seam pressure further: the admitted callable-helper slice is integration-exercised, and the main residual is bind-time validation parity such as duplicate-name rejection.
11. The callable/XLL bridge limitation remains documented, but it is not counted as a `W38` open lane.

## 6. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w38-probe/run-w38-lambda-helper-stage1-baseline.ps1`
2. `powershell -ExecutionPolicy Bypass -File tools/w38-probe/run-w38-map-reduce-scan-stage2-baseline.ps1`
3. `powershell -ExecutionPolicy Bypass -File tools/w38-probe/run-w38-stage3-byrow-bycol-makearray-defined-names-baseline.ps1`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml helper_and_defined_name_lambda_constructors_preserve_metadata`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml callable_arity_shape_exact_accepts_only_matching_arity`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml callable_helpers -- --nocapture`
7. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml eval_surface_value_call_with_callable_supports_map_helper_surface`
8. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml callable_stage1_prepared -- --nocapture`
9. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
10. `powershell -ExecutionPolicy Bypass -File tools/w38-probe/run-w38-suite.ps1 -OutDir .tmp`
11. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
12. `lake build`

## 7. Standing
1. `W38` is underway with a real empirical packet rather than pure seam prose.
2. All nine `W38` inventory members now have seeded native baseline evidence inside `W38`, and OxFunc now has:
   - a typed callable value carrier,
   - executable callable invocation and worksheet-surface dispatch/export admission for the admitted Stage 2 and Stage 3 helper lanes,
   - a direct Stage 1 prepared-expression runtime substrate for `LET`, immediate `LAMBDA`, and direct `ISOMITTED`,
   - and a Lean executable callable/helper substrate for the admitted higher-order helper slice and callable publication/preservation seeds.
3. The packet remains intentionally partial until the remaining bind/admission residuals and callable-carrier seam are tightened across the seam.
4. The callable/XLL bridge limitation is orthogonal and does not by itself keep `W38` open.
5. Workbook Defined Name callable preservation is now empirically and formally characterized as part of first-pass `W38` scope.
