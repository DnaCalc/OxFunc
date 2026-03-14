# XLL GET.INFO Wrapper Execution Record

Status: `in_progress-provisional`
Evidence ID: `W9-XLL-GETINFO-20260314`

## 1. Purpose
Track reproducible evidence for the tester-XLL wrappers around legacy information/macro functions used to inspect locale/profile and cell metadata.

## 2. Scope
1. wrappers implemented in the tester XLL:
   - `ox_GET_CELL`
   - `ox_GET_DOCUMENT`
   - `ox_GET_WORKBOOK`
   - `ox_GET_WORKBOOK_ACTIVE`
   - `ox_GET_WORKSPACE`
2. currently exercised scalar lanes:
   - selected `GET.WORKSPACE(37)` items
   - selected `GET.CELL` items
   - active-workbook name via `GET.WORKBOOK(1)`

## 3. Artifacts
1. manifest: `docs/function-lane/XLL_GET_INFO_SCENARIO_MANIFEST_SEED.csv`
2. runner: `tools/xll-addin/run-get-info-probe.ps1`
3. XLL implementation: `tools/xll-addin/oxfunc_xll/src/lib.rs`
4. reference note: `docs/function-lane/FORMAT_SHIM_AND_GET_INFO_REFERENCE_NOTES.md`

## 4. Current Findings
1. the tester XLL now exposes Rust-side worksheet-callable wrappers for the selected `GET.*` surfaces.
2. those wrappers are not yet parity-closed: the current baseline replay returns `#VALUE!` for the exercised `ox_GET_WORKSPACE(...)` and `ox_GET_WORKBOOK_ACTIVE(1)` rows.
3. the native `GET.CELL(...)` comparison rows are also not yet trustworthy in the current probe runner, because `ExecuteExcel4Macro("GET.CELL(...)")` failed with a formula-parser complaint on the seeded reference argument rows.
4. `GET.WORKSPACE(37)` therefore remains useful as reference material and as a direct native host-observation tool, but not yet as a parity-closed tester-XLL wrapper lane.

## 5. Verification
1. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
2. `powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release`
3. `powershell -File tools/xll-addin/run-get-info-probe.ps1 -Manifest docs/function-lane/XLL_GET_INFO_SCENARIO_MANIFEST_SEED.csv -Out .tmp/xll-get-info-results.csv`

## 6. Standing
1. implementation status: wrappers present in the tester XLL
2. empirical status: `scope_partial`
3. next work:
   - determine why the inner `xlfGetWorkspace` / `xlfGetWorkbook` callback path collapses to `#VALUE!` from the Rust wrapper even though the worksheet registration is callable
   - harden or replace the native `GET.CELL` comparison path in the probe runner
