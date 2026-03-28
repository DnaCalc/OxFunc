# WORKSET - Deferred Host, Metadata, and Database Functions (W23)

## 1. Purpose
Own the low-interest residuals that are not honestly pure function work on the current OxFunc boundary.

This packet originally gathered:
1. host/query state functions,
2. cell-metadata functions,
3. record-style database/criteria-grid functions,
4. visibility-sensitive aggregate functions.

## 2. Provenance
Opened as a successor extraction from `W17`.

Backlog ownership note:
1. `W023` remains the provenance/evidence owner for the residual `HYPERLINK` / `IMAGE` work.
2. Active current-version backlog tracking now sits in `W051`.

Source artifacts:
1. `docs/worksets/W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md`
2. `docs/function-lane/W17_DEFERRED_LOW_INTEREST_INVENTORY.csv`
3. `docs/function-lane/W23_DEFERRED_HOST_METADATA_AND_DATABASE_INVENTORY.csv`
4. `docs/function-lane/W23_SCOPE_RECONCILIATION.csv`
5. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W23_DEFERRED_HOST_METADATA_AND_DATABASE_INVENTORY.csv`
2. `docs/function-lane/W23_SCOPE_RECONCILIATION.csv`

Packet total:
1. `17` functions passed through `W23`.

Current residual open set:
1. `1` function remains open in `W23`.

Current standing by cluster:
1. `12` database functions are now evidenced in-packet.
2. `3` metadata / visibility functions are now evidenced in-packet (`ISFORMULA`, `SUBTOTAL`, `AGGREGATE`).
3. `HYPERLINK` is now complete on the OxFunc side for current-phase purposes.
4. `1` publication/provider-sensitive function remains open in packet scope: `IMAGE`.

## 4. Why These Functions Were Out of the Ordinary Pure Lane
1. `AGGREGATE` and `SUBTOTAL` depend on hidden-row / filtered-row / nested-aggregate visibility semantics.
2. `HYPERLINK` and `IMAGE` expose publication/provider seams even though their scalar value side is narrow or opaque on the current baseline.
3. `ISFORMULA` depends on whether a referenced cell contains a formula, not merely on the evaluated value of that cell.
4. The `D*` database family pressures record extraction, field/header matching, criteria-grid evaluation, and row-context criteria formulas strongly enough to deserve a separate packet from the ordinary pure-family residuals.

## 5. In Scope
1. host/query seam-definition work for the host-sensitive cluster,
2. metadata-query classification and execution planning for `ISFORMULA`,
3. database-family replay, semantic hardening, and bounded current-baseline closure,
4. separation of these seams from the remaining ordinary semantic-hardening residuals.

## 6. Out of Scope
1. locale-sensitive but still modelable pure/profiled functions such as `DATEVALUE`, `TIMEVALUE`, `ASC`, `DBCS`, `JIS`, `BAHTTEXT`, and `NUMBERVALUE`,
2. random/provider-profile functions unless they also require the host/metadata seams above,
3. full closure of the remaining `HYPERLINK` / `IMAGE` publication/provider seams in this pass.

## 7. Initial Function Set
1. Host-sensitive / visibility-sensitive:
   - `AGGREGATE`
   - `HYPERLINK`
   - `IMAGE`
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

## 8. Current Packet Standing
1. the database family now has runtime, Lean, and native Excel evidence inside `W23`.
2. `ISFORMULA` now has a typed host-query seam and current-baseline OxFunc closure.
3. `SUBTOTAL` and `AGGREGATE` now have a typed row-visibility callback seam and current-baseline OxFunc closure for the admitted reference-form slice.
4. the remaining deferred residual in `W23` is now only:
   - `IMAGE`
5. `HYPERLINK` is complete on the OxFunc side: value semantics and presentation intent are modeled, while actual style/clickability application remains host-owned and therefore above the OxFunc function boundary.
6. `IMAGE` now also has a real OxFunc runtime surface:
   - strict Excel-style argument validation,
   - typed `HostInfoProvider::query_image(...)` request normalization,
   - provider-classified `#CONNECT!` / `#BLOCKED!` / provider-error mapping,
   - and an extended `_webimage` rich-value return path.
7. the latest OxFml note confirms generic non-ordinary return-surface preservation and explicit `_webimage` packet evidence locally. The current joint seam reading is now that the semantic success carrier is `ExtendedValue::RichValue(_webimage)`, while the published worksheet fallback remains a separate host-visible projection.
8. OxFml now also claims and the local tree now verifies a real `IMAGE(...)` floor through evaluator, host, and adapter paths, including `TypedContextQueryFamily::Image`.
9. OxFunc also has a local `SingleFormulaHost` regression proving the same split from the OxFunc side.

## 9. Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - first-freeze cleanup around `TypedContextQueryFamily::Image`
   - confirm the current `W042` returned-value vocabulary is sufficient for current-phase `IMAGE` closure
   - broader current-phase promotion/closure work for the `IMAGE` lane
