# WORKSET - Deferred Locale/Profile-Sensitive Text And Number Functions (W30)

## 1. Purpose
Own the `W26` functions whose semantics depend materially on host locale/profile availability or omitted-default locale parsing behavior.

## 2. Provenance
Opened by `W26` scope reconciliation on `2026-03-18`.

Source artifacts:
1. `docs/function-lane/W26_SCOPE_RECONCILIATION.csv`
2. `CURRENT_BLOCKERS.md`
3. `docs/function-lane/W30_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_INVENTORY.csv`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W30_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_INVENTORY.csv`

Current total:
1. `4` deferred functions.

Members:
1. `ASC`
2. `DBCS`
3. `JIS`
4. `NUMBERVALUE`

## 4. First Work Streams
1. define the host/profile availability matrix for width-conversion functions,
2. define the omitted-default separator semantics and locale-profile inputs for `NUMBERVALUE`,
3. decide whether these functions remain OxFunc-owned pure kernels with profile-gated admission or move behind a typed locale/profile capability seam.

## 5. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no host/profile matrix is pinned yet,
   - no honest current-boundary runtime strategy is agreed yet.
