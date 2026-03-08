# OxFunc XLL Add-in Bridge Ops Notes

## 1. Purpose
Operational runbook for building and replaying the W9 `OxFunc64.xll` bridge.

## 2. Build
1. Default build (uses local extracted SDK path convention):
```powershell
powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release
```
2. Explicit SDK location:
```powershell
powershell -File tools/xll-addin/build-oxfunc-xll.ps1 -Profile release -ExcelXllSdkDir "<path-to-Excel2013XLLSDK>"
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
2. Registration call path is `xlfRegister` via official SDK callback bridge (`XLCALL.H` + `XLCALL.CPP`).
3. Seed exports:
   - `ox_ABS` (`U` argument surface through `OX_ABS` bridge wrapper),
   - `ox_ABS_Q` (`B` numeric scalar surface through `OX_ABS_Q`),
   - `ox_PI` (`B` nullary constant surface through `OX_PI`).
