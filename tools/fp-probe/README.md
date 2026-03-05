# FP Probe Tooling

This folder contains runnable W2 probe tooling for Excel and Lean comparison lanes.

## Layout
1. `fp_probe_runner/`
   - Rust CLI that validates a scenario manifest and writes queued execution rows.
2. `run-fp-probe.ps1`
   - PowerShell wrapper around the Rust queue runner.
3. `run-fp-excel-baseline.ps1`
   - PowerShell COM runner for Excel lanes (`FP-A`, `FP-B`, `FP-D`, optional `FP-C` with XLL path).
4. `run-fp-lean-baseline.ps1`
   - PowerShell runner for Lean comparative rows (`lean-runtime` scenarios).
5. `stitch-fp-deviation-ledger.ps1`
   - Updates `FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv` from Excel+Lean results.
6. `run-fp-suite.ps1`
   - One-command orchestration: Excel (`A/B/D`), XLL build + Excel (`C`), Lean run, ledger stitching.
7. `xll/FpEdgeHarnessContract.md`
   - FP-C contract and behavior expectations.
8. `xll/fp_edge_xll/`
   - Rust `cdylib` XLL harness with C ABI exports for edge-value injection and in-XLL `xlfRegister` self-registration.
9. `xll/build-fp-edge-xll.ps1`
   - Builds the Rust XLL and emits `xll/bin/fp_edge_xll.xll`.
10. `xll/fetch-excel-xll-sdk.ps1`
   - Downloads and extracts the official Microsoft Excel XLL SDK used by the harness build.
11. `xll/EXCEL_XLL_SDK_LOCK.json`
   - source/hash lock for SDK bootstrap reproducibility.
   - lock can be rotated intentionally via `fetch-excel-xll-sdk.ps1 -UpdateLock`.
12. `xll/README.md`
   - FP-C operational runbook.
13. `results/FLOATING_POINT_RESULTS_TEMPLATE.csv`
   - schema template for result capture.

## Core Commands
1. Queue output preparation:
```powershell
powershell -File tools/fp-probe/run-fp-probe.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-prepare.csv -Mode prepare
```
2. Excel baseline (`FP-A/B/D`):
```powershell
powershell -File tools/fp-probe/run-fp-excel-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-excel-abd.csv
```
3. Excel interop lane (`FP-C`):
```powershell
powershell -File tools/fp-probe/xll/fetch-excel-xll-sdk.ps1
powershell -File tools/fp-probe/xll/build-fp-edge-xll.ps1 -Profile release
powershell -File tools/fp-probe/run-fp-excel-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-excel-c.csv -Lanes FP-C -XllPath tools/fp-probe/xll/bin/fp_edge_xll.xll
```
4. Lean comparative run:
```powershell
powershell -File tools/fp-probe/run-fp-lean-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-lean.csv
```
5. Full suite:
```powershell
powershell -File tools/fp-probe/run-fp-suite.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -OutDir .tmp
```

## Notes
1. `run-fp-excel-baseline.ps1` consumes explicit `op_*` manifest fields first, with legacy fallback parsing retained for compatibility.
2. `FP-C` execution requires explicit `-XllPath`; when absent or registration fails, rows are marked `blocked_missing_xll` with a reason.
3. Result rows include richer captures (`primary_formula2`, `primary_value2`, `primary_text`, `observed_cells`, `comparison_bools`) for reproducible mismatch analysis.
