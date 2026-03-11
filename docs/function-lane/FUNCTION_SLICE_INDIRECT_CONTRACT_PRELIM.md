# Function Slice Contract (Prelim) - INDIRECT()

## 1. Slice Identity
1. `function_id`: `FUNC.INDIRECT`
2. `display_name`: `INDIRECT`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `2`
2. admission policy:
   - admitted for `INDIRECT(ref_text[, a1_style])`.
   - `ref_text` is prepared as a scalar value before adapter evaluation.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `volatile_contextual`
3. `host_interaction_class`: `workbook_state`
4. `thread_safety_class`: `host_serialized`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `caller_context`
9. `surface_fec_dependency_profile`: `caller_context`

## 4. Pre-call Coercion Policy
1. `ref_text` must coerce to nonempty text.
2. omitted `a1_style` defaults to `TRUE`.
3. explicit blank/missing `a1_style` behaves like `FALSE` in the current empirical baseline.
4. numeric coercion for nonblank `a1_style` follows worksheet boolean-as-number convention:
   - zero => `FALSE`,
   - nonzero => `TRUE`.

## 5. Core Outcome Model
1. `a1_style = TRUE` admits A1-style cell, area, whole-column, and whole-row text references.
2. `a1_style = FALSE` admits R1C1 text references.
3. absolute R1C1 references do not require caller context.
4. relative or mixed relative R1C1 references require caller context and resolve against caller row/column coordinates.
5. successful evaluation returns reference identity rather than dereferenced values.
6. whole-row/whole-column references are preserved as area-form references at the OxFunc boundary.

## 6. Post-call Adaptation Policy
1. successful evaluation returns `EvalValue::Reference`.
2. invalid reference text surfaces to worksheet `#REF!`.
3. coercion failures in `ref_text` or `a1_style` surface to worksheet `#VALUE!`, except carried worksheet errors which propagate directly.
4. worksheet-boundary behavior for direct-return whole-row/whole-column references may include host-surface spill or implicit-intersection effects; OxFunc models the reference result, while those worksheet display outcomes are recorded as boundary observations.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-035` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`
2. `empirical_anchor`:
   - `W10-TENMIX-SEED-20260308`
3. policy decision anchors:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 6)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`
4. current status rationale:
   - A1 cell/area, whole-column/whole-row, absolute R1C1, relative R1C1, and explicit-blank `a1_style` behavior are replayed and test-backed,
   - OxFunc and Lean both now model the critical caller-context seam and the observed `a1_style` defaulting split between omitted and explicitly blank second-argument lanes,
   - no current-phase function-semantic gap remains for the declared reference-baseline slice.

## 9. W10 Coverage
1. A1 cell baseline.
2. A1 area consumed through `SUM`.
3. whole-column A1 text consumed through `SUM`.
4. direct-return whole-row host-surface spill observation.
5. absolute and relative R1C1.
6. explicit blank `a1_style` as `FALSE`.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/indirect.rs`
2. Lean: `formal/lean/OxFunc/Functions/Indirect.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (notes 3 and 6)
