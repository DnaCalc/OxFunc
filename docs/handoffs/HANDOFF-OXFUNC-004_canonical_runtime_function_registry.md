# HANDOFF-OXFUNC-004 - Canonical Runtime Function Registry

Direction: `DnaOneCalc->OxFunc`

Status: `acknowledged`

Filed date: `2026-05-03`

Acknowledged date: `2026-05-03`

Source note:
1. `../DnaOneCalc/docs/HANDOFF_OXFUNC_CANONICAL_FUNCTION_REGISTRY.md`

Target workset:
1. `W091`

## 1. Intake Summary

DNA OneCalc reported that signature help can display incorrect function arity
because function-list and arity/signature truth is being surfaced through
host-built snapshot strings rather than an OxFunc-owned runtime registry.

The concrete observed symptom was `NOW` rendering as a variadic signature even
though OxFunc metadata defines `NOW` as arity `0`.

## 2. OxFunc Acknowledgement

OxFunc accepts the architectural direction:
1. OxFunc must own the only comprehensive function registry.
2. Function entries must include real parameter descriptors, not synthesized
   editor fallbacks.
3. UDF registration must mutate the registry entry set.
4. capability overlays must project availability without rewriting base catalog
   truth.
5. downstream consumers must query OxFunc registry truth or an OxFunc-generated
   registry snapshot rather than maintaining a parallel list.

## 3. Local Processing

Local OxFunc processing artifacts:
1. `docs/worksets/W091_CANONICAL_RUNTIME_FUNCTION_REGISTRY.md`
2. `docs/function-lane/OXFUNC_CANONICAL_RUNTIME_FUNCTION_REGISTRY_CONTRACT.md`
3. `docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md`
4. `.beads/` rollout beads for `W091`

## 4. Open Lanes

execution_state: `in_progress`

scope_completeness: `scope_partial`

target_completeness: `target_partial`

integration_completeness: `partial`

open_lanes:
1. implement registry API,
2. populate all built-in parameter descriptors,
3. implement UDF registration and capability overlays,
4. migrate OxFml and hosts away from duplicate lists,
5. record host and wasm validation evidence.

## 5. Non-Completion Note

This acknowledgement does not close the registry work.

The DnaOneCalc note has been accepted as a valid OxFunc workset input, and the
substantive implementation remains owned by `W091`.

## 2026-05-03 W091 processing update

The inbound DNA OneCalc report has been processed into OxFunc W091. OxFunc now provides the runtime registry API and generated parameter metadata needed to replace host-local comprehensive function lists. DNA OneCalc-facing migration instructions are in `docs/handoffs/HO-FN-012_dnaonecalc_registry_consumption.md`.
