# WORKSET - Deferred Provider Language Capability Baseline (W36)

## 1. Purpose
Own provider-bound language functions whose current-boundary behavior depends on external language services rather than pure local kernels.

## 2. Provenance
Opened by `W31` scope reconciliation on `2026-03-19`.

Source artifacts:
1. `docs/function-lane/W31_SCOPE_RECONCILIATION.csv`
2. `docs/function-lane/W26_EXECUTION_RECORD.md`
3. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W36_DEFERRED_PROVIDER_LANGUAGE_CAPABILITY_INVENTORY.csv`

Current total:
1. `1` deferred function.

Members:
1. `TRANSLATE`

## 4. First Work Streams
1. define same-language pass-through, provider-busy, and provider-unavailable classifications,
2. decide whether `TRANSLATE` and `DETECTLANGUAGE` share one capability family,
3. keep provider-state failures separate from library-context availability truth.

## 5. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no provider-language capability contract is pinned yet,
   - no additional provider-state matrix exists yet beyond the current host baseline.
