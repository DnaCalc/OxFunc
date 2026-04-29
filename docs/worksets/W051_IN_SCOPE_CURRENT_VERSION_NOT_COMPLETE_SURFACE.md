# WORKSET - In-Scope Current-Version Not-Complete Surface (W51)

## 1. Purpose
Centralize the Excel function and operator rows that are still in scope for the current OxFunc version target and are not yet honestly reportable as supported in the published OxFunc library catalog.

This packet now also acts as the first-pass reconciliation hub between:
1. the published library-context snapshot surface seen by downstream consumers,
2. the deferred-current-version list in `W050`,
3. the explicit packet-local residual rows already known to remain open,
4. hidden non-deferred `catalog_only` rows that were still outside legacy `W051`.

This packet also records the current shared-interface acknowledgement outcome for OxFunc ↔ OxFml work:
1. the shared interaction model is now acknowledged for the previously seam-heavy non-deferred surface that was carried through `W023`, `W038`, `W042`, `W046`, `W047`, `W048`, and `W049`,
2. the remaining hidden ordinary backlog remains excluded from that acknowledgement scope unless a concrete later mismatch shows that one of those rows needs more than the ordinary built-in interaction path,
3. the `17` deferred rows in `W050` remain excluded from the current acknowledgement scope and may reopen only on the same concrete-mismatch basis in a later round.

## 2. Provenance
This packet now survives primarily as a parked-baseline summary plus the active
tracker for any reopened non-deferred current-version gap.

Active surviving provenance and control inputs:
1. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
2. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
3. `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md`
4. `docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`
5. the live `W51_*` inventories in `docs/function-lane/`
6. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv`
7. `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`

Historical packet-chain provenance for the removed closed worksets now lives behind:
1. `docs/HISTORY.md`
2. git tag `OxFunc_V1`
3. the archived `W058` normalization packet and the archived `W055` through `W068` closure chain

## 3. Scope
Machine-readable working inputs:
1. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
2. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`
3. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`
4. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv`
5. `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`
6. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv`

Current working total:
1. `41` function rows.
2. `13` operator rows.
3. `54` total rows.
4. current working backlog split:
   - `54` explicitly tracked residual rows with real runtime/formal/evidence floors or packet-specific open lanes,
   - `0` normalized ordinary execution rows remaining after `W068`.
5. current shared-interface-acknowledgement split:
   - the prior seam-heavy non-deferred surface is now acknowledged and promoted out of `W051`,
   - no explicit residual seam-heavy row remains in current `W051`,
   - no hidden non-interesting snapshot entry remains in the active ordinary backlog after `W068`.
6. current hidden-backlog identity split:
   - `0` hidden snapshot entries remain in the current published-catalog reading after `W068`,
   - the grouped text-compat rows have now been cleared out of the live backlog,
   - the ordinary execution program is now fully drained for the current non-deferred surface.

First-pass published-catalog reading:
1. `534` published rows in `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`:
   - `511` functions,
   - `23` operators.
2. `502` rows are currently usable on a first-pass consumer read:
   - `502` supported/complete,
   - `0` preview.
3. `17` rows are deferred in the current version target through `W050`.
4. `49` snapshot entries are non-deferred and not yet honestly supported on the current working read; the current hidden ordinary backlog is still drained.
5. later concrete current-version gaps reopened:
   - the ordinary arithmetic and compare/concat operator rows under `BUG-FUNC-001` / `W073` and then `BUG-FUNC-002` / `W074`,
   - the first-class same-sheet multi-area reference seam and follow-on same-sheet value-materialization lane for `OP_UNION_REF`, `AREAS`, and `INDEX` under `BUG-FUNC-003` / `W075` / `W076`.
   - the corpus-driven numeric comparison tolerance lane for `SWITCH`, the criteria family, and the database family under `BUG-FUNC-004` / `W077`.
   - the adjacent `XLOOKUP` array-valued lookup-needle lane under `BUG-FUNC-006` / `W079`, now locally corrected and awaiting landed-ref promotion.
   - the reopened `RATE` omitted-guess convergence lane for mortgage-style current-baseline rows under `BUG-FUNC-009` / `W081`.
   The bounded `W083` through `W087` reopenings are now landed on committed ref `8234dce5f3e0c50a3c634466ead38c67fa93937e` and removed from active `W051` scope for declared OxFunc ownership.
6. the exact `114` documented-complete snapshot-stale rows from the first-pass reconciliation have now been refreshed out of false `catalog_only` state in the published snapshot export and are not included in the W51 backlog totals.

First-pass documented-complete snapshot-stale groups now refreshed in the current export:
1. `W024` ordinary-done rows: `62`.
2. `W022` criteria-family closures: `7`.
3. `W027` advanced-bond/odd-bond closures: `13`.
4. `W033` information-predicate/forecast closures: `11`.
5. `W037` reopened `XIRR` closure: `1`.
6. `W045` non-`@` operator closures: `19`.
7. `RANDARRAY` legacy cleanup: `1`.

Exact-inventory note:
1. the earlier first-pass packet-group prose had one overlap on `XIRR` between `W024` and later `W037`,
2. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv` resolves that overlap and pins the exact unique refresh set at `114`,
3. the exact source split is now:
   - `W024`: `62`
   - `W022`: `7`
   - `W027`: `13`
   - `W033`: `11`
   - `W037`: `1`
   - `W045`: `19`
   - legacy `RANDARRAY` cleanup: `1`
4. the archived `W058` normalization packet preserved the original hidden-backlog `185` snapshot entries as provenance while creating the machine-clean `192`-row execution inventory used for ordinary backlog closure.
5. the archived `W059` through `W068` packet chain then drained that ordinary backlog to zero, leaving:
   - `0` current snapshot-entry backlog rows,
   - `0` current normalized execution backlog rows.

Completed and removed from this inventory (moved to function-phase-complete):
- `BYCOL`, `BYROW`, `CALL`, `IMAGE`, `ISOMITTED`, `LAMBDA`, `LET`, `MAKEARRAY`, `MAP`, `REDUCE`, `REGISTER.ID`, `SCAN` (12 rows after packet-local closure and shared-freeze promotion)
- `GROUPBY`, `PIVOTBY` (2 rows after grouped-aggregation current-baseline promotion in `W055`)
- `OP_IMPLICIT_INTERSECTION` (1 operator after current-baseline implicit-intersection and legacy-single closure in `W014`)
- `BESSELI`, `BESSELJ`, `BESSELK`, `BESSELY`, `BIN2DEC`, `BIN2HEX`, `BIN2OCT`, `DEC2BIN`, `DEC2HEX`, `DEC2OCT`, `HEX2BIN`, `HEX2DEC`, `HEX2OCT`, `OCT2BIN`, `OCT2DEC`, `OCT2HEX` (16 functions after `W059`)
- `COMPLEX`, `IMABS`, `IMAGINARY`, `IMARGUMENT`, `IMCONJUGATE`, `IMCOS`, `IMCOSH`, `IMCOT`, `IMCSC`, `IMCSCH`, `IMDIV`, `IMEXP`, `IMLN`, `IMLOG10`, `IMLOG2`, `IMPOWER`, `IMPRODUCT`, `IMREAL`, `IMSEC`, `IMSECH`, `IMSIN`, `IMSINH`, `IMSQRT`, `IMSUB`, `IMSUM`, `IMTAN` (26 functions after `W060`)
- `BETA.DIST`, `BETA.INV`, `BETADIST`, `BETAINV`, `BINOM.DIST`, `BINOM.DIST.RANGE`, `BINOM.INV`, `BINOMDIST`, `CHIDIST`, `CHIINV`, `CHISQ.DIST`, `CHISQ.DIST.RT`, `CHISQ.INV`, `CHISQ.INV.RT`, `CONFIDENCE`, `CONFIDENCE.NORM`, `CRITBINOM`, `EXPON.DIST`, `EXPONDIST`, `F.DIST`, `F.DIST.RT`, `F.INV`, `F.INV.RT`, `FDIST`, `FINV`, `GAMMA.DIST`, `GAMMA.INV`, `GAMMADIST`, `GAMMAINV` (29 functions after `W061`)
- `COVAR`, `HYPGEOM.DIST`, `HYPGEOMDIST`, `KURT`, `LOGINV`, `LOGNORM.DIST`, `LOGNORM.INV`, `LOGNORMDIST`, `MODE`, `NEGBINOM.DIST`, `NEGBINOMDIST`, `NORM.DIST`, `NORM.INV`, `NORM.S.DIST`, `NORM.S.INV`, `NORMDIST`, `NORMINV`, `NORMSDIST`, `NORMSINV`, `PERCENTILE`, `PERCENTRANK`, `POISSON`, `POISSON.DIST`, `QUARTILE`, `SKEW`, `SKEW.P`, `STEYX`, `T.DIST`, `T.DIST.2T`, `T.DIST.RT`, `T.INV`, `T.INV.2T`, `TDIST`, `TINV`, `TRIMMEAN` (35 functions after `W062`)
- `DAY`, `DAYS`, `EDATE`, `EOMONTH`, `HOUR`, `ISOWEEKNUM`, `MINUTE`, `MONTH`, `NETWORKDAYS`, `NETWORKDAYS.INTL`, `SECOND`, `TIME`, `WEEKDAY`, `WEEKNUM`, `WORKDAY`, `WORKDAY.INTL`, `YEAR`, `YEARFRAC` (18 functions after `W063`)
- `CUMIPMT`, `CUMPRINC`, `DB`, `DDB`, `DISC`, `DOLLARFR`, `INTRATE`, `PRICEDISC`, `RECEIVED`, `SLN`, `SYD`, `TBILLEQ`, `TBILLPRICE`, `TBILLYIELD`, `VDB` (15 functions after `W064`)
- `DAVERAGE`, `DCOUNT`, `DCOUNTA`, `DGET`, `DMAX`, `DMIN`, `DPRODUCT`, `DSTDEV`, `DSTDEVP`, `DSUM`, `DVAR`, `DVARP` (12 functions after `W065`)
- `CODE`, `CONCATENATE`, `FIND`, `FINDB`, `LEFT`, `LEFTB`, `LEN`, `LENB`, `LOWER`, `MID`, `MIDB`, `PROPER`, `REPLACE`, `REPLACEB`, `REPT`, `RIGHT`, `RIGHTB`, `SEARCH`, `SEARCHB`, `SUBSTITUTE`, `TRIM`, `UNICODE`, `UPPER` (23 functions after `W066`)
- `CEILING.MATH`, `CEILING.PRECISE`, `FLOOR`, `FLOOR.MATH`, `FLOOR.PRECISE`, `ISO.CEILING`, `MDETERM`, `MINVERSE`, `MMULT`, `MUNIT`, `SERIESSUM`, `SUMPRODUCT`, `SUMX2MY2`, `SUMX2PY2`, `SUMXMY2` (15 functions after `W067`)
- `HLOOKUP`, `IFS`, `VLOOKUP` (3 functions after `W068`)
- `COLUMNS`, `RANDARRAY`, `RANDBETWEEN`, `ROWS`, `TRIMRANGE`, `VALUETOTEXT` (6 functions)
- `OP_TRIM_REF_LEADING`, `OP_TRIM_REF_TRAILING`, `OP_TRIM_REF_BOTH` (3 operators, verified against W045 structural slice)

Important current reading:
- the exact packet-local work split for reopened explicit residual rows is pinned through `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`; it now carries the original `ordinary_operator_binary_array_lift` follow-up, the broader `ordinary_operator_broadcast_reconciliation` follow-up, and the `multi_area_reference_seam_correction` follow-up owned by `W075`.
- the hidden ordinary backlog remains empty, but `W051` is no longer empty because the reopened operator lane now sits in the explicit current-gap inventory.
- the hidden ordinary backlog remains empty, but `W051` is no longer empty because the reopened operator lane now sits beside the reopened criteria/database/`SWITCH` numeric-comparison lane in the explicit current-gap inventory.
- the hidden ordinary backlog remains empty, but `W051` is also no longer honest for the adjacent `XLOOKUP` landed-ref promotion lane and reopened `RATE` omitted-guess lane without the corresponding rows tracked under `BUG-FUNC-006` / `W079` and `BUG-FUNC-009` / `W081`.
- the active current hidden backlog is now:
  - `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`, now intentionally empty after `W068`,
  - `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`, now intentionally empty after `W068`.
- the current explicit reopened residual lane is:
  - `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`, now carrying:
    - the reopened ordinary operator rows under `BUG-FUNC-001` / `BUG-FUNC-002` and `W074`,
    - the reopened `SWITCH`, criteria-family, and database-family rows under `BUG-FUNC-004` and `W077`,
    - `OP_UNION_REF`, `AREAS`, and `INDEX` under `BUG-FUNC-003` and `W075` / `W076`,
    - adjacent `XLOOKUP` lookup-family row under `BUG-FUNC-006` and `W079`, pending landed-ref promotion,
    - `RATE` under `BUG-FUNC-009` and `W081`.
- the original hidden `185`-row appendix remains provenance only:
  - these rows were still `catalog_only` in the published snapshot,
  - they were not in `W050`,
  - they were not in legacy `W051`,
  - and this first pass did not find a later packet already closing them.
- the archived `W058` packet supplies the machine-clean execution view of that same backlog in the `OxFunc_V1` history slice.
- the archived `W059` through `W068` packet chain removed the ordinary rows from the active backlog.
- for shared-interface acknowledgement purposes, there is no remaining hidden ordinary backlog in the active current-version non-deferred surface.
- current remaining successor execution split after `W068`:
  - `W073` for the original OxFml handoff intake and first arithmetic widening,
  - `W074` for the broader ordinary-operator broadcast reconciliation and downstream handoff,
  - `W077` for the corpus-driven numeric comparison tolerance follow-on across compare operators, `SWITCH`, criteria family, and database family,
  - `W075` for the first-class same-sheet multi-area reference seam correction,
  - `W076` for the same-sheet multi-area value-materialization seam narrowing,
  - `W079` for the adjacent `XLOOKUP` array-valued lookup-needle landed-ref promotion,
  - `W081` for the reopened `RATE` omitted-guess convergence lane.

Explicit preview-cluster functions:
1. `AREAS`
2. `INDEX`
3. `SWITCH`
4. `XLOOKUP`
5. `RATE`
6. criteria family:
   - `COUNTIF`, `COUNTIFS`, `SUMIF`, `SUMIFS`, `AVERAGEIF`, `AVERAGEIFS`, `MAXIFS`, `MINIFS`
7. database family:
   - `DAVERAGE`, `DCOUNT`, `DCOUNTA`, `DGET`, `DMAX`, `DMIN`, `DPRODUCT`, `DSTDEV`, `DSTDEVP`, `DSUM`, `DVAR`, `DVARP`

Explicit preview-cluster operators:
1. `OP_ADD`
2. `OP_SUBTRACT`
3. `OP_MULTIPLY`
4. `OP_DIVIDE`
5. `OP_POWER`
6. `OP_CONCAT`
7. `OP_EQUAL`
8. `OP_NOT_EQUAL`
9. `OP_LESS_THAN`
10. `OP_LESS_EQUAL`
11. `OP_GREATER_THAN`
12. `OP_GREATER_EQUAL`
13. `OP_UNION_REF`

## 4. Current-Version Rule
For the current version target:
1. every non-deferred row that is not honestly supported in the consumer-facing catalog must appear here, either in the explicit inventory or in the hidden-backlog appendix,
2. rows that are only `catalog_only` in the snapshot export are not exempt from W51 just because they were not yet named in a narrower packet,
3. `GROUPBY` and `PIVOTBY` are now complete for declared current-phase scope under `W055` and removed from `W051`,
4. `IMAGE`, the callable-helper family rows from `W038`, and `CALL` / `REGISTER.ID` are now complete for declared current-phase scope and removed from `W051`,
5. `HYPERLINK` is now treated as complete on the OxFunc side and therefore removed from `W051`; host publication application remains above OxFunc rather than an OxFunc function gap,
6. `ROWS`, `COLUMNS`, `RANDBETWEEN`, `VALUETOTEXT`, `RANDARRAY`, `TRIMRANGE` are now function-phase-complete and removed,
7. trim-reference operators (`OP_TRIM_REF_*`) are verified against W045 structural slice and removed,
8. rows with later complete packet evidence should be kept out of W51 and instead recorded as snapshot/export drift until metadata refresh catches up.
9. the archived shared-interface freeze chain is preserved behind `docs/HISTORY.md` and `OxFunc_V1`; the surviving active anchors are:
   - `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
   - `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
10. `OP_ADD`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_POWER`, `OP_CONCAT`, `OP_EQUAL`, `OP_NOT_EQUAL`, `OP_LESS_THAN`, `OP_LESS_EQUAL`, `OP_GREATER_THAN`, and `OP_GREATER_EQUAL` remain reopened here for the current version target because the local broadcast widening and follow-on numeric-comparison correction are not yet represented by landed refs and still await downstream seam acknowledgment under `HO-FN-005` and `HO-FN-008`.
11. `BUG-FUNC-005` / `W078` no longer reopen `POWER` or `OP_POWER`: fresh Excel replay on 2026-04-29 confirmed `0^0` and `POWER(0,0)` as `#NUM!`, and the local Rust/Lean correction is landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef`.
12. `SWITCH`, the criteria family, and the database family remain reopened here for the current version target because the local numeric-comparison tolerance correction is validated only on the working tree and still awaits landed-ref promotion and downstream seam acknowledgment under `HO-FN-008`.
13. `OP_UNION_REF`, `AREAS`, and `INDEX` remain reopened here for the current version target because the local first-class multi-area correction plus same-sheet value-materialization narrowing are not yet represented by a landed ref and still await downstream seam acknowledgment under `HO-FN-006` and `HO-FN-007`.
14. `MATCH`, `XMATCH`, `HLOOKUP`, and `VLOOKUP` no longer remain reopened here for this lane: live Excel replay on 2026-04-08 pinned array-valued `lookup_value` lifting, and the local correction is landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef` with focused tests replayed on 2026-04-29.
15. `XLOOKUP` remains reopened here only until landed-ref promotion: fresh Excel replay on 2026-04-29 confirmed array-valued `lookup_value` shape preservation and matrix-return scalarization, and the local correction is validated in the working tree under `BUG-FUNC-006` / `W079`.
16. `LEFT, LEFTB`, `MID, MIDB`, and `RIGHT, RIGHTB` no longer remain reopened here for this lane: the text-slice correction is landed on `5d54d7f4ab2cdde6458272292d15ae1b317a0fef` with focused tests replayed on 2026-04-29.
17. `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, `REPT`, `TEXTAFTER`, and `TEXTBEFORE` no longer remain reopened here for this lane: the W080 batch-A correction is landed on `2e818f03a71ba393690275a7fb437ddd9a6bf760` with focused tests replayed on 2026-04-29.
18. `RATE` remains reopened here for the current version target because live Excel replay on 2026-04-10 pinned `RATE(360,-1073.64,200000)` as `0.004166644536345589` per period. The local `W081` correction now matches that omitted-guess lane on the working tree through a bounded bracket-and-bisection fallback around the existing secant path, but no landed ref exists yet and the broader adjacent omitted-guess scan remains open under `BUG-FUNC-009` / `W081`.

## 5. Ownership Rule
1. `W51` is the canonical current-version not-complete working backlog for all non-deferred outstanding rows.
2. Older packets remain provenance/evidence owners and, where applicable, execution owners for the explicit family-specific residual rows.
3. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv` remains provenance for the original hidden snapshot-entry discovery.
4. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv` is the authoritative current execution-owner inventory for the remaining ordinary backlog.
5. the archived `W058` packet remains the post-normalization provenance owner behind `docs/HISTORY.md` and `OxFunc_V1`.
6. New latent gaps or reconciliation findings should be added here immediately, then extracted into narrower workset and bead execution lanes as needed.

## 6. Cleanup And Completion Sequence
The prior non-deferred closure sequence is now qualified by a reopened operator gap.

The surviving current anchors are:
1. `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md` for the published consumer export surface,
2. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md` for the retained runtime carrier model,
3. `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md` for the excluded deferred rows,
4. `docs/worksets/W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md` for the parked Lean-alignment result,
5. `docs/HISTORY.md` plus `OxFunc_V1` for the archived closure chain that drained the former preview and hidden ordinary backlog.

Exit condition:
- `W051` contains every non-deferred row that is still not honestly supported.
- the shared-interface model for the interesting non-deferred surface is explicitly frozen across OxFunc and OxFml.
- documented-complete rows are no longer misreported as `catalog_only` in the published snapshot.
- the reopened `BUG-FUNC-001` / `BUG-FUNC-002` / `W074` operator broadcast lane is either fully published and acknowledged or honestly reclassified.
- the reopened `BUG-FUNC-004` / `W077` numeric-comparison tolerance lane is either fully published and acknowledged or honestly reclassified.
- the reopened `BUG-FUNC-003` / `W075` / `W076` multi-area reference lane is either fully published and acknowledged or honestly reclassified.
- the reopened `BUG-FUNC-006` / `W079` adjacent `XLOOKUP` array-valued lookup-needle lane is fully published.
- the reopened `BUG-FUNC-009` / `W081` `RATE` omitted-guess convergence lane is either fully published or honestly reclassified.
- downstream consumer docs can report one aligned set of counts without relying on stale side packets.

## 7. Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - `OP_ADD`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_POWER`, `OP_CONCAT`, `OP_EQUAL`, `OP_NOT_EQUAL`, `OP_LESS_THAN`, `OP_LESS_EQUAL`, `OP_GREATER_THAN`, and `OP_GREATER_EQUAL` remain open under `BUG-FUNC-001` / `BUG-FUNC-002` / `W074` until the local broadcast widening and compare-lane hardening are fully published, with the operator seam then still acknowledged downstream through `HO-FN-005` and `HO-FN-008`
  - `SWITCH`, the criteria family, and the database family remain open under `BUG-FUNC-004` / `W077` until the local numeric-comparison tolerance correction is landed on a committed ref and acknowledged downstream through `HO-FN-008`
  - `OP_UNION_REF`, `AREAS`, and `INDEX` remain open under `BUG-FUNC-003` / `W075` / `W076` until the local first-class `MultiArea` correction plus same-sheet value-materialization narrowing are landed on a committed ref and acknowledged downstream through `HO-FN-006` and `HO-FN-007`
  - adjacent `XLOOKUP` remains open under `BUG-FUNC-006` / `W079` until the array-valued `lookup_value` correction is promoted to a landed ref
  - `RATE` remains open under `BUG-FUNC-009` / `W081` until the local omitted-guess/default-guess convergence repair is landed on a committed ref, the earlier admitted seed inversion lane remains aligned, and the bounded adjacent omitted-guess scan is reconciled honestly
