# Function Slice Contract (Prelim) - ROW()

## 1. Slice Identity
1. `function_id`: `FUNC.ROW`
2. `display_name`: `ROW`
3. `owner_lane`: `OxFunc`
4. `status`: `function-phase-complete`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `0`
   - maximum: `1`
2. admission policy:
   - admitted for `ROW()` and `ROW(reference)` in the current slice.
   - non-reference arguments remain admission-invalid at worksheet ingress and are not part of the admitted slice.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `caller_context`
9. `surface_fec_dependency_profile`: `caller_context`
10. `fec_facility_tags`: `cap_reference_resolution`; `cap_caller_context`

## 4. Pre-call Coercion Policy
1. references stay visible to the adapter.
2. omitted-argument evaluation requires caller context.
3. admitted reference arguments are interpreted through A1/area shape rather than through dereferenced values.

## 5. Core Outcome Model
1. `ROW()` with omitted argument returns the caller row index as a scalar number.
2. `ROW(single_cell_reference)` returns the referenced row index as a scalar number.
3. `ROW(area_reference)` returns a vertical array containing the distinct row indices from top to bottom.
4. whole-column references retain full row-span semantics, which can surface `#SPILL!` at worksheet publication when the caller anchor cannot host the result shape.

## 6. Post-call Adaptation Policy
1. scalar cases return a numeric `EvalValue`.
2. area cases return a vertical array payload of row indices.
3. worksheet spill/publication limits remain host-surface behavior above the core function result.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, current host locale.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-038` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. `empirical_anchor`:
   - `W13-NONLOCALE-BL-20260314`
3. policy decision anchors:
   - `docs/function-lane/W13_EXECUTION_RECORD.md`
4. current status rationale:
   - `ROW()` is `function-phase-complete` for the current reference baseline,
   - caller-context omission, scalar reference, and vertical area-spill semantics are now aligned across Rust, Lean, and workbook replay,
   - the notable observed edge is that whole-column references are semantically large arrays and may publish as host `#SPILL!` when anchored away from the worksheet edge.

## 9. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/row_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/Row.lean`
3. packet record: `docs/function-lane/W13_EXECUTION_RECORD.md`
