# OxFunc XLL Add-in Bridge Ops Notes

## 1. Purpose
Operational runbook for building and replaying the W9 `OxFunc64.xll` bridge.

## 2. Build
0. Sync generated export specs from `oxfunc_core` (optional if you run build script default path):
```powershell
powershell -File tools/xll-addin/sync-export-specs.ps1
```
1. Default build:
```powershell
powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release
```
2. Skip spec sync for faster local rebuilds:
```powershell
powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release -SkipSpecSync
```

## 3. Bridge Baseline Replay
1. Build-if-missing plus replay:
```powershell
powershell -File tools/xll-addin/run-oxfunc-xll-bridge-baseline.ps1 -Manifest docs/function-lane/XLL_ADDIN_BRIDGE_VALIDATION_SCENARIO_MANIFEST_SEED.csv -Out .tmp/oxfunc-xll-bridge-results.csv -BuildIfMissing
```
2. Explicit XLL path:
```powershell
powershell -File tools/xll-addin/run-oxfunc-xll-bridge-baseline.ps1 -Manifest docs/function-lane/XLL_ADDIN_BRIDGE_VALIDATION_SCENARIO_MANIFEST_SEED.csv -Out .tmp/oxfunc-xll-bridge-results.csv -XllPath tools/xll-addin/bin/OxFunc64.xll
```

## 4. Registration Model
1. The XLL self-registers in `xlAutoOpen`.
2. Registration call path is `xlfRegister` via direct Rust `MdCallBack12` callback binding.
3. Source-of-truth export rows live in `oxfunc_core`:
   - `crates/oxfunc_core/src/xll_export_specs.rs`
4. Generated bridge wrappers and registration rows are emitted by:
   - `tools/xll-addin/oxfunc_xll/build.rs`
5. Generated CSV snapshot for review:
   - `tools/xll-addin/oxfunc_xll/export_specs.csv`
6. Seed exports:
   - `ox_ABS` (`U` argument surface through `OX_ABS` bridge wrapper),
   - `ox_ABS_Q` (`B` numeric scalar surface through `OX_ABS_Q`),
   - `ox_PI` (`B` nullary constant surface through `OX_PI`).
