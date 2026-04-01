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
This packet consolidates active current-version backlog ownership from:
1. `W014_IMPLICIT_INTERSECTION_OPERATOR.md`
2. `W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
3. `W025_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_OUTLIERS.md`
4. `W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md`
5. `W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`
6. `W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM.md`
7. latent catalog gaps visible through `W044`
8. later packet-complete evidence checked during the first-pass catalog reconciliation:
   - `W022_CRITERIA_FAMILY_SHAPE_HARDENING.md`
   - `W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
   - `W027_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_HARDENING.md`
   - `W033_INFORMATION_PREDICATES_AND_FORECAST_COMPATIBILITY_CLOSURE.md`
   - `W037_REOPENED_XIRR_LARGE_ROOT_SOLVER_PRECISION.md`
   - `W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`

## 3. Scope
Machine-readable working inputs:
1. `docs/function-lane/W51_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_INVENTORY.csv`
2. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`
3. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`
4. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv`
5. `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv`
6. `docs/function-lane/W58_GROUPED_ROW_NORMALIZATION_MAP.csv`
7. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
8. `docs/function-lane/W59_SCOPE_RECONCILIATION.csv`
9. `docs/function-lane/W60_SCOPE_RECONCILIATION.csv`
10. `docs/function-lane/W61_SCOPE_RECONCILIATION.csv`
11. `docs/function-lane/W62_SCOPE_RECONCILIATION.csv`
12. `docs/function-lane/W63_SCOPE_RECONCILIATION.csv`
13. `docs/function-lane/W64_SCOPE_RECONCILIATION.csv`
14. `docs/function-lane/W65_SCOPE_RECONCILIATION.csv`
15. `docs/function-lane/W66_SCOPE_RECONCILIATION.csv`
16. `docs/function-lane/W67_SCOPE_RECONCILIATION.csv`
17. `docs/function-lane/W68_SCOPE_RECONCILIATION.csv`
18. `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`
19. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv`

Current working total:
1. `0` function rows.
2. `0` operator rows.
3. `0` total rows.
4. current working backlog split:
   - `0` explicitly tracked residual rows with real runtime/formal/evidence floors or packet-specific open lanes,
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
2. `517` rows are currently usable on a first-pass consumer read:
   - `517` supported/complete,
   - `0` preview.
3. `17` rows are deferred in the current version target through `W050`.
4. `0` snapshot entries are non-deferred and not yet supported; the current hidden ordinary backlog is now drained.
5. the exact `114` documented-complete snapshot-stale rows from the first-pass reconciliation have now been refreshed out of false `catalog_only` state in the published snapshot export and are not included in the W51 backlog totals.

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
4. `W058` preserves the original hidden-backlog `185` snapshot entries as provenance while creating the machine-clean `192`-row execution inventory used for ordinary backlog closure.
5. `W059` then removes `16` rows from the active ordinary backlog.
6. `W060` then removes a further `26` rows.
7. `W061` then removes a further `29` rows.
8. `W062` then removes a further `35` rows.
9. `W063` then removes a further `18` rows.
10. `W064` then removes a further `15` rows.
11. `W065` then removes a further `12` rows.
12. `W066` then removes a further `23` normalized text-family rows.
13. `W067` then removes a further `15` rows.
14. `W068` then removes the final `3` rows, leaving:
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
- the exact packet-local work split for any future explicit residual rows is pinned through `docs/function-lane/W51_INTERESTING_POST_FREEZE_LOCAL_WORK.csv`; it is currently empty because no explicit preview-cluster row remains.
- there is no remaining explicit preview-cluster row; the only residual `W051` membership is now the hidden ordinary backlog.
- the active current hidden backlog is now:
  - `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_CURRENT.csv`, now intentionally empty after `W068`,
  - `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv`, now intentionally empty after `W068`.
- the original hidden `185`-row appendix remains provenance only:
  - these rows were still `catalog_only` in the published snapshot,
  - they were not in `W050`,
  - they were not in legacy `W051`,
  - and this first pass did not find a later packet already closing them.
- `W058` now supplies the authoritative machine-clean execution view of that same backlog:
  - `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv` is the execution inventory,
  - `docs/function-lane/W58_GROUPED_ROW_NORMALIZATION_MAP.csv` pins the seven grouped-row splits,
  - `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv` freezes the exact `W059` through `W068` ownership counts.
- `W059` is now complete and removed from the active backlog.
- `W060` is now complete and removed from the active backlog.
- `W061` is now complete and removed from the active backlog.
- `W062` is now complete and removed from the active backlog.
- `W063` is now complete and removed from the active backlog.
- `W064` is now complete and removed from the active backlog.
- `W065` is now complete and removed from the active backlog.
- `W066` is now complete and removed from the active backlog.
- `W067` is now complete and removed from the active backlog.
- `W068` is now complete and removes the final ordinary rows from the active backlog.
- for shared-interface acknowledgement purposes, there is no remaining hidden ordinary backlog in the active current-version non-deferred surface.
- current remaining successor execution split after `W068`:
  - none.

Explicit preview-cluster functions:
None.

Explicit preview-cluster operators:
None.

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
9. the current shared-interface freeze/promotion pass is owned by:
   - `W042` for the callable minimum carrier,
   - `W046` for registered-external packet shape,
   - `W047` for typed context/query bundle,
   - `W048` for return-surface split,
   - `W049` for runtime library-context provider/snapshot model,
   - `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` as the consolidated outbound model.

## 5. Ownership Rule
1. `W51` is the canonical current-version not-complete working backlog for all non-deferred outstanding rows.
2. Older packets remain provenance/evidence owners and, where applicable, execution owners for the explicit family-specific residual rows.
3. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv` remains provenance for the original hidden snapshot-entry discovery.
4. `docs/function-lane/W51_NORMALIZED_ORDINARY_BACKLOG_CURRENT.csv` is the authoritative current execution-owner inventory for the remaining ordinary backlog.
5. `docs/function-lane/W58_HIDDEN_ORDINARY_BACKLOG_NORMALIZED.csv` remains the full post-normalization provenance inventory and freezes the original successor ownership through `W059` to `W068`.
6. New latent gaps or reconciliation findings should be added here immediately, then extracted into narrower execution packets as needed.

## 6. Cleanup And Completion Sequence
1. Finalize the remaining shared-model owner packets for the interesting backlog:
   - `W042`
   - `W046`
   - `W047`
   - `W048`
   - `W049`
2. Consolidate those owner-packet readings into one outbound current-phase model note:
   - `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
3. Completed in the current pass: the OxFunc ↔ OxFml notes interchange against that single consolidated model no longer shows a material interface mismatch for the interesting non-deferred surface.
4. Completed in the current pass: the shared-interface model is now acknowledged across OxFunc and OxFml and promoted into the current owner-packet reading across `W042`, `W046`, `W047`, `W048`, `W049`, and `W051`.
5. Completed in the current pass: close the packet-local promotion/doc lanes for `W023`, `W038`, and `W046`, removing `IMAGE`, the nine callable-helper rows, and `CALL` / `REGISTER.ID` from `W051`.
6. Completed in the current pass: refresh the exact `114` documented-complete snapshot-stale rows listed in `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv` out of false `catalog_only` state in the published library-context snapshot and downstream labeling guidance.
7. Completed in the current pass: promote `GROUPBY` and `PIVOTBY` through `W055`, removing them from the residual preview cluster and narrowing the explicit `W051` surface to `OP_IMPLICIT_INTERSECTION` only.
8. Completed in the current pass: close `W014` for declared current-phase scope after pinning current-baseline `_xlfn.SINGLE(...)` normalization and removing `OP_IMPLICIT_INTERSECTION` from `W051`.
9. Completed in the current pass: `W058` normalized grouped-name rows in the hidden backlog appendix (`FIND, FINDB`; `LEFT, LEFTB`; `LEN, LENB`; `MID, MIDB`; `REPLACE, REPLACEB`; `RIGHT, RIGHTB`; `SEARCH, SEARCHB`) into machine-clean row identities.
10. Completed in the current pass: `W058` froze the normalized ordinary backlog into narrower successor execution packets by family rather than leaving it as one mega-list.
11. Completed in the current pass: `W059` closes the engineering radix conversion family plus the Bessel quartet and removes `16` rows from active `W051` backlog membership.
12. Completed in the current pass: `W060` closes the complex-number family and removes a further `26` rows from active `W051` backlog membership.
13. Completed in the current pass: `W061` closes the first statistical-distribution and compatibility wave, reconciles the matching `W054` Lean-id gaps, and removes a further `29` rows from active `W051` backlog membership.
14. Completed in the current pass: `W062` closes the second statistical-distribution and compatibility wave, reconciles the remaining T-family `W054` Lean-id gaps for that slice, and removes a further `35` rows from active `W051` backlog membership.
15. Completed in the current pass: `W063` closes the date/time and business-day family and removes a further `18` rows from active `W051` backlog membership.
16. Completed in the current pass: `W064` closes the financial core miscellaneous family and removes a further `15` rows from active `W051` backlog membership.
17. Completed in the current pass: `W065` promotes the already-evidenced database family, refreshes those `12` rows out of hidden `catalog_only` publication drift, and removes them from active `W051` backlog membership.
18. Completed in the current pass: `W066` promotes the already-evidenced text core and compatibility family, refreshes those `16` hidden snapshot entries out of `catalog_only` publication drift, and removes the matching `23` normalized rows from active `W051` backlog membership.
19. Completed in the current pass: `W067` promotes the already-evidenced rounding, matrix, and sumproduct-family rows, refreshes those `15` hidden snapshot entries out of `catalog_only` publication drift, and removes the matching `15` normalized rows from active `W051` backlog membership.
20. Completed in the current pass: `W068` promotes the final lookup/logical residuals, refreshes those `3` hidden snapshot entries out of `catalog_only` publication drift, and removes the final `3` normalized rows from active `W051` backlog membership.
21. Completed in the current pass: `W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md` is now fully reconciled for the non-deferred parked surface; no active Rust-vs-Lean missing-id gap remains for those rows.
22. Regenerate the published consumer surface after each closure wave:
   - snapshot export
   - labeling policy
   - consumer-facing counts
   - `W051` membership

Exit condition:
- `W051` contains every non-deferred row that is still not honestly supported.
- the shared-interface model for the interesting non-deferred surface is explicitly frozen across OxFunc and OxFml.
- documented-complete rows are no longer misreported as `catalog_only` in the published snapshot.
- downstream consumer docs can report one aligned set of counts without relying on stale side packets.

## 7. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none
