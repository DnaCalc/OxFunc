# W12 Execution Record

Status: `in_progress-provisional`
Workset: `W12`
Evidence ID: `W12-MODERATE-BL-20260309`

## 1. Purpose
Track W12 execution status, artifacts, and gate closure for the moderate fifteen-function expansion packet.

## 2. Scope
1. functions: `AVERAGE`, `COUNT`, `COUNTA`, `IFERROR`, `ROUND`, `TEXTJOIN`, `TODAY`, `RAND`, `OFFSET`, `CELL`, `AND`, `CLEAN`, `DATE`, `EXACT`, `HSTACK`.
2. include W11 follow-back evidence hooks for volatile/thread-safe/macro lanes.
3. run `CELL` empirical probe pack before substantial `CELL` implementation.

## 3. Completeness Axes
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - multiple W12 function implementations remain work-in-progress until known Excel-semantic gaps are closed for the declared version axes.
   - aggregate direct-scalar versus array-like policy remains explicit target follow-up for `AVERAGE`/`COUNT`/`COUNTA`.
   - `TEXTJOIN` array flattening and richer formatting/text coercion lanes remain explicit target follow-up.
   - `OFFSET`/`CELL` retain bounded A1-only reference scope and are not yet full caller-context/macro closure.
   - `HSTACK` remains shape-only in OxFunc runtime; payload fill/padding is deferred.
   - W11 registration-flag mapping remains deferred; W12 only contributes stronger volatile and caller-context candidate scenarios.
6. function-phase-complete slices within W12:
   - `TODAY`

## 4. Executed Scope
Execution date:
1. `2026-03-09`
2. `2026-03-10` (TODAY format-hint follow-up and function-phase-complete promotion)
3. `2026-03-10` (W12 replay rerun after format-hint/range-observable expansion)

Function slices with landed scaffolding/runtime seeds:
1. `AVERAGE`
2. `COUNT`
3. `COUNTA`
4. `IFERROR`
5. `ROUND`
6. `TEXTJOIN`
7. `TODAY`
8. `RAND`
9. `OFFSET`
10. `CELL`
11. `AND`
12. `CLEAN`
13. `DATE`
14. `EXACT`
15. `HSTACK`

## 5. Output Artifacts
1. function contracts:
   - `docs/function-lane/FUNCTION_SLICE_AVERAGE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_COUNT_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_COUNTA_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_IFERROR_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_ROUND_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_TEXTJOIN_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_TODAY_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_RAND_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_OFFSET_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_CELL_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_AND_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_CLEAN_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_DATE_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_EXACT_CONTRACT_PRELIM.md`
   - `docs/function-lane/FUNCTION_SLICE_HSTACK_CONTRACT_PRELIM.md`
2. scenario manifests and probe requirements:
   - `docs/function-lane/W12_CELL_PRE_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W12_S1_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W12_S2_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W12_S3_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W12_S4_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W12_S5_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W12_S6_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W12_PROBE_RUNTIME_REQUIREMENTS.md`
3. runtime + formal modules:
   - Rust: `crates/oxfunc_core/src/functions/*`
   - Lean: `formal/lean/OxFunc/Functions/*`
4. side-note ledger:
   - `docs/function-lane/W12_PROFILE_SYSTEM_SIDE_NOTES.md`
5. empirical outputs:
   - `.tmp/w12-cell-pre-results-default.csv`
   - `.tmp/w12-cell-pre-results-compat.csv`
   - `.tmp/w12-cell-pre-results-excel.csv`
   - `.tmp/w12-cell-pre-analysis-report.csv`
   - `.tmp/w12-cell-pre-analysis-summary.json`
   - `.tmp/w12-results-default.csv`
   - `.tmp/w12-results-compat.csv`
   - `.tmp/w12-results-excel.csv`
   - `.tmp/w12-analysis-report.csv`
   - `.tmp/w12-analysis-summary.json`
6. replay tooling:
   - `tools/w12-probe/run-w12-cell-preprobe.ps1`
   - `tools/w12-probe/run-w12-suite.ps1`
   - `tools/w12-probe/analyze-w12-results.ps1`
   - `tools/w12-probe/new-w12-compat-template.ps1`

## 6. Verification Runs
1. `cargo test -p oxfunc_core` -> pass (`161` tests).
2. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml` -> pass.
3. `powershell -File tools/xll-addin/sync-export-specs.ps1` -> pass.
4. `lake build` (from `formal/lean`) -> pass.
5. `powershell -File tools/w12-probe/run-w12-cell-preprobe.ps1 -OutDir .tmp` -> pass.
6. `powershell -File tools/w12-probe/run-w12-suite.ps1 -OutDir .tmp` -> pass.
7. `powershell -File tools/function-lane-check/run-correlation-integrity-check.ps1` -> pass.

## 7. Gate Tracking
### G1 - Classification Closure
1. Status: `closed-provisional`.
2. Notes:
   - all fifteen W12 slices now carry complete declared-scope contract fields in contract docs and runtime metadata, including explicit coercion, core outcome, post-call adaptation, both required version axes, and evidence posture bindings.

### G2 - Runtime/Formal Pairing Closure
1. Status: `closed-provisional`.
2. Notes:
   - each W12 function now has a Rust module and Lean module.
   - bounded seed notes remain explicit for aggregate argument-structure follow-up, caller-context, and dynamic-array payload lanes.

### G3 - Empirical Closure
1. Status: `closed-provisional`.
2. Notes:
   - `CELL` preprobe dual-run executed before W12 promotion closure:
     - rows: `10` (`5` default + `5` compat_template).
     - expectation matched: `10`; mismatched: `0`.
     - analyzer gate status: `green`.
   - W12 suite dual-run rerun on `2026-03-10`:
     - rows: `34` (`17` default + `17` compat_template).
     - expectation matched: `34`; mismatched: `0`.
     - execution failed unexpected: `0`.
     - dual-run requirement satisfied: `true`.
     - analyzer gate status: `green`.

### G4 - W11 Follow-back Readiness
1. Status: `closed-provisional`.
2. Notes:
   - W12 now contributes explicit volatile follow-back candidates:
     - `TODAY` (`W12S4-001`, `W12S4-002`)
     - `RAND` (`W12S4-003`)
   - W12 now contributes explicit caller-context/macro-sensitive candidates:
     - `CELL` (`W12CELL-001..005`, `W12S5-002`, `W12S5-003`)
     - `OFFSET` (`W12S5-001`)

### G5 - Promotion Readiness
1. Status: `in_progress`.
2. Notes:
   - W12 has local scaffolding closure across contract, Rust, Lean, and empirical seed lanes.
   - W12 does not satisfy packet-level implementation closure because known Excel-semantic gaps remain across multiple functions.
   - `TODAY` now satisfies current-phase function closure individually and may be reported as `function-phase-complete`.

## 8. Key Findings
1. W10’s declarative-runner/default-surface posture scaled to a moderate breadth packet without requiring broad special-case dispatcher growth.
2. `CELL` required an empirical-first narrowing pass before broader implementation work; the dual-run preprobe usefully selected the next semantic lanes without constituting closure.
3. provider seams are now standardized across `NOW`, `TODAY`, and `RAND`, which gives W11 a better volatile follow-back matrix.
4. `OFFSET` and `CELL` justified a small shared A1 parse/format helper but also exposed the remaining caller-context/reference-shape work still needed for full parity.
5. count-family and average-family aggregate semantics remain the clearest evidence that direct-scalar versus array-like aggregate inputs need first-class representation before these functions can be treated as implemented; any finer source-class distinction should be justified empirically rather than assumed.
6. `TODAY` now has current-phase closure evidence across provider floor semantics, volatile recalc behavior, and caller-cell format-hinting, while remaining XLL control-alias work stays external to function semantics.
7. `RAND` replay now asserts the worksheet-visible numeric range contract (`0 <= RAND() < 1`) directly in the W12 suite, while W11 carries the separate ordinary volatile-registration follow-back lane.
