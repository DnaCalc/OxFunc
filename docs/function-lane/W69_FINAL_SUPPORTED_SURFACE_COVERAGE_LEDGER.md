# W69 Final Supported Surface Coverage Ledger

This note defines the publication shape for the full-supported-surface W069
program.

## 1. Current Coverage State
The parked non-deferred supported surface is:
1. `517` supported rows total,
2. `10` rows already witness-covered in the current V2 seed surface,
3. `507` rows still awaiting V2 witness coverage.

The remaining `507` rows are partitioned as:
1. `201` ordinary extracted non-operator rows,
2. `267` ordinary curated non-operator rows,
3. `22` operator rows,
4. `5` special extracted rows,
5. `12` special curated rows.

The seam-heavy remainder set is:
1. `17` rows,
2. gated by retained live authorities `W041`, `W043`, and `W049`.

## 2. Source Ledgers
The final coverage ledger is reconciled against:
1. [W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md](W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md)
2. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md)
3. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md](W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md)

The row-level inventory sources are:
1. [W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv](W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv)
2. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv)
3. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv](W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv)

## 3. Publication Rule
The published W069 coverage report should distinguish:
1. `witness_covered`
2. `ordinary_seeded`
3. `queued_in_tranche`
4. `dependency_blocked`

That means:
1. witness coverage is not the same thing as `V1` support status,
2. tranche assignment is not the same thing as final witness coverage,
3. dependency gating must remain visible for seam-heavy rows,
4. ordinary seed artifacts must remain clearly labeled as tranche seeds until
   their later enrichment beads close them.

## 4. Remaining Support-Surface Closure Rule
The remaining supported non-deferred rows may only be considered covered when
each row is assigned to exactly one of these states:
1. `witness_covered`
2. `queued_in_tranche`
3. `dependency_blocked`

That closure rule means:
1. no supported row may be left outside the final coverage ledger,
2. every queued row must point at a deterministic tranche or generator path,
3. every dependency-blocked row must name the retained live authority that
   keeps the gate open,
4. the final published ledger must reconcile exactly to the parked `V1`
   support surface and the deferred `W050` inventory.

## 5. Closure Rule
W069 may claim full supported-surface witness coverage only when:
1. every supported non-deferred row in the parked baseline is either witness
   covered, queued in a frozen tranche, or explicitly dependency-blocked,
2. every queued row has a deterministic generator or curated authoring rule
   attached to it,
3. every dependency-blocked seam row names the retained live authority that
   holds the gate open,
4. the published coverage report reconciles exactly to the current `V1` export
   and the deferred `W050` inventory,
5. no live row remains outside the coverage ledger.

## 6. Notes
This ledger is a publication rule, not a second catalog.
It exists so the `V2` rollout can widen from seeds to the full supported
surface without collapsing witness coverage into support status.
