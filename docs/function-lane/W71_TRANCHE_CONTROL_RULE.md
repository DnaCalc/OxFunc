# W71 Tranche Control Rule

This note freezes the remaining supported-surface rollout rule for W071.

## 1. Authoritative Inputs
The control rule reads from:
1. [W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md](W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md)
2. [W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv](W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv)
3. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv)
4. [W71_FROZEN_GAP_RECONCILIATION.md](W71_FROZEN_GAP_RECONCILIATION.md)

## 2. Deterministic Rule
W71 executes the supported-surface rollout in tranche order:
1. `T1` ordinary extracted non-operator rows
2. `T2` ordinary curated non-operator rows
3. `T3` operator surface
4. `T4` special extracted surface, dependency-gated by retained live authorities
5. `T5` special curated surface, dependency-gated by retained live authorities

Within each tranche, rows are consumed in register sequence order and closed as
single bead batches. A batch must reconcile to exactly one tranche slice and
must keep `surface_stable_id` stable.

## 3. Current Known State
The current parked baseline remains:
1. `517` witness-covered rows
2. `0` remaining supported rows
3. `17` seam-heavy rows retained behind live authorities `W041`, `W043`, and `W049`

## 4. Closure Rule
A W71 tranche bead may close only when:
1. the tranche slice has actual witness rows or explicit dependency-gated witness records,
2. the count reconciliation matches the frozen W69 gap inventory,
3. the function-lane index and W071 workset references point at the emitted artifact.
