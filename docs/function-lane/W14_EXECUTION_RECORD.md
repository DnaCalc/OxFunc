# W14 Execution Record - Implicit Intersection Operator

Status: `in_progress`
Workset: `W014`

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

## 4. Main Findings
1. Native Excel now pins the previously open 2-D seed lane:
   - `=@A1:B2` at `C3` returns `#VALUE!` on the current baseline
2. Native Excel also pins the stored-form split:
   - `.Formula` normalizes away explicit `@`
   - `.Formula2` preserves explicit `@`
3. The admitted runtime slice can be modeled honestly without preserving a dedicated spill-provenance bit in OxFunc, as long as spill anchors are resolved upstream into the current spill payload or error.
4. `@` needs refs-visible preparation and caller context; values-only pre-adapter preparation remains the wrong seam.

## 5. Verification Basis
Primary verification surfaces:
1. native Excel replay:
   - `tools/w14-probe/run-w14-implicit-intersection-baseline.ps1`
   - `.tmp/w14-implicit-intersection-results.csv`
2. Rust runtime:
   - `crates/oxfunc_core/src/functions/op_implicit_intersection.rs`
3. Lean substrate:
   - `formal/lean/OxFunc/Functions/ImplicitIntersection.lean`

## 6. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - structured-reference/table-context interaction remains outside the admitted slice
   - compatibility-version and `_xlfn.SINGLE(...)` serialization behavior are still only partially pinned
   - OxFml/FEC-side acknowledgment of the scalarization seam remains pending
