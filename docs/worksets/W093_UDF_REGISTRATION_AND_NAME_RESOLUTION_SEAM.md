# W093 UDF Registration And Name-Resolution Seam

Status: `in_progress`

## 1. Purpose

Design the DNA Calc UDF registration seam across XLL, VBA, JavaScript custom
functions, Automation-style registration, and registered-external worksheet
`REGISTER.ID` / `CALL` surfaces without creating another comprehensive
function list.

This workset builds on `W091`: OxFunc owns the canonical runtime function
registry for built-ins and runtime-registered UDF function entries. OxFml owns
formula grammar, binding, formula-name resolution, and bind/editor cache
invalidation. Workbook/sheet defined names remain document/formula-environment
state, not OxFunc function-registry state.

## 2. Problem Statement

UDF registration reaches the calculation system through multiple source
mechanisms:

1. XLL add-ins call Excel C API registration paths such as `xlfRegister`.
2. VBA hosts discover public standard-module functions during project/module
   load or edit.
3. JavaScript custom functions are declared by add-in manifest/JSON metadata
   and executed through an Office add-in runtime.
4. Worksheet `REGISTER.ID` returns a register id for external code resources
   and is adjacent to `CALL`; it does not by itself create an ordinary
   worksheet-visible UDF entry with completion/signature/help metadata.

All of these can affect whether formula text resolves to a callable function.
They must converge on OxFunc registry truth without making OxFml, DNA OneCalc,
or a host maintain a duplicate function list.

## 3. Scope

In scope:

1. source-neutral UDF registration request/result contract,
2. source-specific metadata mapping for XLL, VBA, JavaScript custom functions,
   Automation, and registered-external worksheet lanes,
3. registry snapshot identity/change-set semantics for formula binding and
   editor cache invalidation,
4. collision and precedence policy between built-ins, UDFs, and formula names,
5. capability-overlay interaction for unavailable or host-gated UDFs,
6. OxFml handoff for name-resolution and cache invalidation,
7. deterministic seam tests and replay scenarios for the first implementation
   pass.

Out of scope:

1. full XLL marshalling and lifetime parity,
2. full VBA runtime implementation,
3. full JavaScript add-in runtime implementation,
4. full workbook defined-name ownership in OxFunc,
5. treating sampled registration behavior as function semantic closure,
6. replacing the W091 canonical registry with a second UDF catalog.

## 4. Ownership Decisions

1. OxFunc owns `FunctionRegistry` entries, UDF registration/unregistration,
   registry snapshot identity, source classification, signature metadata, and
   capability projections for callable function entries.
2. OxFml owns formula parse/bind, name lookup, function-call binding,
   editor-help lookup, and invalidation of bind/editor artifacts when the
   registry epoch or formula-name environment changes.
3. Workbook/sheet defined names, LAMBDA names, table names, and other
   formula-name environment entries belong above OxFunc as document/formula
   state. OxFml consumes that environment during binding and evaluation.
4. Host loaders and add-in adapters own source discovery. They normalize their
   findings into OxFunc UDF registration requests only when the source creates
   a worksheet-visible callable function entry.
5. Registered-external descriptor lookup and invocation remain adjacent seam
   state. They should not be promoted into ordinary UDF entries unless friendly
   worksheet-visible function metadata is supplied.

## 5. Initial Seam Shape

The first OxFunc contract should define:

1. `UdfRegistrationRequest`,
2. `UdfRegistrationResult`,
3. `UdfSourceKind`,
4. `UdfExecutionProfile`,
5. `RegistryChangeSet`,
6. `FunctionRegistrySnapshotIdentity`,
7. collision/replacement policy,
8. unregister semantics,
9. capability-overlay interaction,
10. invocation-target descriptors separated from callable surface metadata.

Source adapters should map into that shape:

1. XLL maps `xlfRegister` metadata into callable surface name, arity/signature,
   volatility, thread-safety, category, help, and a separate invocation target
   descriptor containing exported target/type-text details.
2. VBA maps public standard-module functions into host-discovered UDF entries
   with project/module/procedure provenance.
3. JavaScript maps custom-function JSON/manifest metadata into namespaced UDF
   entries with descriptions, parameters, result kind, invocation/calling-object
   needs, async/streaming/cancelability flags, autocomplete visibility, and host
   runtime capability requirements.
4. Worksheet `REGISTER.ID` / `CALL` remains a registered-external descriptor
   and invocation seam unless friendly worksheet function metadata is supplied.

## 5A. Public Source Mapping Evidence

Current source evidence maps into the existing W093 request shape without adding
new source kinds.

| Source lane | Bind-visible UDF entry condition | `UdfSourceKind` | Source registration id | Invocation target |
| --- | --- | --- | --- | --- |
| XLL `xlfRegister` | Registration supplies worksheet-visible function text plus stable exported procedure details. | `XllRegisteredFunction` | Host-stable add-in/module/procedure/type-text registration key. | `UdfInvocationTargetDescriptor::Xll` with module path when known, export name, type text, optional register id, and opaque runtime values. |
| VBA public module function | Host discovers a public standard-module function admitted by macro/security policy. | `VbaPublicModuleFunction` | Host-stable workbook/project/module/procedure key, versioned by project/module edit identity. | `UdfInvocationTargetDescriptor::Vba` with project ref, module name, procedure name, and opaque runtime values. |
| JavaScript custom function | Add-in metadata declares a worksheet-visible custom function id/name pair. | `JavaScriptCustomFunction` | Add-in identity plus namespace/custom-function id and metadata version. | `UdfInvocationTargetDescriptor::JavaScript` with add-in id, optional namespace, custom-function id, runtime ref, and opaque runtime values. |
| Automation registration | Host admits a COM/Automation member as a worksheet-visible function. | `AutomationRegisteredFunction` | ProgID or CLSID plus member/signature identity under the host adapter. | `UdfInvocationTargetDescriptor::Automation` with ProgID or CLSID, member, and opaque runtime values. |
| Registered-external-backed UDF | Host supplies friendly worksheet-visible metadata for a registered-external descriptor. | `RegisteredExternalBridge` or `HostRegisteredExternal` according to adapter ownership. | Stable registered-external descriptor id plus friendly surface metadata identity. | `UdfInvocationTargetDescriptor::RegisteredExternal` or `HostOpaque`. |
| Plain worksheet `REGISTER.ID` / `CALL` | None by default; descriptor-only state is not a function-registry entry. | None for ordinary registry mutation. | Registered-external descriptor id only. | Registered-external packet state outside ordinary `FunctionRegistry`. |

Mapping decisions:

1. `surface_name`, `arity`, `parameters`, `display_signature`,
   `short_description`, `long_description`, and `category` are the callable
   worksheet surface. If the source cannot supply that surface, it does not
   create a bind-visible UDF entry.
2. `stable_source_registration_id` is source-local but must be stable across
   refreshes of the same underlying registration so unregister and same-source
   update can be deterministic.
3. `source_provenance` may carry compact source-local provenance for audit and
   replay, but formula binding must use the stable function id and registry
   snapshot identity rather than parsing provenance text.
4. `UdfExecutionProfile` carries capability and execution availability facts;
   disabling macros, add-ins, Automation, or external libraries projects
   availability and does not delete the registry entry.
5. JavaScript custom-function namespace, autocomplete visibility, streaming,
   cancellation, and calling-object details remain the dedicated
   `oxf-ypq2.9` metadata bead before source-adapter promotion.
6. `REGISTER.ID` / `CALL` descriptor-only mutation remains the W046/W052
   registered-external lane and may produce targeted reevaluation evidence, not
   broad function-registry rebinding.

## 6. Name-Resolution And Invalidation Direction

OxFml should bind against a function registry view or registry-derived immutable
snapshot with an explicit snapshot identity.

Initial policy:

1. parse artifacts are independent of the function registry,
2. bind and semantic-plan artifacts include the function registry snapshot
   identity used,
3. editor completion/signature-help reads the current registry view or a fresh
   snapshot,
4. formulas that previously produced `#NAME?` may become bindable after UDF
   registration,
5. formulas bound to a UDF may become `#NAME?` or capability-blocked after
   unregister or capability denial,
6. evaluation validates that the bound function identity is still present and
   available in the active registry/capability view,
7. registry mutation publishes a new immutable registry-backed snapshot
   generation rather than a hidden mutable epoch,
8. `REGISTER.ID` / `CALL` descriptor-only mutation causes targeted
   reevaluation by default unless it also changes the bind-visible function-name
   world.

## 7. Initial Bead Lanes

1. source-evidence and public-doc mapping,
2. W46/W052 registered-external seam reconciliation,
3. OxFunc UDF registration contract,
4. OxFml invalidation and name-resolution handoff,
5. collision/precedence empirical evidence,
6. first registry API implementation slice,
7. deterministic seam tests and replay evidence,
8. host-adapter follow-on planning.

## 8. Reporting Contract

All W093 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Initial status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: API contract, OxFml handoff acknowledgement, source adapter
   mappings, registered-external reconciliation, registry snapshot integration,
   invocation-target descriptors, collision evidence, deterministic seam tests,
   and host-runtime follow-ons.

## 2026-05-22 OxFunc Repo-Local Contract/API Tranche

Reviewed inbound observations:
`../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` now acknowledges `HO-FN-014` as
the active W074 design lane for registry snapshot identity, name-resolution
invalidation, UDF-vs-defined-name precedence, and migration from static
built-in lookup to registry-backed formula-call binding.

OxFunc repo-local changes:

1. `crates/oxfunc_core/src/registry.rs` now exposes source-neutral W093
   mutation packets for UDF registration and unregistration.
2. Successful bind-visible mutations emit immutable
   `FunctionRegistrySnapshotIdentity` and `RegistryChangeSet` values.
3. Rejected mutations return typed `UdfRegistrationResult::Rejected` or
   `UdfUnregistrationResult::Rejected` without advancing semantic snapshot
   identity.
4. Descriptor-only `REGISTER.ID` / `CALL` catalog changes are represented as a
   change set with unchanged snapshot identity and
   `descriptor_only_mutation = true`.
5. `FunctionEntry` remains the callable worksheet surface descriptor, while
   `UdfInvocationTargetDescriptor` stores XLL, VBA, JavaScript, Automation,
   registered-external, or host-opaque invocation routing separately.
6. `UdfOpaqueRuntimeValue` preserves `ReferenceLike` and host references as
   opaque values. No TreeCalc-specific function branch or precedence rule was
   introduced.
7. Existing W091 `register_udf(FunctionEntry)` and `unregister_udf(function_id)`
   APIs remain for current downstream callers.

Guardrail response to OxFml coordinator report:

1. The compile-blocking W093 helper symbols are present in
   `registry.rs`: `stable_registry_fingerprint`, `typed_rejection`, and
   `udf_entry_from_request`.
2. The OxFunc-focused and registered-external integration checks compile and
   pass after the local registry/test adjustments listed below.

Validation evidence:

1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib registry`:
   passed, `24` passed.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib`: passed,
   `1307` passed, `1` ignored.
4. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --test oxfml_registered_external_interface_integration`:
   passed, `3` passed.
5. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml`: passed.

Status axes after this tranche:

1. `execution_state`: `in_progress`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: OxFml W074 formula-call registry lookup and invalidation,
   Excel oracle precedence matrix, source-adapter implementations, broad UDF
   execution semantics.

## 2026-05-22 Collision And Precedence Evidence Intake

Clean-room evidence currently comes from OxFml `W074-CALC005` Excel COM 16.0
oracle rows plus OxFunc local registry mutation tests.

Evidence now available for W093:

1. UDF versus defined-name call collision:
   - `W074-CALC005-003` observes `=UDF(1)` with a VBA UDF and workbook
     defined name `UDF=77` returning `#VALUE!`, while the control UDF call
     without the defined-name collision returns `11`.
   - W093 interpretation: the collision is an OxFml formula-name precedence
     rule, not an OxFunc registry collision rule.
2. UDF versus defined-name non-call collision:
   - `W074-CALC005-004` observes bare `=UDF` resolving to the workbook defined
     name value `77`.
   - W093 interpretation: workbook/sheet defined names remain document/formula
     environment state and must not be moved into the OxFunc function registry.
3. Built-in versus defined-name contrast:
   - `W074-CALC005-001` and `W074-CALC005-002` observe built-in `SUM(...)`
     winning call-callee position while bare `=SUM` resolves to the workbook
     defined name.
   - W093 interpretation: built-in protection in OxFunc registry mutation is
     still valid, but bare-name meaning belongs to OxFml name resolution.
4. Late UDF registration and unregister:
   - `W074-CALC005-012` observes late VBA UDF registration changing
     `=LateUdf(2)` from `#NAME?` to `22`.
   - `W074-CALC005-013` observes UDF removal returning to `#NAME?`.
   - W093 interpretation: bind-visible registration/unregister must advance
     registry snapshot identity and trigger OxFml bind/cache invalidation.
5. UDF replacement/update by same source registration id:
   - OxFunc registry tests exercise same-source update replacement,
     same-source update rejection when it would collide with another UDF
     surface, typed rejection without snapshot advancement, and unregister
     change sets.
   - W093 interpretation: this is an OxFunc registry mutation rule independent
     of Excel name-world precedence.

Still open after this intake:

1. JavaScript namespaced custom-function collision behavior remains source
   evidence work under `oxf-ypq2.9`.
2. Broader workbook/sheet/UDF/defined-name combinations remain W074 oracle
   work and are not frozen by this W093 intake.
3. OxFml formula-call registry lookup and cache invalidation implementation
   evidence remains `oxf-ypq2.12` / W074 work.
4. Source-adapter implementations remain successor W093 work.

## 2026-05-22 Registered-External Seam Reconciliation

W093 is reconciled with the earlier W046/W052 registered-external seam.

Current settled split:

1. `REGISTER.ID` / `CALL` descriptor-only mutation remains adjacent
   registered-external state, not ordinary UDF function registration.
2. Plain worksheet `REGISTER.ID` returning a register id does not create an
   editor-completion entry, signature-help row, or bind-visible ordinary
   function entry.
3. `CALL` by register id or direct `{ library_name, procedure,
   declared_type_text }` target remains a registered-external invocation lane.
4. A registered-external-backed ordinary UDF entry is created only when the
   host also supplies friendly worksheet-visible function metadata: stable
   surface name, arity/signature, callable metadata, and source registration
   identity.
5. Descriptor-only catalog mutation may produce a `RegistryChangeSet` with
   unchanged `FunctionRegistrySnapshotIdentity` and
   `descriptor_only_mutation = true`, supporting targeted reevaluation instead
   of broad formula rebinding by default.

Current packet alignment:

1. `RegisterIdRequest`, `RegisteredExternalDescriptor`, and
   `RegisteredExternalCallRequest` remain the typed registered-external packet
   family.
2. `RegisteredExternalDescriptor` keeps the current seven-field shape:
   stable registration id, register id, origin kind, display name, library
   name, procedure, and declared type text.
3. `RegisteredExternalCatalogMutation*` / controller packets remain
   OxFml-owned funnel packets for the current phase unless a downstream
   consumer later needs them as shared OxFunc-owned runtime packet families.
4. Bind-visible function registration/unregister still advances registry
   snapshot identity; descriptor-only `CALL` / `REGISTER.ID` mutation does not
   by itself advance the ordinary function registry identity.

No further W093 contract narrowing is needed for the current registered-external
split.

## 2026-05-22 JavaScript Custom-Function Metadata Mapping

Public source anchors:

1. Microsoft Office Add-ins custom-functions JSON metadata:
   <https://learn.microsoft.com/en-us/office/dev/add-ins/excel/custom-functions-json>
2. Microsoft Office Add-ins JSDoc metadata autogeneration:
   <https://learn.microsoft.com/en-us/office/dev/add-ins/excel/custom-functions-json-autogeneration>
3. Microsoft Office Add-ins streaming/cancelable custom-functions behavior:
   <https://learn.microsoft.com/en-us/office/dev/add-ins/excel/custom-functions-web-reqs>
4. Microsoft Office Add-ins invocation address options:
   <https://learn.microsoft.com/en-us/office/dev/add-ins/excel/custom-functions-parameter-options>

Mapping into W093:

| JavaScript metadata fact | W093 field |
| --- | --- |
| Add-in identity and manifest/JSON metadata version | `stable_source_registration_id` component and `source_provenance` |
| Function `id` | `function_id` component and `UdfInvocationTargetDescriptor::JavaScript.custom_function_id` |
| Function `name` | `surface_name` |
| Function `description` / `helpUrl` | `short_description` / `long_description` or provenance until richer help publication exists |
| Parameter `name`, `description`, `type`, `dimensionality` | `ParameterDescriptor`, `Arity`, and source provenance for JS-specific type/dimensionality detail |
| Result `type`, result `dimensionality`, custom enum metadata | Source provenance and future adapter metadata; not a new `EvalValue` variant by itself |
| Namespace from manifest resources | `UdfInvocationTargetDescriptor::JavaScript.namespace` and source registration id component |
| Runtime URL/ref from manifest/runtime binding | `UdfInvocationTargetDescriptor::JavaScript.runtime_ref` |
| `stream` / `@streaming` | `UdfExecutionProfile.streaming = true`; invocation target remains JavaScript |
| `cancelable` / `@cancelable` | `UdfExecutionProfile.cancellable = true`; mutually exclusive with streaming for admitted metadata |
| `volatile` / `@volatile` | `VolatilityClass` mapping once the adapter has a stable OxFunc volatility profile |
| `excludeFromAutoComplete` / `@excludeFromAutoComplete` | Editor-surface projection metadata; it does not remove the callable registry entry |
| `requiresAddress`, `requiresParameterAddresses`, stream address variants | Caller-context dependency in source provenance and future adapter metadata |
| `capturesCallingObject` | Caller-object dependency in source provenance and future adapter metadata |

Decisions:

1. Existing `UdfSourceKind::JavaScriptCustomFunction`,
   `UdfExecutionProfile`, and
   `UdfInvocationTargetDescriptor::JavaScript` are sufficient for current W093.
2. Namespace and custom-function `id` are source identity and invocation-target
   facts, not separate Excel built-in or TreeCalc host-name lanes.
3. JavaScript autocomplete visibility is an editor projection over the registry;
   it must not delete or hide the callable entry from binding if the formula
   explicitly names it.
4. Streaming and cancelable functions are availability/execution-profile facts.
   The source adapter must reject or type-diagnose metadata that tries to admit
   both for the same function.
5. Invocation address, parameter-address, and calling-object requirements are
   caller-context dependencies. OxFunc records them as metadata; OxFml/host
   runtime owns supplying the actual invocation context.
6. Custom data types, linked entities, custom enums, and JS-specific result
   dimensionality do not create TreeCalc-specific function branches in OxFunc.
   They remain source-provenance or future typed adapter metadata until a
   concrete rich-value lane is admitted.

Still open:

1. JavaScript runtime execution and marshalling are source-adapter work, not
   closed by this metadata mapping.
2. Namespaced custom-function collision behavior against workbook/sheet defined
   names remains W074/OxFml evidence before name/call freeze.
3. Registry-backed formula-call lookup and invalidation remains `oxf-ypq2.12`
   / OxFml W074 work.

## 2026-05-23 W056 Structured-Table ReferenceLike Guardrail

Reviewed inbound observations:
`../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` and OxCalc W056 both keep the
same ownership split for table references: OxFml owns generic
structured-reference binding, OxCalc owns TreeCalc table selectors/readers and
dependency lowering, and OxFunc consumes only opaque `ReferenceLike` plus
resolver/reader APIs.

OxFunc repo-local changes:

1. `ReferenceKind::Structured` targets are now exempt from the generic
   bracket-means-external-workbook guard in `resolver.rs`; bracketed structured
   targets remain opaque carrier strings rather than parsed selectors.
2. The existing generic `allow_structured_refs` capability still rejects
   structured carriers before provider calls when the caller denies structured
   reference resolution.
3. `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK` are exercised over bracketed
   structured-table carriers through `ReferenceResolver::resolve_reference_values`
   without dense `resolve_reference` calls.
4. The exercised carriers cover table data column, whole data section, header
   section, totals section, and current-row forms. Sparse blank handling is
   exercised through `COUNTBLANK` using declared extent minus defined cells
   plus defined empty-string cells.
5. Direct scalar and direct array behavior for the first aggregate group is
   unchanged by the structured carrier guardrail.

Validation evidence:

1. `cargo fmt --manifest-path crates\oxfunc_core\Cargo.toml`: passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --test structured_table_reference_guardrails`:
   passed, `3` passed.

Status axes for this W056 table-carrier slice:

1. `execution_state`: `in_progress`
2. `scope_completeness`: `scope_complete` for the first aggregate guardrail
   slice only
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: sparse-reader widening beyond `SUM` / `COUNT` / `COUNTA` /
   `COUNTBLANK`, context-sensitive reference-returning/table-shape functions,
   and downstream W056 retained table evidence outside OxFunc write authority.

Range-taking function inventory:

1. First exercised sparse-reader group: `SUM`, `COUNT`, `COUNTA`, and
   `COUNTBLANK` are supported for opaque structured-table carriers through
   generic resolver/reader APIs.
2. Existing dense generic reference path: ordinary value-taking scalar,
   aggregate, text, statistical, lookup-vector, and array-producing functions
   can consume structured carriers only when the resolver materializes them as
   ordinary `EvalValue`/`EvalArray`; this is not sparse table-reader coverage.
3. Reference-visible/context-sensitive functions such as `AREAS`,
   `FORMULATEXT`, `CELL`, `ROW`, `COLUMN`, `ROWS`, `COLUMNS`, `INDEX`,
   `OFFSET`, `MATCH`, `XLOOKUP`, `SUBTOTAL`, `AGGREGATE`, `CALL`,
   `OP_IMPLICIT_INTERSECTION`, `OP_SPILL_REF`, and reference operators require
   function-specific classification or typed exclusion before W056 can claim
   product-wide table reference behavior through OxFunc.
4. Structured-table name/defined-name/call precedence, table context identity,
   header/totals/current-row availability, and TreeCalc selector meaning remain
   OxFml/OxCalc-owned inputs, not OxFunc function branches.

Successor beads:

1. `oxf-ypq2.15` tracks sparse-reader widening for aggregate/statistical/text
   functions beyond the first group.
2. `oxf-ypq2.16` tracks reference-visible and context-sensitive structured-table
   behavior classification or typed exclusions.
