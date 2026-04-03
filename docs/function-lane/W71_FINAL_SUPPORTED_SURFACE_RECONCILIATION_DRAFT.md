# W71 Final Supported-Surface Reconciliation Draft

This draft defines the publication shape for the eventual W071 closure record.
It does not claim the witness surface is fully populated yet.

## 1. Purpose
W071 needs a final reconciliation artifact that can prove, row by row, that
every supported non-deferred surface member is either:
1. witness-covered,
2. queued in a deterministic tranche,
3. dependency-blocked by a retained live authority.

This draft is the reviewable shape for that final publication surface.

## 2. Current Surface
The parked non-deferred supported surface remains:
1. `517` supported rows total,
2. `10` witness-covered rows in the seeded V2 surface,
3. `507` rows still awaiting W071 witness population.

The sealed tranche breakdown is:
1. `201` ordinary extracted non-operator rows,
2. `267` ordinary curated non-operator rows,
3. `22` operator rows,
4. `5` special extracted rows,
5. `12` special curated rows.

The seam-heavy dependency-gated subset remains:
1. `17` rows total,
2. gated by retained live authorities `W041`, `W043`, and `W049`.

## 3. Draft Publication States
The eventual W071 final reconciliation should label each row as one of:
1. `witness_covered`
2. `queued_in_tranche`
3. `dependency_blocked`

That publication rule matches the frozen W069 control surface and keeps the
final report distinguishable from the parked `V1` support export.

## 4. Source Surfaces
This draft is anchored to:
1. [W71_FROZEN_GAP_RECONCILIATION.md](W71_FROZEN_GAP_RECONCILIATION.md)
2. [W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md](W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md)
3. [W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv](W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv)
4. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md)
5. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv)
6. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md](W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md)
7. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv](W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv)
8. [W69_FINAL_SUPPORTED_SURFACE_COVERAGE_RECONCILIATION.json](W69_FINAL_SUPPORTED_SURFACE_COVERAGE_RECONCILIATION.json)
9. [W69_FINAL_SUPPORTED_SURFACE_COVERAGE_LEDGER.md](W69_FINAL_SUPPORTED_SURFACE_COVERAGE_LEDGER.md)

## 5. Closure Rule
W071 can only close once the final reconciliation surface is populated with:
1. actual witness rows for the supported non-deferred surface, or
2. explicit dependency-gated witness records for the retained seam-heavy rows.

This draft is therefore the next control checkpoint, not the terminal proof.

