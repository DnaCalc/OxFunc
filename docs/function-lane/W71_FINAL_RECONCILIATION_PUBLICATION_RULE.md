# W71 Final Reconciliation Publication Rule

This note defines how W71 will publish the final supported-surface witness
reconciliation once the remaining batches are exhausted.

## 1. Publication Inputs
The final reconciliation must be computed from:
1. [W71_FROZEN_GAP_RECONCILIATION.md](W71_FROZEN_GAP_RECONCILIATION.md)
2. [W71_FROZEN_GAP_RECONCILIATION.csv](W71_FROZEN_GAP_RECONCILIATION.csv)
3. [W71_TRANCHE_CONTROL_RULE.md](W71_TRANCHE_CONTROL_RULE.md)
4. [W71_TRANCHE_REGISTER.csv](W71_TRANCHE_REGISTER.csv)
5. the emitted tranche JSON batches under `OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_W71_*`

## 2. Publication Rule
The final reconciliation publishes only when:
1. every supported non-deferred row has a witness payload or an explicit
   dependency-gated witness record,
2. the tranche counts sum exactly to the frozen supported-surface total,
3. the remaining gap ledger is empty for the parked baseline,
4. the downstream reading guides point at the populated witness surface rather
   than the frozen framework ledgers.

## 3. Current Draft Shape
The draft shape remains:
1. supported rows: `517`
2. witness-covered rows: `517`
3. remaining supported rows: `0`
4. dependency-gated seam-heavy rows retained behind `W041`, `W043`, and `W049`

This note is the publication policy only. It does not claim final closure.
