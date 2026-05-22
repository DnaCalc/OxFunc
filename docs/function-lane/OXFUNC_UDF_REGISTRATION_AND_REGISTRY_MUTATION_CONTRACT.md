# OxFunc UDF Registration And Registry Mutation Contract

Status: `contract_seed`

Owning workset:
`docs/worksets/W093_UDF_REGISTRATION_AND_NAME_RESOLUTION_SEAM.md`

## 1. Purpose

Define the future-facing OxFunc contract for runtime UDF registration without
creating another comprehensive function registry.

This contract extends the W091 canonical runtime function registry direction:
OxFunc owns callable function entries and runtime registry mutations; OxFml and
hosts consume registry views or registry-derived snapshots.

## 2. Source-Neutral Registration Shape

The first implementation slice should introduce source-neutral concepts:

1. `UdfRegistrationRequest`
2. `UdfRegistrationResult`
3. `UdfSourceKind`
4. `UdfExecutionProfile`
5. `UdfInvocationTargetDescriptor`
6. `RegistryChangeSet`
7. `FunctionRegistrySnapshotIdentity`
8. collision policy
9. unregister policy

Required registration fields:

1. stable source-local registration id,
2. surface name,
3. canonical runtime function id,
4. source kind,
5. provenance payload,
6. arity and parameter descriptors,
7. display signature,
8. volatility and determinism declarations,
9. thread-safety or async/streaming declarations where known,
10. help/category metadata where supplied,
11. host execution profile or capability requirements,
12. optional invocation-target descriptor reference.

## 3. Source Kinds

Initial `UdfSourceKind` values:

1. `XllRegisteredFunction`
2. `VbaPublicModuleFunction`
3. `JavaScriptCustomFunction`
4. `AutomationRegisteredFunction`
5. `RegisteredExternalBridge`
6. `HostRegisteredExternal`

Interpretation:

1. XLL, VBA, JavaScript, Automation, and host registrations may produce
   worksheet-visible UDF entries when they supply a stable surface name and
   signature metadata.
2. `RegisteredExternalBridge` is an adjacent source marker for rows whose
   execution is backed by registered-external descriptor state, but it is not a
   request to put every `REGISTER.ID` descriptor in the ordinary function
   registry.
3. `CALL` / register-id invocation remains a registered-external seam unless
   the host also registers a friendly worksheet-visible function entry.
4. Plain worksheet `REGISTER.ID` that only returns a numeric register id should
   not create an editor completion, signature-help entry, or bind-visible
   ordinary function.

Source mapping:

| Source | Required callable-surface facts | Required invocation facts | Notes |
| --- | --- | --- | --- |
| XLL `xlfRegister` | Surface name, argument count/type-derived arity, argument names when available, category/help metadata when supplied, volatility/thread-safety flags when evidenced. | Export/procedure name, optional module path, type text, optional register id. | Transport and marshalling stay in the XLL adapter; OxFunc receives normalized registry facts. |
| VBA public standard-module function | Procedure name as surface name, arity/parameter names discovered by the host, project/module provenance, macro-security availability. | Project ref, module name, procedure name. | Module edits may update or unregister by the same source registration id. |
| JavaScript custom function | Add-in id, namespace/id, function name, description, parameters, result shape, autocomplete visibility, and execution flags where metadata supplies them. | Add-in id, namespace, custom-function id, runtime ref. | Detailed JavaScript metadata mapping remains `oxf-ypq2.9`; this row confirms the existing source kind is sufficient. |
| Automation registered function | Friendly worksheet surface name and signature supplied by the host adapter. | ProgID or CLSID plus member dispatch identity. | Capability denial blocks availability without deleting the registry entry. |
| Registered-external-backed UDF | Friendly surface name, arity/signature, callable metadata, and source registration identity supplied by a host. | Stable registered-external descriptor id or host opaque target id. | Descriptor-only `REGISTER.ID` / `CALL` state is not enough to create a registry entry. |

No additional `UdfSourceKind` is required for the current public source mapping
pass. If a future adapter needs finer source taxonomy, it should add fields to
source provenance or invocation-target descriptors first and add a new source
kind only when registry-level mutation policy differs.

## 4. Registry Snapshot Identity And Change Set

Every successful bind-visible registry mutation should produce a new immutable
registry-backed snapshot identity. `FunctionRegistrySnapshotIdentity` should be
the same semantic identity that OxFml pins in bind, semantic-plan, editor-help,
and replay artifacts, or a field inside that existing immutable snapshot
identity. It must not become an unrelated second invalidation axis.

`RegistryChangeSet` should record:

1. previous snapshot identity,
2. new snapshot identity,
3. added function ids,
4. removed function ids,
5. replaced function ids,
6. changed surface names,
7. affected source registration ids.

Consumers use this change set for bind/editor cache invalidation. The change
set is not a second catalog; it is a mutation summary over the OxFunc registry.

Rejected mutations return typed `UdfRegistrationResult::Rejected` outcomes and
do not advance the registry snapshot identity unless a future audit-log epoch is
separately introduced and explicitly distinguished from semantic registry
identity.

## 5. Collision And Replacement Policy

Default policy:

1. built-in surface names are protected,
2. UDF-to-UDF same-surface collisions are rejected unless the same source
   registration id is updating its own entry,
3. built-in replacement requires an explicit source/capability policy,
4. function ids are stable and unique within a registry epoch,
5. unregister by unknown id or source registration id returns a typed no-op or
   typed error, not silent success.

Any future Excel-compatible shadowing behavior must be documented with
reproducible public or empirical evidence before promotion.

The implementation lane must not start until W093 records a first empirical or
public-doc evidence pass for:

1. UDF-vs-built-in collisions,
2. UDF-vs-UDF replacement/update by the same source registration id,
3. workbook/sheet defined-name versus function-call name precedence,
4. namespaced JavaScript custom-function behavior.

## 6. Capability Interaction

Registration changes the entry set. Capability overlays project availability
over that entry set.

Examples:

1. a JavaScript custom function can be registered but unavailable when the JS
   runtime is disabled,
2. an XLL UDF can be registered but host-blocked in an untrusted profile,
3. a VBA UDF can be present but unavailable when macros are disabled,
4. a registered external can be present but provider-blocked.

Capability denial must not delete the registry entry.

## 7. Invocation Target Separation

`FunctionEntry` should remain the callable worksheet surface descriptor. It
should not become the raw execution ABI for every source.

`UdfInvocationTargetDescriptor` should carry source-specific execution routing:

1. XLL exported procedure/module, type text, register id, and calling-convention
   relevant facts,
2. VBA project/module/procedure provenance and macro-security profile,
3. JavaScript add-in id, namespace, custom-function id, runtime, async or
   streaming behavior, cancellation support, and invocation/calling-object
   needs,
4. Automation ProgID/CLSID/member dispatch facts where admitted,
5. registered-external descriptor reference when an ordinary UDF surface is
   backed by the registered-external seam.

Evaluation binds to the stable function id and then resolves the invocation
target under the active host capability/security profile.

## 8. OxFml Consumption Contract

OxFml should consume:

1. registry snapshot identity,
2. registry lookup by surface name,
3. bound function id,
4. availability from capability-scoped registry views,
5. registry change sets for bind/editor invalidation.

OxFml should not maintain a duplicate comprehensive UDF list.

OxFml formula-call binding/evaluation must migrate from static built-in
metadata lookup to a registry-backed lookup path for UDF-aware contexts. Editor
help/completion consumption alone is not enough for W093 closure.

## 9. Evidence Requirements

The first promoted implementation slice needs deterministic tests for:

1. register then bind,
2. unregister then stale binding invalidation,
3. source update changing signature,
4. built-in collision rejection,
5. UDF-to-UDF collision rejection/update,
6. capability denial without registry deletion,
7. `REGISTER.ID` returning a registered-external id without ordinary
   function-help metadata,
8. JavaScript namespaced custom-function registration,
9. `REGISTER.ID` / `CALL` descriptor-only mutation causing targeted
   reevaluation rather than broad bind invalidation,
10. registry-backed formula-call bind/evaluation for a UDF-aware context.

Status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: Rust API implementation, OxFml consumer integration,
   registered-external reconciliation, source-adapter detail, invocation target
   descriptors, collision evidence, and deterministic replay evidence.

## 2026-05-22 Repo-Local API Tranche

OxFunc now exposes the first repo-local W093 registry mutation API tranche in
`oxfunc_core::registry`.

Added Rust API surface:

1. `UdfRegistrationRequest`
2. `UdfRegistrationResult`
3. `UdfUnregistrationResult`
4. `UdfSourceKind`
5. `UdfExecutionProfile`
6. `UdfInvocationTargetDescriptor`
7. `UdfOpaqueRuntimeValue`
8. `UdfReplacementPolicy`
9. `RegistryChangeSet`
10. `FunctionRegistrySnapshotIdentity`
11. `UdfRegistrationRejection` and `UdfRegistrationRejectionCode`

Runtime behavior exercised locally:

1. successful bind-visible UDF registration advances the immutable registry
   snapshot identity and emits a `RegistryChangeSet`;
2. typed registration rejection preserves the current snapshot identity;
3. same-source registration update replaces callable surface metadata and
   reports changed surface names;
4. UDF unregister by source registration id removes the entry, removes the
   invocation target, advances snapshot identity, and emits removed function ids;
5. descriptor-only registered-external mutation can emit a change set whose
   previous and new snapshot identities are identical and whose
   `descriptor_only_mutation` flag is true;
6. capability overlays remain projections and do not delete registry entries;
7. invocation targets are stored separately from `FunctionEntry` callable
   surface metadata;
8. `ReferenceLike` and host references can be carried as opaque invocation
   target values without materialization or TreeCalc-specific branching.

Compatibility note:

1. Existing W091-era `FunctionRegistry::register_udf(FunctionEntry)` and
   `FunctionRegistry::unregister_udf(function_id)` remain available for current
   OxFml and DNA OneCalc consumers.
2. The new W093 request/result methods are the future-facing mutation contract
   for callers that need snapshot identity, change sets, and typed rejection
   outcomes.

Local evidence:

1. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib`: passed,
   `1307` passed, `1` ignored.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib registry`:
   passed, `24` passed.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --test oxfml_registered_external_interface_integration`:
   passed, `3` passed.
4. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml`: passed.

Open lanes:

1. OxFml W074 formula-call binding, cache invalidation, and registry snapshot
   identity consumption remain downstream-owned.
2. Excel oracle evidence for built-in/UDF/defined-name/defined-name-LAMBDA
   precedence remains open.
3. Source adapters for XLL, VBA, JavaScript custom functions, Automation, and
   registered-external-backed UDFs still need source-specific implementation
   and evidence beyond the source-neutral OxFunc API.
4. Broad UDF execution semantics and host runtime implementation remain out of
   scope for this tranche.

## 2026-05-22 Collision And Name-Precedence Evidence Intake

W093 now has a bounded clean-room evidence intake for the first collision and
name-precedence lanes.

Observed or exercised:

1. OxFml `W074-CALC005-003` / `004` observe that a workbook defined name can
   win over a VBA UDF in both call-callee and non-call bare-name contexts.
2. OxFml `W074-CALC005-001` / `002` observe the contrast that built-in
   `SUM(...)` wins call-callee position while bare `=SUM` resolves to a
   workbook defined name.
3. OxFml `W074-CALC005-012` / `013` observe late UDF registration and removal
   changing formula outcomes through registry/name-world invalidation.
4. OxFunc registry tests exercise same-source UDF update, UDF surface
   collision rejection, unregister change sets, typed rejection without
   snapshot advancement, and descriptor-only mutation without snapshot
   advancement.

Contract consequences:

1. OxFunc registry mutation must not encode formula name/call precedence.
2. Built-in protection, same-source update, UDF-to-UDF collision rejection, and
   typed rejection remain OxFunc registry rules.
3. UDF-vs-defined-name and bare-name/call-callee precedence remain OxFml W074
   rules over the registry snapshot plus host/document name world.
4. Successful bind-visible UDF registration/unregistration remains a registry
   snapshot/change-set event consumed by OxFml invalidation.
5. JavaScript custom-function namespace/collision evidence is still open and
   owned by the JavaScript metadata mapping bead before source-adapter
   promotion.

## 2026-05-22 Registered-External Reconciliation

This contract remains aligned with the existing W046/W052 registered-external
seam.

Rules preserved:

1. `REGISTER.ID` / `CALL` descriptor-only mutation is not ordinary UDF
   registration.
2. Plain `REGISTER.ID` register-id creation does not create a function-help,
   completion, or bind-visible callable function entry.
3. `CALL` invocation by register id or direct target remains a
   registered-external invocation path.
4. A registered-external source may create an ordinary `FunctionRegistry` UDF
   entry only when the host supplies friendly worksheet-visible metadata:
   surface name, arity/signature, callable metadata, source registration id,
   and invocation target descriptor.
5. Descriptor-only mutation uses a targeted-reevaluation change set with
   unchanged semantic registry snapshot identity by default.
6. Bind-visible function registration/unregister remains the case that advances
   `FunctionRegistrySnapshotIdentity` for OxFml bind/editor invalidation.

The current seven-field `RegisteredExternalDescriptor` is sufficient for the
W093 registry-mutation contract; no additional OxFunc descriptor fields are
needed before source-adapter work.

## 2026-05-22 JavaScript Metadata Contract

JavaScript custom-function metadata maps to the current W093 contract without a
new source kind.

Contract additions:

1. `UdfSourceKind::JavaScriptCustomFunction` covers add-in custom functions
   declared by JSON metadata or generated from supported JSDoc tags.
2. `stable_source_registration_id` should combine add-in identity, namespace
   when present, custom-function id, and metadata version/fingerprint.
3. `surface_name` comes from the worksheet-visible function name, while
   `function_id` remains the stable OxFunc registry id for the callable entry.
4. `UdfInvocationTargetDescriptor::JavaScript` carries add-in id, namespace,
   custom-function id, runtime ref, and opaque runtime values.
5. `UdfExecutionProfile.streaming` and `UdfExecutionProfile.cancellable` carry
   the corresponding JavaScript custom-function execution flags; a JS adapter
   must not admit both as simultaneously true for one function.
6. Address, parameter-address, and calling-object requirements are
   caller-context dependencies. They belong in source provenance or future typed
   adapter metadata, while the actual invocation context is supplied by
   OxFml/host runtime.
7. `excludeFromAutoComplete` is editor projection metadata. It does not delete
   the function from the registry or make explicit formula binding invalid.
8. JS custom enums, linked entities, custom data types, and result
   dimensionality are preserved as source metadata until a later typed
   rich-value/result-shape contract admits them.

This closes the W093 metadata-shape question for JavaScript custom functions
only. It does not implement the JavaScript runtime adapter, freeze collision
precedence, or close OxFml registry-backed formula binding/cache invalidation.
