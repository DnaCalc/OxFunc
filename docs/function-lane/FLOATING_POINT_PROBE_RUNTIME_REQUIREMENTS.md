# Floating-Point Probe Runtime Requirements

Status: `active`
Workset: `W2`

## 1. Purpose
Define the runtime prerequisites and minimal operating contract for executing floating-point probe scenarios.

## 2. Required Inputs
1. Scenario manifest CSV:
   - `docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv`
2. Probe runners:
   - `tools/fp-probe/fp_probe_runner/`
   - `tools/fp-probe/run-fp-excel-baseline.ps1`
   - `tools/fp-probe/run-fp-lean-baseline.ps1`
   - `tools/fp-probe/stitch-fp-deviation-ledger.ps1`
   - `tools/fp-probe/run-fp-suite.ps1`
3. Results output path:
   - caller-provided CSV path.

## 3. Environment Requirements
1. Windows desktop Excel installation (local installed Excel instance).
2. Ability to capture exact Excel build/channel per run.
3. Workbook Compatibility Version capture workflow:
   - default workbook setting baseline (captured on each row),
   - optional template-driven override via `run-fp-excel-baseline.ps1 -WorkbookTemplate <path>`.
4. Locale baseline: `en-US` only for this phase.
5. For probe tooling implementation:
   - non-script code in this repo should be Rust.

## 4. XLL Harness Requirements (FP-C Lane)
1. Implemented contract in:
   - `tools/fp-probe/xll/FpEdgeHarnessContract.md`
2. Required function coverage:
   - `OXFP_NEG_ZERO`
   - `OXFP_POS_INF`
   - `OXFP_NEG_INF`
   - `OXFP_QNAN`
   - `OXFP_SNAN`
   - optional helper: `OXFP_BITS_ECHO`
3. Build path:
   - native XLL with simple C ABI exports,
   - self-registration in `xlAutoOpen` using Excel C API callback path (`xlfRegister` via official SDK `XLCALL.H`/`XLCALL.CPP`),
   - avoid external helper libraries for core interop behavior.
4. Build command:
```powershell
powershell -File tools/fp-probe/xll/build-fp-edge-xll.ps1 -Profile release
```
5. SDK prerequisite:
```powershell
powershell -File tools/fp-probe/xll/fetch-excel-xll-sdk.ps1
```
6. Optional explicit SDK path:
```powershell
powershell -File tools/fp-probe/xll/build-fp-edge-xll.ps1 -Profile release -ExcelXllSdkDir "<path-to-Excel2013XLLSDK>"
```

## 5. Minimal Execution Commands
1. Rust runner:
```powershell
cargo run --manifest-path tools/fp-probe/fp_probe_runner/Cargo.toml -- --manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv --out .tmp/fp-results.csv --mode dry-run
```
2. PowerShell wrapper:
```powershell
powershell -File tools/fp-probe/run-fp-probe.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results.csv -Mode dry-run
```
3. Tooling must emit schema-compatible CSV output and include run provenance.
4. Excel baseline (`FP-A/B/D`):
```powershell
powershell -File tools/fp-probe/run-fp-excel-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-excel-abd.csv
```
5. Excel interop lane (`FP-C`):
```powershell
powershell -File tools/fp-probe/run-fp-excel-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-excel-c.csv -Lanes FP-C -XllPath tools/fp-probe/xll/bin/fp_edge_xll.xll
```
6. Lean comparative run:
```powershell
powershell -File tools/fp-probe/run-fp-lean-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-lean.csv
```
7. Full suite:
```powershell
powershell -File tools/fp-probe/run-fp-suite.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```

## 6. Output Contract
Output CSV columns:
1. `scenario_id`
2. `lane`
3. `mode`
4. `execution_status`
5. `observed_class`
6. `excel_version`
7. `excel_channel`
8. `compat_version`
9. `locale_profile`
10. `runner_version`
11. `artifact_ref`
12. `primary_cell`
13. `primary_formula2`
14. `primary_value2`
15. `primary_text`
16. `observed_cells`
17. `comparison_bools`
18. `notes`

Template:
1. `tools/fp-probe/results/FLOATING_POINT_RESULTS_TEMPLATE.csv`

## 7. Current Limitations
1. Baseline run set currently records one locale (`en-US`) and one local Excel build/channel baseline.
2. Workbook Compatibility Version coverage beyond default requires explicit template-driven reruns.
3. Promotion into Foundation-level `EMP-*` evidence registry remains a downstream editorial step.
