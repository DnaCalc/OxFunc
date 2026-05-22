# HO-FN-014 - UDF registry mutation and name-resolution invalidation

Status: filed
Direction: OxFunc -> OxFml
Source workset: W093
Target: OxFml formula binding, name-resolution, editor-help, and host cache surfaces
Filed date: 2026-05-04
Related prior handoff: HO-FN-011

## Purpose

Open the OxFml-facing design seam for UDF registration and function-registry
mutation after W091 made OxFunc the canonical runtime function registry owner.

The key point is separation of ownership:

1. OxFunc owns callable function registry entries and UDF mutations.
2. OxFml owns formula parse/bind, name resolution, and bind/editor cache
   invalidation.
3. Workbook/sheet defined names are formula/document environment state, not
   OxFunc function-registry state.

## Proposed shared direction

OxFunc should expose immutable registry-backed snapshot identities and change
sets for successful bind-visible UDF registration/unregistration.

OxFml should:

1. bind function calls against an OxFunc registry view or registry-derived
   snapshot,
2. include the registry snapshot identity in bind and semantic-plan cache keys,
3. invalidate bind/editor-help artifacts affected by registry mutation,
4. allow formulas previously producing `#NAME?` to become bindable after UDF
   registration,
5. treat unregister or capability denial as a possible `#NAME?` or
   capability-blocked transition for previously bound formulas,
6. keep workbook/sheet defined names in the formula-name environment rather
   than moving them into OxFunc,
7. distinguish bind-visible UDF registration from `REGISTER.ID` / `CALL`
   descriptor-only mutation.

## Source registration lanes

Expected source lanes:

1. XLL `xlfRegister`,
2. host-discovered VBA public module functions,
3. JavaScript custom-function manifest/JSON metadata,
4. Automation or host-registered functions,
5. worksheet `REGISTER.ID` / `CALL` registered-external paths.

`REGISTER.ID` should remain a registered-external lookup lane unless the host
also supplies friendly worksheet-visible UDF metadata.

Plain `REGISTER.ID` / `CALL` descriptor mutation should default to targeted
reevaluation and should not create editor-completion or bind-visible ordinary
function entries.

## Requested OxFml response

Please identify:

1. current bind/editor cache artifacts that need registry snapshot identity
   keys,
2. current formula-name precedence rules that affect UDF-vs-defined-name
   collisions,
3. any OxFml-only metadata needed in an OxFunc `RegistryChangeSet`,
4. whether `#NAME?` recovery after late UDF registration needs a dedicated
   invalidation event distinct from ordinary formula text change,
5. the concrete path to migrate formula-call binding/evaluation from static
   built-in metadata lookup to registry-backed lookup for UDF-aware contexts.

Status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`: OxFml acknowledgement, shared invalidation design,
   registered-external reconciliation, formula-call registry lookup migration,
   and first seam tests.

## 2026-05-22 OxFunc API Update

OxFunc has added the repo-local W093 mutation packet surface in
`oxfunc_core::registry`:

1. `FunctionRegistrySnapshotIdentity`
2. `RegistryChangeSet`
3. `UdfRegistrationRequest`
4. `UdfRegistrationResult`
5. `UdfUnregistrationResult`
6. `UdfInvocationTargetDescriptor`
7. typed rejection outcomes

Clarifications for OxFml W074:

1. Successful bind-visible UDF registration, same-source update, and unregister
   advance the semantic registry snapshot identity and return a
   `RegistryChangeSet`.
2. Rejected mutations return typed rejection detail and do not advance semantic
   snapshot identity.
3. Descriptor-only `REGISTER.ID` / `CALL` mutation returns a change set with
   unchanged snapshot identity and `descriptor_only_mutation = true`; this
   supports targeted reevaluation instead of broad rebinding by default.
4. Callable worksheet surface metadata remains in `FunctionEntry`; source
   invocation routing remains in `UdfInvocationTargetDescriptor`.
5. `ReferenceLike` and host reference carriers remain opaque inside invocation
   target descriptors.
6. No TreeCalc-specific name/call precedence branch has been added in OxFunc.

Local checks:

1. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --lib registry`:
   passed, `24` passed.
2. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml --test oxfml_registered_external_interface_integration`:
   passed.
3. `cargo test --manifest-path crates\oxfunc_core\Cargo.toml`: passed.

Remaining OxFml asks:

1. consume the snapshot identity/change-set surface in W074 bind/editor/runtime
   cache invalidation;
2. keep formula-call binding migration registry-backed for UDF-aware contexts;
3. record any OxFml-only metadata needed beyond the current
   `RegistryChangeSet`;
4. keep TreeCalc-specific namespace behavior out of the shared rule until Excel
   precedence evidence or a separate host extension packet justifies it.
