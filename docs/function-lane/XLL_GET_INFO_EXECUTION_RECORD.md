# XLL GET.INFO Wrapper Execution Record

Status: `complete-provisional`
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

## 4. Findings
1. the tester XLL now exposes working Rust-side worksheet-callable wrappers for the seeded `GET.*` surfaces.
2. the key wrapper fix was true macro-type registration via the `#` suffix in `type_text`; without that suffix the inner callback path returned `xlretInvXlfn`.
3. the `GET.CELL` native comparison lane in the runner also required translation from manifest `A1` references to working Excel4 `Sheet!R1C1` references before `ExecuteExcel4Macro(...)` would accept the call.
4. with those two fixes in place, all six seeded wrapper rows matched the native Excel4 results on the current host baseline.

## 5. Verification
1. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
2. `powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release`
3. `powershell -File tools/xll-addin/run-get-info-probe.ps1 -Manifest docs/function-lane/XLL_GET_INFO_SCENARIO_MANIFEST_SEED.csv -Out .tmp/xll-get-info-results.csv`
4. observed result: `6/6` matched

## 6. Standing
1. implementation status: wrappers present and parity-closed for the seeded lanes
2. empirical status: `scope_complete` for the current manifest slice
3. remaining work:
   - broaden beyond the seeded `GET.WORKSPACE(37)` / `GET.CELL` / `GET.WORKBOOK(1)` rows when the `CELL` / `GET.*` seam is expanded further
