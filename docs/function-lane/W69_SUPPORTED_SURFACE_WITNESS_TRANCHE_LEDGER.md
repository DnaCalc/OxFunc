# W69 Supported Surface Witness Tranche Ledger

This ledger freezes the first full-coverage rollout partition for the remaining
supported non-deferred W069 surface.

## Current Counts
- supported non-deferred rows: 517
- witness-covered rows: 10
- remaining witness gap rows: 507

## Tranches
1. T1 ordinary extracted non-operator rows
   - selection rule: metadata_status=function_meta_extracted AND special_interface_kind=ordinary AND category!=Operators
   - count: 201
2. T2 ordinary curated non-operator rows
   - selection rule: metadata_status=function_meta_curated AND special_interface_kind=ordinary AND category!=Operators
   - count: 267
3. T3 operator surface
   - selection rule: category=Operators
   - count: 22
4. T4 special extracted surface
   - selection rule: metadata_status=function_meta_extracted AND special_interface_kind!=ordinary
   - count: 5
5. T5 special curated surface
   - selection rule: metadata_status=function_meta_curated AND special_interface_kind!=ordinary
   - count: 12

## Closure Rule
W069 may claim full supported-surface witness coverage only when:
1. every supported non-deferred row in the parked baseline is either witness-covered or assigned to one of the frozen tranches above,
2. every row assigned to a tranche has either a generated/curated V2 witness row or an explicit dependency-block record,
3. the final coverage ledger reconciles to the current V1 export and the W050 deferred inventory,
4. any dependency-gated special-interface row records the retained live authority it depends on,
5. the published coverage report distinguishes witness coverage from ordinary V1 support status.

## Inventory
See [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv) for the reviewable row-level tranche assignment.
