# WORKSET - Deferred Width Conversion Host/Profile Capability Baseline (W34)

## 1. Purpose
Own the width-conversion functions whose current-boundary behavior depends on host/profile availability and conversion policy rather than a stable pure local kernel.

## 2. Provenance
Opened by `W30` scope reconciliation on `2026-03-19`.

Source artifacts:
1. `docs/function-lane/W30_SCOPE_RECONCILIATION.csv`
2. `docs/function-lane/W26_EXECUTION_RECORD.md`
3. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W34_DEFERRED_WIDTH_CONVERSION_HOST_PROFILE_INVENTORY.csv`

Current total:
1. `3` deferred functions.

Members:
1. `ASC`
2. `DBCS`
3. `JIS`

## 4. First Work Streams
1. define the host/profile availability matrix for width-conversion functions,
2. decide whether capability admission lives in library context, runtime capability views, or both,
3. characterize the profile-sensitive pass-through and unavailable lanes against at least one additional host/profile when available.

## 5. Status
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no typed width-conversion capability seam is pinned yet,
   - no cross-profile matrix exists yet beyond the current host baseline.
