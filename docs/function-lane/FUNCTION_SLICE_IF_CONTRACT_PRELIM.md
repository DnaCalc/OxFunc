# Function Slice Contract (Prelim) - IF()

## 1. Slice Identity
1. `function_id`: `FUNC.IF`
2. `display_name`: `IF`
3. `owner_lane`: `OxFunc`
4. `status`: `in_progress` (W111 expanded typed scope)

## 2. Signature and Admission Contract
1. arity:
   - minimum: `2`
   - maximum: `3`
2. admission policy:
   - admitted for `IF(condition, then_value[, else_value])`.

## 3. Semantic Class Axes
1. `determinism_class`: `deterministic`
2. `volatility_class`: `nonvolatile`
3. `host_interaction_class`: `none`
4. `thread_safety_class`: `safe_pure`
5. `arg_preparation_profile`: `refs_visible_in_adapter`
6. `coercion_lift_profile`: `custom`
7. `kernel_signature_class`: `custom`
8. `function_adapter_fec_dependency_profile`: `none`
9. `surface_fec_dependency_profile`: `ref_only`

## 4. Pre-call Coercion Policy
1. references stay visible to the adapter so branch preparation can remain selective.
2. the condition argument is prepared first and coerced through the function-local truthiness policy.
3. scalar conditions prepare the selected branch only. Array conditions preserve
   shape information from both branches; this prepared-value rule does not
   establish evaluator expression scheduling, effects or reference-valued selection.
4. condition text accepts only ASCII-case-insensitive TRUE/FALSE; numeric text
   and whitespace-decorated logical text return `#VALUE!`, including references.

## 5. Core Outcome Model
1. when the condition coerces true, the prepared `then_value` is returned.
2. when the condition coerces false, the prepared `else_value` is returned.
3. if `else_value` is omitted, the false branch defaults to logical `FALSE`.
4. non-selected branch values/errors are masked per result cell. For array
   conditions, dimensions are the coordinatewise maxima of condition and both
   branches, including an unused branch. Singleton axes broadcast; absent
   coordinates of other axes yield `#N/A` only when selected.
5. selected blank cells and explicit missing arguments publish numeric zero.
   An omitted third argument remains distinct from an explicitly missing slot.

## 6. Post-call Adaptation Policy
1. successful evaluation returns the selected scalar, text, logical, or error payload directly as `EvalValue`.
2. condition coercion failures map to worksheet-visible `#VALUE!` unless a worksheet error code is already carried through preparation/coercion.

## 7. Version Scope (Required Axes)
1. Excel application version/channel scope:
   - bounded local empirical baseline: Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`.
2. Workbook Compatibility Version scope:
   - bounded dual-run workbook lanes: `default` and `compat_template`.
   - `compat_template` is the `.xls` compatibility template emitted by `tools/w10-probe/new-w10-compat-template.ps1`.

## 8. Evidence Posture
1. `spec_anchor`:
   - packet conformance row `FDEF-035` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
   - public reference ids linked there: `XLS-CF-FN-001`, `XLS-CF-FN-002`, `XLS-CF-FN-007`, `XLS-CF-TV-007`, `XLS-CF-TV-008`
2. `empirical_anchor`:
   - `W10-TENMIX-SEED-20260308`
3. policy decision anchors:
   - `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 2)
   - `docs/function-lane/W10_EXECUTION_RECORD.md`
4. current status rationale:
   - W111 withdraws the earlier phase claim for the expanded typed scope,
   - 812 discovery/control observations exercise prepared-value selection;
     the frozen independent packet exposed direct one-cell array shape failures,
   - evaluator scheduling, reference-returning expressions and HO-FN-023
     acknowledgment/integration remain open.

## 9. W10 Coverage
1. condition coercion is explicit and deterministic.
2. the historical scalar adapter selected one branch; this does not prove array expression scheduling or reference-returning behavior.
3. missing `else` defaults to logical false in the current reference baseline.

## 10. Artifact Bindings
1. Rust: `crates/oxfunc_core/src/functions/if_fn.rs`
2. Lean: `formal/lean/OxFunc/Functions/IfFn.lean`
3. side-note linkage: `docs/function-lane/W10_PROFILE_SYSTEM_SIDE_NOTES.md` (note 2)


## 11. W111 reference and evidence

Excel 16.0 build 20430, 64-bit, Workbook Compatibility Version 2; channel unverified.
See `evidence/w111-broad-20260929/conditional/` and HO-FN-023. The first independent
660-case packet has 496 exact cases, 24 valid direct one-cell array failures, and
140 malformed local array inputs retained but withheld. Refinement remains active.
`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: the shape refinement and fresh
replay; expression scheduling and effects; reference-returning selection;
uncommon host/reference forms; receiving acknowledgment and integration.
