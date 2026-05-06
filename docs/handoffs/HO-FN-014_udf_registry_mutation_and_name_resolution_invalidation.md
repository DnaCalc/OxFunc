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
