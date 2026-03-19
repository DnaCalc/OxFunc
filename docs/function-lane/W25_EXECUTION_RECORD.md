# W25 Execution Record - Misc Add-In And Dynamic-Array Outlier Classification

Status: `complete-provisional`
Workset: `W25`

## 1. Purpose
Record the classification closure for the two `W24` extracted outliers `EUROCONVERT` and `RANDARRAY`.

## 2. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W25` scope.

## 3. Findings
1. `EUROCONVERT` is not treated as an ordinary current-host worksheet function on the reference baseline.
2. Microsoft support explicitly states that `EUROCONVERT` returns `#NAME?` unless the Euro Currency Tools Add-in is installed and loaded.
3. OxFunc therefore classifies `EUROCONVERT` as an external add-in-owned surface and does not claim an in-core implementation target for it now.
4. The second inventory member was an internal typo: `RANDARRA` is corrected to `RANDARRAY`.
5. `RANDARRAY` remains deferred to future dynamic-array/version-gated work rather than this packet.

## 4. Source Basis
1. Native W24 replay already pinned `EUROCONVERT(...) -> #NAME?` and `RANDARRAY() -> #NAME?` on the current host baseline.
2. Official Microsoft support page for `EUROCONVERT` confirms the add-in requirement.

## 5. Reconciled Outcome
1. `W25` is now a classification/reconciliation packet, not an implementation packet.
2. `EUROCONVERT` is declared external/deferred.
3. `RANDARRAY` spelling is corrected and future ownership is deferred outside `W25`.

## 6. Verification
1. Re-read the native replay evidence already recorded in `W24 Batch 15`.
2. Re-checked the official Microsoft support page for `EUROCONVERT` add-in requirements.

## 7. Standing
1. `W25` is closed.
2. This closure does not make any implementation claim for `EUROCONVERT`.
3. This closure does not resolve future `RANDARRAY` semantic work; it only removes the typo and wrong packet ownership.
