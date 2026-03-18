# WORKSET - Deferred Host, Metadata, and Database Functions (W23)

## 1. Purpose
Own the low-interest residuals that are not honestly "pure" function work on the current OxFunc boundary.

This packet extracts functions that depend on one or more of:
1. host/query state,
2. cell metadata rather than only cell value,
3. record-style database criteria evaluation that is better treated as a separate evaluator/shape seam than as ordinary scalar/range hardening.

## 2. Provenance
Opened as a successor extraction from `W17`.

Source artifacts:
1. `docs/worksets/W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md`
2. `docs/function-lane/W17_DEFERRED_LOW_INTEREST_INVENTORY.csv`
3. `docs/function-lane/W23_DEFERRED_HOST_METADATA_AND_DATABASE_INVENTORY.csv`
4. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W23_DEFERRED_HOST_METADATA_AND_DATABASE_INVENTORY.csv`

Current total:
1. `23` deferred host/metadata/database functions.

High-level partitions:
1. `10` host-integrated / visibility-sensitive functions.
2. `12` database functions.
3. `1` cell-metadata query function (`ISFORMULA`).

## 4. Why These Functions Are Out of the Ordinary Pure Lane
1. `AGGREGATE` and `SUBTOTAL` depend on hidden-row / filtered-row / nested-aggregate visibility semantics.
2. `CALL`, `COPILOT`, `DETECTLANGUAGE`, `GETPIVOTDATA`, `HYPERLINK`, `IMAGE`, `PHONETIC`, and `REGISTER.ID` depend on host, provider, workbook, UI, pivot, or stored metadata seams.
3. `ISFORMULA` depends on whether a referenced cell contains a formula, not merely on the evaluated value of that cell.
4. The `D*` database family is still spreadsheet-semantics rather than pure host query, but it pressures record extraction, field/header matching, criteria-grid evaluation, and row-context criteria formulas strongly enough to deserve a separate packet from the ordinary bounded pure-family residuals.

## 5. In Scope
1. host/query seam-definition work for the host-sensitive cluster,
2. metadata-query classification and execution planning for `ISFORMULA`,
3. database-family replay and semantic hardening planning,
4. separation of these seams from the remaining ordinary semantic-hardening residuals.

## 6. Out of Scope
1. locale-sensitive but still modelable pure/profiled functions such as `DATEVALUE`, `TIMEVALUE`, `ASC`, `DBCS`, `JIS`, `BAHTTEXT`, and `NUMBERVALUE`,
2. random/provider-profile functions unless they also require the host/metadata seams above,
3. closure of the extracted functions in this opening pass.

## 7. Initial Function Set
1. Host-sensitive:
   - `AGGREGATE`
   - `CALL`
   - `COPILOT`
   - `DETECTLANGUAGE`
   - `GETPIVOTDATA`
   - `HYPERLINK`
   - `IMAGE`
   - `PHONETIC`
   - `REGISTER.ID`
   - `SUBTOTAL`
2. Database family:
   - `DAVERAGE`
   - `DCOUNT`
   - `DCOUNTA`
   - `DGET`
   - `DMAX`
   - `DMIN`
   - `DPRODUCT`
   - `DSTDEV`
   - `DSTDEVP`
   - `DSUM`
   - `DVAR`
   - `DVARP`
3. Metadata-query:
   - `ISFORMULA`

## 8. Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - typed host/query seam requirements remain to be split from ordinary value semantics,
   - database-family record/criteria geometry is not yet replay-closed,
   - `ISFORMULA` needs an explicit cell-metadata seam statement.
