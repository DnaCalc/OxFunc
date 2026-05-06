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
