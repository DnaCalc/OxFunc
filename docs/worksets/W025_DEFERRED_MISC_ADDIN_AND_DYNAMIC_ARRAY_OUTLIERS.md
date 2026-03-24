# WORKSET - Deferred Misc Add-In And Dynamic-Array Outliers (W25)

## 1. Purpose
Classify and reconcile the two `W24` outliers that proved not to be ordinary current-host worksheet functions on the reference baseline.

## 2. Provenance
Opened by `W24 Batch 15` extraction.

Backlog ownership note:
1. `W025` remains provenance-only for the earlier `EUROCONVERT` / `RANDARRAY` classification split.
2. Current-version deferred ownership now sits in `W050` for `EUROCONVERT`.
3. Current-version in-scope not-complete ownership now sits in `W051` for `RANDARRAY`.

Source artifacts:
1. `docs/worksets/W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
2. `docs/function-lane/W24_BATCH15_MISC_ORDINARY_CONVERSION_EXECUTION_RECORD.md`
3. `docs/function-lane/W25_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_INVENTORY.csv`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W25_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_INVENTORY.csv`

Current total:
1. `2` extracted functions.

Members:
1. `EUROCONVERT`
2. `RANDARRAY`

## 4. Entry Criteria
Functions belong in `W25` only if current-baseline native replay proved that they do not behave as ordinary always-present worksheet functions on the current host baseline.

## 5. Resolution Summary
1. `EUROCONVERT` is now classified as an external add-in-owned worksheet surface on the current Excel baseline.
2. Official Microsoft support states that it returns `#NAME?` unless the Euro Currency Tools Add-in is installed and loaded.
3. OxFunc will not implement `EUROCONVERT` now as an in-core ordinary function claim.
4. The second inventory member was a typo: `RANDARRA` is corrected to `RANDARRAY`.
5. `RANDARRAY` remains deferred to future dynamic-array/version-gated work; this packet only closes the classification and naming issue.

## 6. Artifacts
1. `docs/function-lane/W25_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_INVENTORY.csv`
2. `docs/function-lane/W25_EXECUTION_RECORD.md`

## 7. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W25` scope; future implementation ownership, if any, sits outside this classification packet.
