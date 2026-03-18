# W24 Batch 12 Execution Record - Cashflow Rate Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B12-CASHFLOW-RATE-20260318`

## 1. Purpose
Record the cashflow rate family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `IRR`, `XNPV`, and `XIRR` for the admitted current reference baseline,
2. replace the older note-only bounded standing with packet evidence,
3. bind the runtime and Lean substrate to a replayable native worksheet packet.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - broader difficult multi-root and alternate iteration-path parity remains outside this packet,
   - mixed-type and multi-column cashflow/date coercions remain outside the admitted slice.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_CASHFLOW_RATE_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch12-cashflow-rate-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `docs/function-lane/W16_BATCH71_CASHFLOW_RATE_NOTES.md`
10. `formal/lean/OxFunc/Functions/CashflowRateFamily.lean`
11. `docs/function-lane/W17_DEFERRED_LOW_INTEREST_INVENTORY.csv`
12. `docs/worksets/W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md`

## 5. Empirical Findings
From `.tmp/w24-batch12-cashflow-rate-results.csv`:
1. `IRR({-100,121})` returned `0.21`, and the explicit-guess lane converged to the same root.
2. `IRR({100,121})` returned `#NUM!`, matching the local sign-change rejection rule.
3. `XNPV(0.1,{-100,121},{45000,45365})` returned `10`.
4. `XNPV` returned `#NUM!` on both length mismatch and pre-anchor-date lanes.
5. `XIRR({-100,121},{45000,45365})` returned `0.21`, and the explicit-guess lane converged to the same root.
6. `XIRR` returned `#NUM!` on both missing-sign-change and malformed date-vector lanes.

## 6. Implementation Result
1. The family runtime and Lean binding were already integrated through dispatch/export/formal surfaces.
2. The packet did not expose a solver mismatch on the seeded current-baseline lanes.
3. The family is now packet-evidenced instead of remaining only note-bounded.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch12-cashflow-rate-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml cashflow_rate_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. `IRR`, `XNPV`, and `XIRR` are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted numeric vector/date-vector slice above.
3. `W024` continues with the remaining unblocked families after this packet.
