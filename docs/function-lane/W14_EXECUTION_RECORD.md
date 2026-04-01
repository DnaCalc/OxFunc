# W14 Execution Record - Implicit Intersection Operator

Status: `complete`
Workset: `W014`
Evidence ID:
1. `W14-IMPLICIT-INTERSECTION-BL-20260401`

## 1. Purpose
Characterize and implement the admitted current-baseline OxFunc-side semantics for `OP_IMPLICIT_INTERSECTION` / `@`.

## 2. Packet Outputs
Artifacts produced or updated in this packet:
1. `docs/worksets/W014_IMPLICIT_INTERSECTION_OPERATOR.md`
2. `docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md`
3. `docs/function-lane/FUNCTION_SLICE_OP_IMPLICIT_INTERSECTION_CONTRACT_PRELIM.md`
4. `docs/function-lane/W14_IMPLICIT_INTERSECTION_SCENARIO_MANIFEST_SEED.csv`
5. `docs/function-lane/W14_EXECUTION_RECORD.md`
6. `tools/w14-probe/run-w14-implicit-intersection-baseline.ps1`
7. `.tmp/w14-implicit-intersection-results.csv`
8. `crates/oxfunc_core/src/functions/op_implicit_intersection.rs`
9. `formal/lean/OxFunc/Functions/ImplicitIntersection.lean`

## 3. Current Result
The admitted current-baseline OxFunc-side slice is now explicit and executable:
1. scalar operands pass through unchanged
2. single-column references select the caller-row item
3. single-row references select the caller-column item
4. array payloads scalarize to the top-left item
5. spill-anchor references scalarize from the resolved spill payload
6. current 2-D range lane returns `#VALUE!`
7. `.Formula` normalization strips explicit `@` while `.Formula2` preserves it on the seeded lanes
8. `_xlfn.SINGLE(...)` compatibility syntax evaluates identically on the current baseline and normalizes back to explicit `@` in `.Formula2`

## 4. Main Findings
1. Native Excel now pins the previously open 2-D seed lane:
   - `=@A1:B2` at `C3` returns `#VALUE!` on the current baseline
2. Native Excel also pins the stored-form split:
   - `.Formula` normalizes away explicit `@`
   - `.Formula2` preserves explicit `@`
3. Native Excel accepts `_xlfn.SINGLE(A1:A3)` on the current baseline, evaluates it to the same scalar result, stores `.Formula` as `=A1:A3`, and stores `.Formula2` as `=@A1:A3`.
4. The admitted runtime slice can be modeled honestly without preserving a dedicated spill-provenance bit in OxFunc, as long as spill anchors are resolved upstream into the current spill payload or error.
5. `@` needs refs-visible preparation and caller context; values-only pre-adapter preparation remains the wrong seam.
6. Current OxFml semantic-plan and evaluator tests confirm that legacy-single compatibility remains an upstream concern and does not require a second OxFunc operator implementation.

## 5. Verification Basis
Primary verification surfaces:
1. native Excel replay:
   - `tools/w14-probe/run-w14-implicit-intersection-baseline.ps1`
   - `.tmp/w14-implicit-intersection-results.csv`
2. Rust runtime:
   - `crates/oxfunc_core/src/functions/op_implicit_intersection.rs`
3. Lean substrate:
   - `formal/lean/OxFunc/Functions/ImplicitIntersection.lean`
4. OxFml compatibility/evaluation coverage:
   - `..\OxFml\crates\oxfml_core\tests\evaluator_tests.rs`
   - `..\OxFml\crates\oxfml_core\tests\semantic_plan_tests.rs`

## 6. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared W14 current-phase scope

## 7. Current Completion Reading
1. the admitted current-baseline OxFunc-side `@` slice is no longer a missing runtime lane:
   - Rust runtime exists,
   - Lean binding exists,
   - native replay exists,
   - the current host baseline now pins `_xlfn.SINGLE(...)` normalization,
   - and the OxFml adapter/semantic-plan/evaluator lanes now exercise the seeded `@` and legacy-single rows end-to-end.
2. `W014` is therefore complete for declared current-phase OxFunc scope.
3. broader pre-dynamic-array compatibility-version sweep and structured-reference/table-context work remain orthogonal future validation/interop lanes rather than blockers to the current supported claim.
