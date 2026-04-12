# WORKSET - XIRR Solver Precision Reconciliation (W087)

## 1. Purpose
Own the bounded local reconciliation where current `XIRR` solver output drifts
from live Excel `Value2` on a pinned current-baseline cashflow/date witness.

## 2. Why This Packet Exists
Direct local replay and live Excel replay on 2026-04-10 narrowed a real local
solver-precision lane:
1. `XIRR({-10000,2750,4250,3250,2750},{44927,45108,45292,45473,45658})`:
   - OxFunc -> `0.24449183218286558`
   - Excel `Value2` -> `0.24449183344840997`
2. the mismatch is not a published-display rounding issue; it sits in the local
   iterative solve path in `cashflow_rate_family.rs`,
3. this lane is distinct from the earlier `RATE` omitted-guess convergence bug
   under `W081`,
4. earlier `XIRR` evidence did not pin this exact witness at current-baseline
   `Value2` precision, so the reopened lane needs a bounded local owner.

## 3. Provenance
1. user direction on 2026-04-10
2. direct local OxFunc replay on 2026-04-10
3. live Excel `Value2` replay on 2026-04-10
4. `docs/bugs/reports/BUGREP-FUNC-018_xirr_solver_precision_drift.md`
5. `docs/bugs/streams/BUG-FUNC-014_xirr_solver_precision_drift.md`
6. `crates/oxfunc_core/src/functions/cashflow_rate_family.rs`

## 4. Scope
In scope:
1. record the bounded `XIRR` precision drift as a canonical bug stream and
   owner workset,
2. characterize the current iterative-path drift against the pinned Excel
   witness,
3. reconcile the local implementation against that witness,
4. add focused exact-value regression coverage,
5. reconcile `W051` and adjacent truth surfaces honestly.

Out of scope:
1. reopening `RATE` omitted-guess behavior already owned by `W081`,
2. blanket reopening of `IRR` or `XNPV` without direct replay evidence,
3. display-policy handling for finance last-digit rows outside the pinned
   `XIRR` witness.

## 5. Initial Epic Lanes
1. bug intake and owner registration
2. solver-precision replay characterization
3. local iterative-path reconciliation
4. focused validation
5. W51 truth reconciliation
6. bounded adjacent financial-iteration review framing

## 6. Closure Condition
`W087` is complete for declared scope only when:
1. the pinned `XIRR` witness matches live Excel locally,
2. focused exact-value validation is recorded,
3. bounded adjacent multi-cashflow positive-root replay is either aligned or
   explicitly parked as still-open scope,
4. `W051` and related truth surfaces no longer overclaim the reopened row.

## 7. Current Reading
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none
