# Floating-Point Probe Runtime Requirements

Status: `draft`
Workset: `W2`

## 1. Purpose
Define the runtime prerequisites and minimal operating contract for executing floating-point probe scenarios.

## 2. Required Inputs
1. Scenario manifest CSV:
   - `docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv`
2. Probe runner scaffold:
   - `tools/fp-probe/fp_probe_runner/`
3. Results output path:
   - caller-provided CSV path.

## 3. Environment Requirements
1. Windows desktop Excel installation (local installed Excel instance).
2. Ability to capture exact Excel build/channel per run.
3. Workbook Compatibility Version capture workflow (starting baseline: default workbook setting).
4. Locale baseline: `en-US` only for this phase.
5. For probe tooling implementation:
   - non-script code in this repo should be Rust.

## 4. XLL Harness Requirements (FP-C Lane)
1. Implement contract in:
   - `tools/fp-probe/xll/FpEdgeHarnessContract.md`
2. Required function coverage:
   - `OXFP_NEG_ZERO`
   - `OXFP_POS_INF`
   - `OXFP_NEG_INF`
   - `OXFP_QNAN`
   - `OXFP_SNAN`
3. Build path:
   - native XLL with simple C ABI exports,
   - based on official Excel SDK headers (`xlcall.h` and related),
   - avoid external helper libraries for core interop behavior.

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
12. `notes`

Template:
1. `tools/fp-probe/results/FLOATING_POINT_RESULTS_TEMPLATE.csv`

## 7. Current Limitations
1. Current execution tooling is scaffold-only and does not yet drive Excel runs end-to-end.
2. `FP-C` scenarios remain blocked pending harness implementation.
3. Observation capture and promotion (`EMP-*`) remain downstream tasks.
