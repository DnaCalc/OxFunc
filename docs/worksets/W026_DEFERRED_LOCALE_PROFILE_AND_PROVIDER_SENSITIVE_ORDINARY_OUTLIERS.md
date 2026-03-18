# WORKSET - Deferred Locale/Profile And Provider-Sensitive Ordinary Outliers (W26)

## 1. Purpose
Own the `W24` rows that proved not to be ordinary host-independent pure functions on the current baseline because their semantics depend on locale/profile availability or an external provider surface.

## 2. Provenance
Opened by `W24` extraction after native replay on `2026-03-18`.

Source artifacts:
1. `docs/worksets/W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
2. `CURRENT_BLOCKERS.md`
3. `docs/function-lane/W26_DEFERRED_LOCALE_PROFILE_AND_PROVIDER_SENSITIVE_ORDINARY_INVENTORY.csv`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W26_DEFERRED_LOCALE_PROFILE_AND_PROVIDER_SENSITIVE_ORDINARY_INVENTORY.csv`

Current total:
1. `5` extracted functions.

Members:
1. `ASC`
2. `DBCS`
3. `JIS`
4. `NUMBERVALUE`
5. `TRANSLATE`

## 4. Entry Criteria
Functions belong in `W26` only if native replay showed they depend materially on:
1. host locale/profile availability,
2. omitted-separator locale defaults,
3. external translation/provider behavior.

## 5. First Work Streams
1. characterize `ASC` / `DBCS` / `JIS` availability and width-conversion behavior by host/profile,
2. characterize `NUMBERVALUE` omitted-separator defaults by host locale/profile,
3. characterize `TRANSLATE` provider-bound behavior and whether it belongs with a broader host/provider seam.

## 6. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no dedicated replay packet exists yet,
   - locale/profile matrix and provider behavior are not yet closure-grade.
