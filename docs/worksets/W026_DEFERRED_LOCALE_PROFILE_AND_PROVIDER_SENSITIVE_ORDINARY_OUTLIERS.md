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

## 5. Execution Outcome
1. `W26` now owns a dedicated current-host characterization packet:
   - `docs/function-lane/W26_HOST_PROFILE_PROVIDER_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W26_RUNTIME_REQUIREMENTS.md`
   - `tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
   - `.tmp/w26-host-profile-provider-results.csv`
2. The packet shows two distinct successor seam classes:
   - locale/profile-sensitive text/number functions -> `W30`
   - provider-language functions -> `W31`
3. `W26` therefore closes by reconciliation rather than by pretending these functions are solved on the current boundary.

## 6. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W26` scope after reconciliation
