# WORKSET - Normal-Distribution Exact-Value Accuracy (W086)

## 1. Purpose
Own the bounded local reconciliation where current `NORM.DIST` and `NORM.INV`
approximations drift from live Excel `Value2` on pinned current-baseline exact-
value witnesses.

## 2. Why This Packet Exists
Direct local replay and live Excel replay on 2026-04-10 narrowed a real local
accuracy lane:
1. `NORM.DIST(0,0,1,TRUE)`:
   - OxFunc -> `0.49999998499999976`
   - Excel `Value2` -> `0.5`
2. `NORM.INV(0.975,0,1)`:
   - OxFunc -> `1.9599639471668913`
   - Excel `Value2` -> `1.9599639845400536`
3. the mismatch is not a display-policy issue; it sits in the local
   approximation path in `normal_log_family.rs`,
4. earlier W062 evidence pinned rounded witnesses rather than exact current-
   baseline `Value2` parity, so the reopened lane needs a bounded local owner.

## 3. Provenance
1. user direction on 2026-04-10
2. direct local OxFunc replay on 2026-04-10
3. live Excel `Value2` replay on 2026-04-10
4. `docs/bugs/reports/BUGREP-FUNC-017_normal_distribution_exact_value_drift.md`
5. `docs/bugs/streams/BUG-FUNC-013_normal_distribution_exact_value_accuracy_gap.md`
6. `crates/oxfunc_core/src/functions/normal_log_family.rs`

## 4. Scope
In scope:
1. record the bounded `NORM.DIST` / `NORM.INV` exact-value drift as a
   canonical bug stream and owner workset,
2. characterize the current local approximation gap against the pinned Excel
   witnesses,
3. reconcile the local implementation against those exact witnesses,
4. add focused exact-value regression coverage,
5. reconcile `W051` and adjacent truth surfaces honestly.

Out of scope:
1. blanket reopening of the full W062 statistical-distribution family,
2. display-policy handling for finance last-digit rows,
3. widening beyond the directly replayed helper-adjacent rows without new
   evidence.

## 5. Initial Epic Lanes
1. bug intake and owner registration
2. exact-value replay characterization
3. local approximation reconciliation
4. focused validation
5. W51 truth reconciliation
6. bounded adjacent helper review framing

## 6. Closure Condition
`W086` is complete for declared scope only when:
1. the pinned `NORM.DIST` and `NORM.INV` witnesses match live Excel locally,
2. focused exact-value validation is recorded,
3. bounded helper-adjacent replay is either aligned or explicitly reopened,
4. `W051` and related truth surfaces no longer overclaim the reopened rows.

## 7. Current Reading
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none
