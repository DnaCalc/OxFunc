# WORKSET - Finance Functions F# Benchmark Cross-Check (W29)

## 1. Purpose
Run a deliberate cross-check of OxFunc finance families against the public ExcelFinancialFunctions F# implementation and its documented compatibility/test surface, then classify any discrepancies against direct Excel evidence.

## 2. Provenance
Opened after `W027` established that targeted structural comparison against the F# finance reference is useful, but that no full OxFunc-local benchmark pass has yet been run against the broader F# finance surface.

Public sources:
1. `https://github.com/fsprojects/ExcelFinancialFunctions`
2. `https://fsprojects.github.io/ExcelFinancialFunctions/compatibility.html`
3. `../Foundation/research/runs/20260228-130325-excel-compat-spec-index-pass-01/outputs/23_excel_financial_functions_watch_note.md`

## 3. Scope
1. verify which finance families have already been fully compared against the F# implementation and test/compatibility surface,
2. where that comparison has not been done, add an explicit benchmark pass,
3. compare discrepancies across:
   - OxFunc,
   - ExcelFinancialFunctions,
   - direct Excel empirical results,
4. record whether each discrepancy is:
   - an OxFunc defect,
   - an F# known divergence from Excel,
   - an Excel-version/build difference,
   - or a still-open ambiguity.

## 4. In Scope
1. `financial_time_value_family`
2. `cashflow_rate_family`
3. `coupon_family`
4. `bond_core_family`
5. `odd_bond_family`
6. closely related finance helper families when needed to explain a discrepancy

## 5. Out Of Scope
1. treating the F# project as semantic authority over Excel,
2. closing finance families without direct Excel parity where needed,
3. unrelated non-finance locale/provider/host seams.

## 6. Initial Findings
1. OxFunc has packet-evidenced current-baseline closure for several finance families, but no local doc currently claims a full benchmark pass against ExcelFinancialFunctions.
2. The public ExcelFinancialFunctions compatibility page says the library has `199,252` tests against Excel 2010.
3. The same public compatibility page explicitly notes remaining differences for at least:
   - `RATE` and `ODDFYIELD` due root-finding differences,
   - `XIRR` and `XNPV` in some negative-rate/root-finding cases.
4. `W027` already used the public F# `priceMat` / `yieldMat` and `oddLFunc` formulas as a targeted structural cross-check and then closed the direct Excel blocker lanes for:
   - `PRICEMAT` / `YIELDMAT`,
   - `ODDLPRICE` / `ODDLYIELD`.
5. The remaining gap is therefore a broader benchmark ledger, not those specific blocker lanes.

## 7. Deliverables
1. benchmark scope note,
2. discrepancy ledger between OxFunc, F#, and Excel,
3. explicit note on which F# differences are known non-Excel behaviors,
4. handoff back into `W027` and related finance packets.

## 8. Dependencies
1. depends on current finance packet evidence from `W24` and `W27`,
2. supports `W027` but does not replace direct Excel parity work.

## 9. Execution Result
1. `W29` now has a reproducible three-way benchmark runner:
   - `tools/w29-probe/run-w29-finance-benchmark-crosscheck.ps1`
2. The packet now records:
   - `docs/function-lane/W29_FINANCE_BENCHMARK_DISCREPANCY_LEDGER.csv`
   - `docs/function-lane/W29_EXECUTION_RECORD.md`
   - `.tmp/w29-finance-summary.json`
3. The public F# unit-test project was also run locally and passed `3221` tests.
4. Current classification outcome:
   - `6` benchmark lanes aligned across OxFunc, F#, and Excel,
   - `4` lanes showed OxFunc matching F# but not Excel,
   - `1` lane showed F# matching Excel while OxFunc differed,
   - `2` lanes showed both OxFunc and F# diverging from Excel.
5. `W29` therefore completes as a benchmark-and-classification packet and reopens concrete OxFunc parity repair work in `W32`.

## 10. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W29` scope; successor repair work is now owned by `W32`
