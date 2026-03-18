# W24 Batch 13 Execution Record - Coupon Family

Status: `complete-provisional`
Workset: `W24`
Evidence ID: `W24-B13-COUPON-20260318`

## 1. Purpose
Record the coupon family closure packet inside the `W24` ordinary mega-batch.

## 2. Scope
1. close `COUPDAYBS`, `COUPDAYS`, `COUPDAYSNC`, `COUPNCD`, `COUPNUM`, and `COUPPCD` for the admitted current reference baseline,
2. promote the family from bounded-note status to packet evidence,
3. correct the local early-date boundary behavior so it matches the current Excel baseline.

## 3. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - 1904 workbook date system support remains outside this packet,
   - irregular schedule and broader serial-`60` edge investigations remain outside this packet,
   - broader version sweeps remain outside this packet.

## 4. Executed Scope
Artifacts created or updated:
1. `docs/function-lane/FUNCTION_SLICE_COUPON_FAMILY_CONTRACT_PRELIM.md`
2. `docs/function-lane/W24_BATCH13_COUPON_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/W24_BATCH13_COUPON_RUNTIME_REQUIREMENTS.md`
4. `docs/function-lane/W24_BATCH13_COUPON_EXECUTION_RECORD.md`
5. `tools/w24-probe/run-w24-batch13-coupon-baseline.ps1`
6. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
7. `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
8. `docs/function-lane/W24_ORDINARY_FUNCTIONS_MEGA_BATCH_CHECKLIST.csv`
9. `docs/function-lane/W16_BATCH69_COUPON_FUNCTIONS_NOTES.md`
10. `formal/lean/OxFunc/Functions/CouponFamily.lean`
11. `crates/oxfunc_core/src/functions/coupon_family.rs`
12. `docs/function-lane/W17_DEFERRED_LOW_INTEREST_INVENTORY.csv`
13. `docs/worksets/W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md`

## 5. Empirical Findings
From `.tmp/w24-batch13-coupon-results.csv`:
1. The regular semiannual actual/actual sample matched across all six functions.
2. The basis split matched the seeded bounded contract, including `COUPDAYBS(...,basis=0) -> 70` and `COUPDAYS(...,basis=3) -> 182.5`.
3. The quarterly end-of-month schedule matched the seeded clamp behavior.
4. Serial `0` is admitted as an early-date boundary for day/date-returning lanes: `COUPDAYBS(0,...) -> 0` and `COUPPCD(0,...) -> 0`.
5. `COUPNUM(0,...)` returns `#NUM!`, so the early-date boundary does not imply a valid coupon count before the first positive coupon date.
6. Settlement exactly on a coupon date advances to the following period: `COUPPCD(136,...) -> 136`, `COUPNCD(136,...) -> 320`, `COUPNUM(136,...) -> 223`.
7. Negative date serials and invalid frequencies return `#NUM!`.

## 6. Implementation Result
1. The family runtime and Lean binding were already integrated through dispatch/export/formal surfaces.
2. The packet exposed and corrected a real local bug in the coupon date-boundary logic.
3. The family is now packet-evidenced instead of remaining note-bounded.

## 7. Verification Runs
1. `powershell -ExecutionPolicy Bypass -File tools/w24-probe/run-w24-batch13-coupon-baseline.ps1`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml coupon_family`
3. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
4. `lake build`

## 8. Standing
1. The six coupon family members above are now function-phase-complete for the admitted current reference baseline.
2. The closure is bounded to the admitted regular coupon-schedule slice above.
3. `W024` continues with the remaining unblocked families after this packet.
