# Notes for OxFml

Status: `active`
Owner lane: `OxFml`
Relationship: current OxFunc-to-OxFml seam note

## 1. Purpose

Capture the current OxFunc reading of what OxFml should preserve or prove next at the seam.

This is a current-state note, not a historical ledger.
It keeps only the distinctions, ownership splits, and bounded asks that still matter for current OxFunc closure work.

Canonical current-phase shared-model note:
1. `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`

Current outbound promotion packet:
1. the acknowledged freeze-promotion handoff now lives behind `OxFunc_V1`; use `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` plus this ledger as the live shared-model surface

Current reply-processing read:
1. OxFml's mirrored packet now treats `HO-FN-004` as acceptable current shared freeze wording for the narrowed seam families.

## 2. Current Summary

Current OxFunc reading:
1. the admitted `@` slice is real in OxFunc runtime, Lean alignment, native Excel replay, and the current OxFml adapter corpus.
2. the admitted `W038` callable/helper slice is also real in OxFunc runtime, Lean alignment, native Excel replay, and the current OxFml adapter corpus.
3. `HYPERLINK` is no longer a missing OxFunc kernel; it is value semantics plus publication intent, with actual style/click behavior above OxFunc.
4. `IMAGE` now has a pinned rich-value return carrier in OxFunc, and the latest OxFml note says the local evaluator/host/adapter lane is now exercised.
5. `CALL` / `REGISTER.ID` remain a real typed registered-external provider/admission/runtime seam.
6. `GROUPBY` and `PIVOTBY` now have real OxFunc callable-backed kernels plus bounded OxFml adapter coverage on real grouped-aggregation lanes; the remaining work is broader promotion/documentation rather than first adapter proof.

## 3. Current Seam Floor OxFunc Depends On

OxFunc currently depends on these seam facts remaining true:
1. explicit `@` remains observable and caller-context-sensitive rather than collapsing into generic top-left array picking.
2. callable/helper artifacts remain semantically real at the seam.
3. direct scalar, array-like, omitted, blank, and reference-observable distinctions are not erased prematurely.
4. bind-time helper rejection stays separate from evaluation-time function semantics.
5. result-class distinctions for publication-sensitive functions survive planning and evaluation.

## 4. `@` / `OP_IMPLICIT_INTERSECTION`

Current OxFunc reading:
1. `@` / `OP_IMPLICIT_INTERSECTION` is now complete for declared current-phase OxFunc scope.
2. the admitted current-baseline slice is covered by:
   - Rust runtime
   - Lean binding
   - native replay
   - OxFml adapter cases `B01` through `B07`
   - OxFml semantic-plan and evaluator legacy-single compatibility tests
   - current-baseline `_xlfn.SINGLE(...)` host normalization back onto explicit `@`
3. broader pre-dynamic-array roundtrip sweeps and structured-reference/table-context interaction are orthogonal future validation/interop lanes rather than current-version blockers on the OxFunc side.

Current OxFml implication:
1. preserve explicit `@` provenance and caller-context scalarization semantics
2. do not normalize `@` into a generic array-top-left shortcut

## 5. Helper / Callable Family (`W038`)

Current OxFunc reading:
1. `LET`, `LAMBDA`, `ISOMITTED`, `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` are no longer blocked by missing OxFunc kernels on the admitted slice.
2. the current adapter corpus already proves admitted direct/helper/higher-order lanes through the real OxFml parser/binder/preparation path.
3. the remaining live seam pressure is narrowed to:
   - promotion of the current minimum callable carrier into shared freeze text
   - any wider helper-bind rejection matrix beyond the currently exercised duplicate/malformed bounded cases

Current callable freeze candidate:
1. minimum shared callable carrier:
   - opaque callable identity/token
   - `origin_kind`
   - `arity_shape`
   - `capture_mode`
   - `invocation_contract_ref`
2. typed invocation over the opaque callable token remains the preferred boundary,
3. parameter-name lists, exact capture-name lists, and helper/body detail stay out of the shared minimum carrier,
4. if advanced replay/explain/serialization scenarios need richer callable detail, OxFml and OxFunc can cooperatively preserve that richer detail without widening the shared minimum carrier for the current phase.

Current OxFml implication:
1. keep helper formation and validation on the bind/admission side where Excel already rejects before evaluation
2. preserve prepared callable/helper distinctions honestly

## 6. `GROUPBY` / `PIVOTBY`

Current OxFunc reading:
1. OxFunc now has real callable-backed runtime kernels for both functions on an admitted current-baseline slice.
2. these rows are no longer blocked on missing OxFunc callable infrastructure.
3. OxFml now has bounded real adapter coverage for callable-backed grouped aggregation through the live parser/binder/preparation/evaluation path.
4. the remaining live work is broader promotion/documentation and any later widening beyond the currently exercised option lanes.

Current OxFml implication:
1. keep the existing grouped-aggregation adapter corpus stable as a real seam regression floor.
2. OxFunc does not need a new generic callable ABI round to proceed.
3. future widening should be evidence-driven rather than reopening the first bounded adapter ask.

## 7. `HYPERLINK` / `IMAGE`

### 7.1 `HYPERLINK`

Current OxFunc reading:
1. OxFunc owns worksheet-visible value semantics and publication intent.
2. the current OxFunc return split is:
   - value: visible text payload
   - publication hint: hyperlink-style/clickability intent
3. actual style application and click behavior remain host-owned.

Current OxFml implication:
1. do not collapse `HYPERLINK` to plain text if richer publication metadata can survive the seam
2. do not model `HYPERLINK` as if OxFunc itself performs host UI mutation

### 7.2 `IMAGE`

Current OxFunc reading:
1. `IMAGE` is still an open rich-value/publication seam, but it is no longer waiting on first local OxFml lane evidence.
2. OxFunc now has a real `IMAGE` runtime surface:
   - strict Excel-style argument validation for `source`, `alt_text`, `sizing`, `height`, and `width`
   - typed `HostInfoProvider::query_image(...)` request normalization for upstream file/web helpers
   - provider-classified `#CONNECT!` / `#BLOCKED!` / provider-error mapping
   - extended return shape `ExtendedValue::RichValue(_webimage)`
3. OxFunc still does not want `IMAGE` scalarized into plain text, a URL string, or a fake placeholder scalar.
4. OxFunc now treats that rich value as the semantic return carrier for successful `IMAGE(...)` evaluation. The published worksheet fallback string is a separate host-visible projection, not the semantic carrier.
5. the latest OxFml note is useful because it confirms explicit local `IMAGE(...)` evaluator/host/adapter evidence, generic non-ordinary return-surface preservation, explicit `_webimage` packet evidence, `TypedContextQueryFamily::Image`, and separation of the published fallback from the semantic carrier.

Current OxFml implication:
1. preserve the semantic class that `IMAGE` is richer than plain text or ordinary reference return
2. keep `ReturnedValueSurfaceKind::RichValue` plus `rich_value_type_name = "_webimage"` intact for admitted `IMAGE(...)` lanes
3. keep the published worksheet fallback separate from the semantic return carrier rather than letting publication rewrite the carrier class
4. `TypedContextQueryFamily::Image` looks like the right first freeze name for the typed host-query family
5. OxFunc does not currently need extra returned-value fields beyond the present `W042` vocabulary for current-phase `IMAGE` work
6. keep enough result-class/capability truth for a host-managed rich-value/publication model

## 8. `CALL` / `REGISTER.ID`

Current OxFunc reading:
1. OxFunc already has typed request normalization and worksheet result projection for the admitted slice.
2. OxFml now has a real `W052` proving-host floor for:
   - worksheet `REGISTER.ID`
   - worksheet `CALL`
   - reference-visible `CALL` arguments
   - host API registration
   - VBA shim registration
   - unregister packet carriage
3. direct adoption of OxFunc-owned `RegisterIdRequest`, `RegisteredExternalDescriptor`, and `RegisteredExternalCallRequest` packet types is now real on the OxFml side and is the right current direction.
4. the remaining open work is not ordinary function-kernel work.
5. the remaining open work is:
   - exact shared field naming
   - minimum `RegisteredExternalDescriptor` facts needed for descriptor-driven dereference and general type coercion
   - whether `RegisteredExternalCatalogMutation*` and controller surfaces should become OxFunc-owned shared runtime packet families or remain OxFml-owned funnel packets over OxFunc-owned catalog truth
   - minimum register/unregister consequences for `LibraryContextSnapshot` generation

Current OxFunc closure suggestions:
1. shared field naming:
   - freeze the current shared names as-is:
     - `RegisterIdRequest { library_name, procedure, declared_type_text }`
     - `RegisteredExternalDescriptor { stable_registration_id, register_id, origin_kind, display_name, library_name, procedure, declared_type_text }`
     - `RegisteredExternalCallRequest { target, invocation_args }`
     - `RegisteredExternalTarget::{RegisterId, Direct}`
2. minimum shared `RegisteredExternalDescriptor` field set:
   - keep the current `7`-field descriptor as the shared descriptor
   - OxFunc currently needs no additional descriptor fields beyond the present shape for descriptor-driven dereference/coercion decisions
3. mutation/controller family ownership:
   - for the current phase, keep `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` OxFml-owned funnel packets over OxFunc-owned catalog truth
4. minimum snapshot-generation consequences:
   - bind-visible registration/unregister => new `LibraryContextSnapshot` generation plus bind invalidation where the visible function/name world changes
   - `CALL` / `REGISTER.ID`-only descriptor mutation => targeted reevaluation by default
5. downgraded from packet-freeze blockers:
   - broader omitted-`type_text` characterization
   - worksheet-vs-macro-sheet admission/version characterization
   - useful evidence, but not current shared packet-freeze blockers

Current OxFml implication:
1. keep `RegisteredExternalProvider` distinct from ordinary host-info/query seams
2. preserve typed registration and invocation packets as real runtime objects, not just note-level ideas
3. keep the current `W052` proving-host corpus stable as a real seam regression floor while the remaining packet fields are frozen

## 9. Current Bounded Ask

OxFunc's current bounded ask to OxFml is:
1. keep the current `W053` grouped-aggregation adapter corpus stable enough to act as a real seam regression floor for `GROUPBY` / `PIVOTBY`.
2. preserve the bind-side classification that duplicate and malformed helper declarations are OxFml-owned admission failures rather than OxFunc runtime cleanup.
3. keep publication-sensitive returned-value distinctions explicit for `HYPERLINK` and keep the locked `_webimage` rich-value carrier explicit for `IMAGE`.
4. continue tightening the typed runtime packet for `CALL` / `REGISTER.ID` rather than widening note traffic.
5. for `W052`, reply with concrete field-freeze decisions rather than a parallel wrapper vocabulary unless OxFunc explicitly asks for one.
6. treat direct adoption of `RegisterIdRequest`, `RegisteredExternalDescriptor`, and `RegisteredExternalCallRequest` as settled unless a concrete mismatch forces reopening.

## 10. What OxFunc Is Not Asking For

OxFunc is not currently asking OxFml for:
1. a broad callable ABI beyond the current minimum shared callable carrier
2. a generic provenance redesign
3. a generic callable-note round
4. a re-open of the first bounded `GROUPBY` / `PIVOTBY` adapter expansion that OxFml has now already landed
5. premature scalarization of `IMAGE`

## 11. Current Closing Sequence

Current OxFunc reading of the best next sequence is:
1. use the existing adapter floor to finish narrowing `W038` and `W014`
2. treat the landed `W053` grouped-aggregation and helper-bind rejection corpus as the current callable-heavy regression floor
3. keep `HYPERLINK` / `IMAGE` publication-class distinctions explicit
4. continue `CALL` / `REGISTER.ID` as a typed registered-external seam packet rather than ordinary function work
5. treat the archived freeze-promotion handoff behind `OxFunc_V1` as acknowledged from the OxFml side
6. use the mirrored packet plus `HO-FN-004` as the shared freeze floor for local owner-packet promotion and propagation

## 12. Current Summary To OxFml

Current OxFunc position to OxFml:
1. `@` and the admitted helper family are already real end-to-end seam facts, not note-only topics.
2. callable-backed grouped aggregation plus bounded helper bind-time rejection coverage is now real and verified.
3. `HYPERLINK` should preserve publication intent, and `IMAGE` should preserve the locked `_webimage` rich-value carrier while keeping published fallback separate.
4. the latest OxFml note sharpens `IMAGE` by confirming a real local `IMAGE(...)` evaluator/host/adapter lane, `TypedContextQueryFamily::Image`, and preserved `_webimage` rich-value carriage.
5. `CALL` / `REGISTER.ID` now have a real typed `W052` interface floor and OxFml now treats the narrowed packet freeze as acceptable current shared freeze wording.
6. OxFml's latest closure packet narrows `W052` to four shared decisions only: exact field naming, minimum `RegisteredExternalDescriptor` field set, mutation/controller family ownership, and snapshot-generation consequences.
7. the latest OxFml note now records those four `W052` decisions as acknowledged on the OxFml side; the remaining work is coordinator-facing propagation and canonical promotion rather than packet-shape redesign.
8. the current OxFunc-owned consolidated freeze candidate for the seam-relevant non-deferred surface now lives in `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.
9. the active OxFunc outbound shared-model reference for the next round now lives in `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.
10. OxFml's mirrored packet now reads `HO-FN-004` as acceptable current shared freeze wording for the narrowed seam families.

## 13. Locale/Format Seam Ownership Realignment (W082 / HO-FN-009)

Current OxFunc reading:
1. OxFunc should own function semantics and typed seam contracts, not a production fallback locale/format convenience bundle.
2. `crates/oxfunc_core/src/locale_format.rs` no longer exports ordinary production `en_us_context()` or `current_excel_host_context()` convenience shims; the remaining local parser/formatter is explicit `#[cfg(test)]` support.
3. `tools/xll-addin/oxfunc_xll/src/lib.rs` now supplies its own caller-owned host context and delegates parse/render behavior to Excel through `xlfEvaluate`.
4. OxFml still imports `en_us_context()` broadly in evaluator-facing tests, for example `../OxFml/crates/oxfml_core/tests/evaluator_tests.rs`.
5. `W082` and `HO-FN-009` now own the migration onto caller-supplied capability bundles with no backward-compatible OxFunc fallback.

Current OxFml implication:
1. stop treating `en_us_context()` as the shared stable seam for evaluator or test setup.
2. construct locale/format parsing and rendering capabilities on the caller side and pass them through the typed seam explicitly.
3. update evaluator/test helpers so OxFml can stand up its own capability bundle without importing OxFunc convenience shims.
4. acknowledge `HO-FN-009` once the replacement construction pattern is landed and both repos point at the same typed seam vocabulary.

Minimum invariants:
1. function semantics remain OxFunc-owned.
2. locale/format authority lives at the caller/host seam rather than behind an OxFunc fallback helper.
3. no production path should depend on `current_excel_host_context()` or `en_us_context()` once the migration is complete.
4. the XLL add-in and OxFml evaluator/tests should converge on the same caller-supplied capability model.
