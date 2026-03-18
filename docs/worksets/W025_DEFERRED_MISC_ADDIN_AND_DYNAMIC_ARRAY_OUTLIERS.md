# WORKSET - Deferred Misc Add-In And Dynamic-Array Outliers (W25)

## 1. Purpose
Own the two `W24` outliers that proved not to be ordinary current-host worksheet functions on the reference baseline.

## 2. Provenance
Opened by `W24 Batch 15` extraction.

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
2. `RANDARRA`

## 4. Entry Criteria
Functions belong in `W25` only if current-baseline native replay proved that they do not behave as ordinary always-present worksheet functions on the current host baseline.

## 5. First Work Streams
1. characterize `EUROCONVERT` host/add-in availability and any legacy add-in activation requirements,
2. reconcile `RANDARRA` inventory naming against `RANDARRAY`,
3. decide whether each function belongs to a host/add-in seam or a dynamic-array/version-gated workset.

## 6. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no dedicated successor implementation packet exists yet,
   - host/add-in and version-gating semantics are not yet characterized.
