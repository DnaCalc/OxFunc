# W24 Batch 11 Execution Record - Financial Time-Value Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B11-FINANCIAL-TIME-VALUE-20260318`

## 1. Purpose
Record the financial time-value family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `PV`, `FV`, `PMT`, `NPER`, `NPV`, `RATE`, `IPMT`, `PPMT`, `ISPMT`, `MIRR`, `FVSCHEDULE`, `PDURATION`, `RRI`, `NOMINAL`, and `EFFECT` for the admitted current reference baseline,
2. replace the incorrect local `ISPMT` schedule note with the witnessed Excel behavior,
3. bind the runtime and Lean substrate to a replayable native worksheet packet.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader cross-build `RATE` convergence parity remains outside this packet.
   - richer mixed cashflow/sequence breadth remains outside this packet.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch11-financial-time-value-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `docs/function-lane/W16_BATCH55_FINANCIAL_TIME_VALUE_NOTES.md`
10. `formal/lean/OxFunc/Functions/FinancialTimeValueFamily.lean`
11. `crates/oxfunc_core/src/functions/financial_time_value_family.rs`

## 5. Empirical Findings
From `.tmp/w24-batch11-financial-time-value-results.csv`:
1. The annuity identity packet matched for `PV`, `FV`, `PMT`, and `NPER`.
2. `RATE(48,PMT(0.01,48,8000),8000)` returned `0.00999999999999997`, matching the admitted inversion sample.
3. `NPV(0.1,-10000,3000,4200,6800)` returned `1188.44341233522`.
4. `IPMT(0.10/12,1,36,8000)` returned `-66.6666666666667` and `PPMT(...)` returned `-191.470830884033`.
5. `ISPMT(0.1,1,4,1000)` returned `-75`, while `ISPMT(0.1,0,4,1000)` returned `-100`; the old local expectation of `-100` at period `1` was wrong.
6. `MIRR`, `FVSCHEDULE`, `PDURATION`, `RRI`, `NOMINAL`, and `EFFECT` matched the seeded sample lanes.

## 6. Implementation Result
1. The family was already broadly integrated through dispatch/export/Lean surfaces.
2. The packet exposed and corrected a real semantic bug in `ISPMT`.
3. The admitted scalar and numeric-sequence family is now packet-evidenced rather than note-only.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch11-financial-time-value-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml financial_time_value_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. The family members listed above are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted scalar/sequence financial slice above.
3. `W024` continues with the remaining unblocked families after this packet.
