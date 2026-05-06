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
