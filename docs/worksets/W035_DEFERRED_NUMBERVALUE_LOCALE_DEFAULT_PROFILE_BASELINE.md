# WORKSET - Deferred NUMBERVALUE Locale-Default Profile Baseline (W35)

## 1. Purpose
Own the locale/profile-sensitive omitted-default behavior of `NUMBERVALUE`.

## 2. Provenance
Opened by `W30` scope reconciliation on `2026-03-19`.

Source artifacts:
1. `docs/function-lane/W30_SCOPE_RECONCILIATION.csv`
2. `docs/function-lane/W26_EXECUTION_RECORD.md`
3. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W35_DEFERRED_NUMBERVALUE_LOCALE_DEFAULT_INVENTORY.csv`

Current total:
1. `1` deferred function.

Members:
1. `NUMBERVALUE`

## 4. First Work Streams
1. characterize omitted-default decimal/group separator behavior against locale/profile inputs,
2. define the minimum typed locale-default seam OxFunc needs from OxFml/FEC,
3. keep explicit-separator lanes separate from omitted-default lanes in future closure claims.

## 5. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - omitted-default parsing policy is not yet pinned beyond the current host baseline,
   - no typed locale-default profile contract is pinned yet.
