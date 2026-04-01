# Function Slice Contract (Preliminary) - Date Time And Business Day Family

Status: `provisional`
Workset: `W063`
Primary Functions: `DAY`, `DAYS`, `EDATE`, `EOMONTH`, `HOUR`, `ISOWEEKNUM`, `MINUTE`, `MONTH`, `NETWORKDAYS`, `NETWORKDAYS.INTL`, `SECOND`, `TIME`, `WEEKDAY`, `WEEKNUM`, `WORKDAY`, `WORKDAY.INTL`, `YEAR`, `YEARFRAC`

## 1. Scope
1. close the ordinary date/time and business-day slice already carried in local Rust and Lean family substrates,
2. preserve the current pinned `1900` workbook-date baseline, including Excel's serial-`0` / serial-`60` quirks where applicable,
3. treat `WORKDAY*`, `NETWORKDAYS*`, and `YEARFRAC` as part of the same calendar/basis family for current-baseline closure.

## 2. Admitted Current-Baseline Slice
1. `DAY`, `MONTH`, `YEAR`, and `DAYS`
   - preserve truncated-serial extraction and subtraction on the `1900` serial timeline,
   - preserve serial `0 -> 1900-01-00`,
   - preserve fake leap-day serial `60 -> 1900-02-29`,
   - reject negative serials with `#NUM!`.
2. `HOUR`, `MINUTE`, `SECOND`, and `TIME`
   - extract from the fractional-day portion of the serial after rejecting negative inputs with `#NUM!`,
   - truncate component inputs toward zero,
   - accept numeric text and logicals through the ordinary numeric coercion path,
   - treat omitted/blank `TIME` components as zero,
   - reject component magnitudes above `32767` with `#NUM!`.
3. `EDATE`, `EOMONTH`, `WEEKDAY`, `WEEKNUM`, and `ISOWEEKNUM`
   - preserve month-clamp behavior and serial-timeline weekday numbering,
   - preserve the fake-leap serial-`60` normalization lanes,
   - preserve `WEEKNUM(...,21)` and `ISOWEEKNUM` ISO-week behavior on the same serial timeline,
   - reject invalid return-type lanes with `#NUM!`.
4. `WORKDAY`, `WORKDAY.INTL`, `NETWORKDAYS`, and `NETWORKDAYS.INTL`
   - preserve inclusive business-day counting and business-day stepping on the `1900` serial baseline,
   - honor weekend-number and seven-bit weekend-mask parsing,
   - preserve the current observed split where `WORKDAY.INTL` rejects all-weekend mask `1111111` with `#VALUE!`, while `NETWORKDAYS.INTL` accepts it and returns `0`,
   - preserve holiday-range handling through ordinary reference expansion.
5. `YEARFRAC`
   - preserve the current bounded basis slice `0..4`,
   - preserve basis-specific `30/360`, bounded Actual/Actual, `Actual/360`, `Actual/365`, and European `30/360` behavior,
   - reject invalid basis values with `#NUM!`.

## 3. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg preparation:
   - `ValuesOnlyPreAdapter` for all covered rows
6. fec dependency:
   - `None` for all covered rows,
   - surface dependency `RefOnly` for all rows except `YEARFRAC`, which preserves the existing `None` / `None` family posture.

## 4. Evidence Basis
1. legacy local notes:
   - `docs/function-lane/W16_BATCH38_DATE_PARTS_NOTES.md`
   - `docs/function-lane/W16_BATCH39_TIME_PARTS_NOTES.md`
   - `docs/function-lane/W16_BATCH44_DATE_WEEK_FAMILY_NOTES.md`
   - `docs/function-lane/W16_BATCH64_WORKDAY_NETWORKDAYS_NOTES.md`
   - `docs/function-lane/W16_BATCH76_DISCOUNT_BILL_YEARFRAC_NOTES.md`
2. native packet:
   - `docs/function-lane/W63_SCENARIO_MANIFEST_SEED.csv`
   - `.tmp/w63-date-time-business-day-results.csv`
3. runtime harness:
   - `tools/w63-probe/run-w63-date-time-business-day-baseline.ps1`
4. packet execution record:
   - `docs/function-lane/W63_EXECUTION_RECORD.md`
