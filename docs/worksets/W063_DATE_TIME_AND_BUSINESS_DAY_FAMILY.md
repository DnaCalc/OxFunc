# WORKSET - Date Time And Business Day Family (W063)

## 1. Purpose
Promote the fifth ordinary successor packet from the normalized `W051` backlog by closing the date/time and business-day family for the current reference Excel baseline.

This packet closes:
1. `DAY`
2. `DAYS`
3. `EDATE`
4. `EOMONTH`
5. `HOUR`
6. `ISOWEEKNUM`
7. `MINUTE`
8. `MONTH`
9. `NETWORKDAYS`
10. `NETWORKDAYS.INTL`
11. `SECOND`
12. `TIME`
13. `WEEKDAY`
14. `WEEKNUM`
15. `WORKDAY`
16. `WORKDAY.INTL`
17. `YEAR`
18. `YEARFRAC`

## 2. Dependencies
1. `W057_HIDDEN_ORDINARY_BACKLOG_SYSTEMATIC_COMPLETION_PLAN.md`
2. `W058_GROUPED_ROW_NORMALIZATION_AND_HIDDEN_BACKLOG_SPLIT.md`
3. `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`
4. prior implementation/evidence lineage from:
   - `crates/oxfunc_core/src/functions/date_parts_family.rs`
   - `crates/oxfunc_core/src/functions/date_week_family.rs`
   - `crates/oxfunc_core/src/functions/workday_networkdays_family.rs`
   - `crates/oxfunc_core/src/functions/discount_bill_yearfrac_family.rs`
   - `formal/lean/OxFunc/Functions/DatePartsFamily.lean`
   - `formal/lean/OxFunc/Functions/DateWeekFamily.lean`
   - `formal/lean/OxFunc/Functions/WorkdayNetworkdaysFamily.lean`
   - `formal/lean/OxFunc/Functions/DiscountBillYearfracFamily.lean`
   - `docs/function-lane/W16_BATCH38_DATE_PARTS_NOTES.md`
   - `docs/function-lane/W16_BATCH39_TIME_PARTS_NOTES.md`
   - `docs/function-lane/W16_BATCH44_DATE_WEEK_FAMILY_NOTES.md`
   - `docs/function-lane/W16_BATCH64_WORKDAY_NETWORKDAYS_NOTES.md`
   - `docs/function-lane/W16_BATCH76_DISCOUNT_BILL_YEARFRAC_NOTES.md`

## 3. Scope
In scope:
1. native Excel replay for the declared current-baseline date-part, time-part, date-week, business-day, and `YEARFRAC` slice,
2. closure-grade OxFunc runtime and dispatch evidence for all `18` rows,
3. Lean substrate/binding alignment confirmation for the four existing date-family formal artifacts,
4. snapshot/export promotion so these rows stop reading as hidden `catalog_only` backlog,
5. `W051` removal/update for the `18` rows.

Out of scope:
1. locale/version sweeps beyond the declared current reference baseline,
2. `DATEVALUE`, `TIMEVALUE`, `DAYS360`, `DATEDIF`, and coupon/bill siblings outside the admitted `W063` rows,
3. workbook date-system expansion beyond the pinned `1900` baseline.

## 4. Output Contract
This packet must produce:
1. this workset spec,
2. `docs/function-lane/FUNCTION_SLICE_DATE_TIME_AND_BUSINESS_DAY_FAMILY_CONTRACT_PRELIM.md`,
3. `docs/function-lane/W63_SCENARIO_MANIFEST_SEED.csv`,
4. `docs/function-lane/W63_RUNTIME_REQUIREMENTS.md`,
5. `docs/function-lane/W63_SCOPE_RECONCILIATION.csv`,
6. `docs/function-lane/W63_EXECUTION_RECORD.md`,
7. `tools/w63-probe/run-w63-date-time-business-day-baseline.ps1`,
8. `.tmp/w63-date-time-business-day-results.csv`,
9. `tools/w44-probe/generate-w44-library-context-snapshot.ps1`,
10. refreshed downstream `W051` / snapshot / policy / worklist counters.

## 5. Completion Gate
`W063` may be reported complete only when:
1. every covered row has at least one explicit native Excel replay lane in `W63_SCENARIO_MANIFEST_SEED.csv`,
2. native Excel replay matches the seeded `W63` manifest for all declared lanes,
3. the targeted Rust tests for the four family modules pass,
4. `lake build` passes without introducing new `W054` active gaps,
5. all `18` `W063` rows move out of `catalog_only` in the regenerated `W44` snapshot,
6. `W051` and downstream counts reconcile to the post-`W063` backlog.

## 6. Notes
1. `W063` is expected to be a closure-by-evidence-and-publication packet rather than a large semantic rewrite packet unless the native replay exposes a mismatch.
2. `YEARFRAC` remains admitted through the already-pinned bounded basis `0/1/2/3/4` slice; broader finance-family refactoring stays outside this packet.
