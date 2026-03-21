# WORKSET - Reference Metadata And Formula Visibility Functions (W40)

## 1. Purpose
Open the packet for reference-sensitive metadata and formula-visibility functions that were classified as interesting from the start.

This packet targets the part of Excel that reveals workbook/reference identity rather than just computing over values.

## 2. Provenance
Opened after the interesting-function review separated this cluster from:
1. the host/database residual packet `W023`,
2. the lambda/helper packet `W038`,
3. the dynamic-array shaping packet `W039`.

Relevant context:
1. `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv`
2. `docs/worksets/W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
3. `docs/worksets/W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W40_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_INVENTORY.csv`

Current total:
1. `5` functions.

Members:
1. `ADDRESS`
2. `AREAS`
3. `FORMULATEXT`
4. `SHEET`
5. `SHEETS`

## 4. Why This Packet Matters
1. These functions expose reference identity, sheet/workbook identity, and stored-formula visibility.
2. They pressure the seam between pure value semantics and host/grid metadata.
3. They are a strong complement to `CELL` / `INFO` without being identical to the host-query packet.
4. They are likely to clarify where direct-cell binding and formula-storage truth belong in the OxFml/OxFunc split.

## 5. In Scope
1. empirical characterization of admitted current-baseline semantics,
2. contract work for reference metadata, formula visibility, and sheet-identity queries,
3. explicit seam statements for what host metadata must be supplied,
4. runtime work for the admitted slice where honest on the current boundary,
5. scenario manifests, runtime requirements, execution record, and evidence registry rows.

## 6. Out Of Scope
1. `ISFORMULA`, which remains owned by `W023`,
2. `CELL` / `INFO`, already handled in `W015`,
3. database functions from `W023`,
4. general implicit-intersection/operator work from `W014`.

## 7. Gate Criteria
This workset can only be reported `scope_complete` when:
1. all five members have reproducible native evidence for the admitted slice,
2. the packet explicitly states what is pure reference semantics versus what needs host/grid metadata,
3. runtime and seam artifacts are honest for the admitted slice,
4. no known function-semantic gap remains in declared current-baseline scope for the admitted slice.

## 8. Current Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W040` scope
