# WORKSET - Deferred Provider Language Functions (W31)

## 1. Purpose
Own provider-bound language functions whose observed worksheet behavior depends on an external service/provider seam rather than a pure local kernel.

## 2. Provenance
Opened by `W26` scope reconciliation on `2026-03-18`.

Source artifacts:
1. `docs/function-lane/W26_SCOPE_RECONCILIATION.csv`
2. `CURRENT_BLOCKERS.md`
3. `docs/function-lane/W31_DEFERRED_PROVIDER_LANGUAGE_FUNCTIONS_INVENTORY.csv`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W31_DEFERRED_PROVIDER_LANGUAGE_FUNCTIONS_INVENTORY.csv`

Current total:
1. `1` deferred function.

Members:
1. `TRANSLATE`

## 4. First Work Streams
1. classify provider-unavailable / provider-busy / same-language pass-through semantics,
2. align with the broader provider-sensitive host packet direction in `W023`,
3. decide whether `TRANSLATE` and `DETECTLANGUAGE` share one capability surface.

## 5. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W31` scope after reconciliation; successor ownership moved to `W036`.
