# W69 Seam-Heavy Supported Surface Ledger

This ledger records the retained special-interface remainder set for W069.

## Counts
- seam-heavy remaining rows: 17

## Inventory Groups
1. callable helper rows: 7
   - FUNC.LAMBDA
   - FUNC.BYCOL
   - FUNC.BYROW
   - FUNC.ISOMITTED
   - FUNC.MAKEARRAY
   - FUNC.MAP
   - FUNC.REDUCE
   - FUNC.SCAN
2. host / presentation / locale rows: 5
   - FUNC.RTD
   - FUNC.NUMBERVALUE
   - FUNC.NOW
   - FUNC.TODAY
   - FUNC.ASC
   - FUNC.DBCS
   - FUNC.JIS
3. registered-external rows: 2
   - FUNC.CALL
   - FUNC.REGISTER.ID

## Dependency Gates
1. W049 retains the callable-helper, presentation, locale, and host-profile attachment surface.
2. W043 retains the host-subscription provider seam for RTD.
3. W041 retains the registered-external invocation/registration seam.

## Inventory
See [W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv](W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv) for the row-level gate map.
