# Function Slice Contract (Prelim) - SEQUENCE()

## 1. Slice Identity
1. `function_id`: `FUNC.SEQUENCE`
2. `display_name`: `SEQUENCE`
3. `owner_lane`: `OxFunc`
4. `status`: `provisional`

## 2. Signature and Admission Contract
1. arity:
   - minimum: `1`
   - maximum: `4`
2. admission policy:
   - admitted for `SEQUENCE(rows[, columns[, start[, step]]])`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `values_only_pre_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. dimension and scalar parameters are prepared as values before adapter evaluation.
2. explicit blank/missing defaults match the current empirical baseline:
   - `rows` defaults to `1`,
   - `columns` defaults to `1`,
   - `start` defaults to `1`,
   - `step` defaults to `1`.
3. numeric text is admitted for dimension arguments.
4. non-integral, negative, zero, or non-finite dimensions are rejected.

## 5. Core Outcome Model
1. `SEQUENCE` materializes a full row-major array payload.
2. the first cell is `start`.
3. each subsequent cell increments by `step` in row-major order across the spill shape.
4. explicit omitted middle arguments use defaults without changing later provided arguments.

## 6. Post-call Adaptation Policy
1. successful evaluation returns `EvalValue::Array` with materialized payload cells.
2. zero dimensions surface to worksheet `#CALC!`.
3. other dimension or scalar coercion failures surface to worksheet `#VALUE!`, except carried worksheet errors which propagate directly.

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
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
4. current status rationale:
   - omitted `rows`, `columns`, `start`, and `step` lanes are replayed and test-backed,
   - the Rust and Lean implementations now both model materialized payload arrays rather than shape-only placeholders,
   - no current-phase function-semantic gap remains for the declared reference-baseline slice.

## 9. W10 Coverage
1. baseline `SEQUENCE(2,3)` payload materialization.
2. zero-dimension `#CALC!` lane.
3. numeric-text dimension coercion.
4. explicit omission defaults for `rows`, `columns`, `start`, and `step`.
5. negative-step payload materialization.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/sequence.rs`
2. Lean: `formal/lean/OxFunc/Functions/Sequence.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 5)
