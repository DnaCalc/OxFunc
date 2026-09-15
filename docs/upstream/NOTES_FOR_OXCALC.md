# Notes for OxCalc

Status: `active`
Owner lane: OxFunc
Relationship: current outbound seam-context note from OxFunc to OxCalc

## 1. Purpose

Record OxFunc receiving-side decisions for OxCalc handoffs that affect
function metadata, kernel selector ownership, and prepared-package
invalidation.

## 2. CALC-003 Receiving Decision

OxFunc decision: `accept_metadata_and_semantics_reservation_current_scope_enforcement`.

Accepted:
1. OxFunc owns reduction-sensitive kernel metadata.
2. OxFunc owns error-collapse-sensitive kernel metadata.
3. OxFunc owns exact `NumericalReductionPolicy` semantics.
4. OxFunc owns exact `ErrorAlgebra` / worksheet-error precedence semantics.

Canonical OxFunc receipt:
1. `docs/handoffs/HANDOFF-CALC-003_OXFUNC_RECEIPT.md`

Canonical OxFunc contract:
1. `docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`

Reserved invalidation signal:
1. `semantic_kernel_metadata_version`

Current blocker/open lane:
1. current Rust exercises `NumericalReductionPolicy::SequentialLeftFold`
   through `SUM` prepared aggregate evaluation,
2. current Rust exposes and tests
   `ErrorAlgebra::CanonicalExcelLegacy` as the worksheet-error collapse helper
   boundary,
3. `PairwiseTree`, `KahanCompensated`, replay tree-shape fields, compensation
   state, and broad per-family error-collapse wiring remain successor lanes,
4. OxCalc should treat `semantic_kernel_metadata_version` as the canonical
   prepared-package invalidation signal for selector metadata changes.

## 3. CALC-004 Receiving Decision

OxFunc decision: `accept_identity_reservation_activate_image_producer_facts`.

Accepted:
1. an OxFunc metadata/profile shape equivalent to
   `RichArgAccepted(required_capability_set)`,
2. sparse-reader admission metadata as a successor lane with final Rust naming
   deferred,
3. producer capability publication as typed metadata on producers or returned
   rich/sparse carriers.

Canonical OxFunc receipt:
1. `docs/handoffs/HANDOFF-CALC-004_OXFUNC_RECEIPT.md`

Canonical OxFunc contract:
1. `docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`

Reserved invalidation signal:
1. `arg_admission_metadata_version`

First preferred activation lane:
1. `IMAGE` / `_webimage` producer capability publication.

Deferred lane:
1. sparse range readers for aggregate reducers, after sparse reader API and
   replay semantics are specified.

Current blocker/open lane:
1. current Rust shape is
   `ArgAdmissionMetadata::RichArgAccepted { required_capability_set_keys }`;
   no current `ArgPreparationProfile::RichArgAccepted` variant exists,
2. rich-argument mismatch is deterministic at admission/preparation validation
   when producer capability keys are checked against required keys,
3. no current sparse-reader profile/runtime path is enforced,
4. `IMAGE` registry metadata and
   `eval_image_surface_extended_with_capabilities(...)` currently emit
   `_webimage` producer/exercised capability facts; no generic rich producer
   protocol is claimed.

## 4. W050 Landing Classification

This table is the OxFunc landing-space response for OxCalc W050 closure
review. It separates code/test-backed evidence from successor work and does
not treat metadata reservations as runtime execution claims.

| Item | Classification | OxFunc evidence / successor lane |
| --- | --- | --- |
| `semantic_kernel_metadata_version` publication from real registry metadata | current evidence exists in OxFunc | `RegistryFunctionMeta`; `render_registry_metadata_csv(...)`; `emit_registry_metadata`; tests `reducer_registry_metadata_publishes_semantic_kernel_version`, `semantic_kernel_version_changes_when_selector_metadata_changes`, `registry_metadata_csv_exports_version_and_capability_columns` |
| `arg_admission_metadata_version` publication from real registry metadata | current evidence exists in OxFunc | `RegistryFunctionMeta`; `ArgAdmissionMetadata::version_key()`; `render_registry_metadata_csv(...)`; tests `arg_admission_version_changes_when_admission_metadata_changes`, `registry_metadata_csv_exports_version_and_capability_columns` |
| affected-kernel metadata for reduction-sensitive/error-collapse-sensitive functions | current evidence exists in OxFunc | `semantic_kernel_metadata_for_id(...)`; tests `reducer_registry_metadata_publishes_semantic_kernel_version`, `selector_registry_metadata_publishes_error_algebra_without_reduction_policy` |
| selector enforcement for `NumericalReductionPolicy` | current evidence exists in OxFunc | Current-scope enforcement is `SequentialLeftFold` through `SUM`; tests `eval_sum_exercises_sequential_left_fold_reduction_policy`, `sequential_left_fold_is_order_visible_for_reduction_sensitive_sums`, `non_current_reduction_policies_are_explicitly_deferred`. Successor lane: `W050-SK-NRP-REPLAY`, for `PairwiseTree`, `KahanCompensated`, replay tree-shape, and compensation-state fields |
| selector enforcement for `ErrorAlgebra` | successor work, with proposed workset/lane | Helper boundary exists and is tested by `canonical_excel_legacy_error_algebra_collapses_by_precedence`, but broad function-family runtime enforcement is successor lane `W050-SK-ERROR-ALGEBRA-FAMILY-WIRING` |
| `RichArgAccepted(required_capability_set_keys)` runtime admission | successor work, with proposed workset/lane | Metadata shape and deterministic capability mismatch helper exist; test `rich_arg_admission_metadata_validates_required_capability_keys`. No built-in rich-argument consumer exists. Successor lane: `W050-ARG-RICH-CONSUMER-ADMISSION` |
| `IMAGE` / `_webimage` producer capability publication | current evidence exists in OxFunc | Registry-level keys and stable capability key helpers; tests `image_registry_entry_publishes_webimage_producer_capabilities`, `registry_metadata_csv_exports_version_and_capability_columns`, `image_extended_surface_returns_webimage_rich_value` |
| `exercised_capability_keys` publication on returned rich carriers | current evidence exists in OxFunc | Current shape is adjacent returned-result metadata from `eval_image_surface_extended_with_capabilities(...)`, not fields embedded inside `RichValue`; tests `image_extended_capability_result_reports_successful_producer_exercise`, `image_extended_capability_result_does_not_claim_denied_provider_exercise` |
| generic rich producer protocol beyond `IMAGE` / `_webimage` | successor work, with proposed workset/lane | Successor lane: `W050-RICH-GENERIC-PRODUCER-PROTOCOL`; current code claims only `IMAGE` / `_webimage` |
| sparse range reader admission and replay semantics | successor work, with proposed workset/lane | Successor lane: `W050-SPARSE-RANGE-READER-ADMISSION-REPLAY`; blocked until OxFunc defines sparse reader API, runtime boundary, and replay-visible fields |

## 5. W110-2 `send-values` default flip (HO-FN-020, 2026-09-15)

Outbound note from OxFunc bead `oxf-xvt5.10`; the full record is
`docs/handoffs/HANDOFF_OXFML_opaque_callable_send_sync.md` ("Default flip").

1. `oxfunc_value_types` and `oxfunc_core` now declare
   `default = ["send-values"]`. For every default-features build,
   `oxfunc_core::value::Shared<T>` is `Arc<T>`, `OpaqueCallable: Send + Sync`,
   and `oxfunc_value_types::send_values_audit` proves `CalcValue`, `CoreValue`,
   `CalcArray`, `ReferenceLike`, `RichValue`, `RichObjectValue`,
   `PresentationValue`, `ErrorMetadataValue` and `CallableValue` are
   `Send + Sync` at compile time. OxCalc did not have to change anything for
   this: its only `Rc` is the engine-internal `TreecalcInvocationHostSlot`.
2. The four lines `assert_send::<GridCalcRefWorkbook>()`,
   `assert_send::<GridCalcRefSheet>()`, `assert_send::<GridOptimizedValuation>()`
   and `assert_send::<oxfunc_core::value::CalcValue>()` in
   `src/oxcalc-core/src/grid/machine.rs` `concurrency_prep_send_audit`
   (`#[cfg(test)]`) compile now — verified 2026-09-15 on a scratch copy of
   `oxcalc-core` with the feature on, which the default flip makes the ordinary
   build; a control `assert_send::<std::rc::Rc<u8>>()` in the same module
   fails with E0277 as the only error. The audit's "recorded upstream blocker"
   paragraph, which names `Option<Rc<RichValue>>` and `Rc<dyn OpaqueCallable>`,
   is stale as of this flip.
3. The `clippy::arc_with_non_send_sync` allowance in `OxCalc/Cargo.toml`
   ("until the W053 Rc->Arc value-model migration") is removable at the same
   time.
4. The `Rc` arm stays reachable through `--no-default-features` for one
   release. Item 2 landing in OxCalc is one of the three retirement conditions
   named in the handoff; until then OxFunc keeps the seam.
5. Arc clone cost on the evaluation hot path is not measured and no claim is
   made about it.

## 6. Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxCalc W050 closure decision after reviewing this landing classification
  - OxFml/OxCalc consumption of OxFunc metadata-version signals
  - `W050-SK-NRP-REPLAY`
  - `W050-SK-ERROR-ALGEBRA-FAMILY-WIRING`
  - `W050-ARG-RICH-CONSUMER-ADMISSION`
  - `W050-RICH-GENERIC-PRODUCER-PROTOCOL`
  - `W050-SPARSE-RANGE-READER-ADMISSION-REPLAY`
