# FP-C XLL Ops Notes

## 1. Purpose
Operational runbook for the W2 FP-C harness build and registration path.

## 2. SDK Bootstrap
1. Lock metadata:
   - `EXCEL_XLL_SDK_LOCK.json`
2. Fetch and verify SDK:
```powershell
powershell -File tools/fp-probe/xll/fetch-excel-xll-sdk.ps1
```
3. Optional bypass (only for controlled debugging):
```powershell
powershell -File tools/fp-probe/xll/fetch-excel-xll-sdk.ps1 -SkipHashCheck
```
4. Controlled lock rotation (editorial + operational review required):
```powershell
powershell -File tools/fp-probe/xll/fetch-excel-xll-sdk.ps1 -UpdateLock
```
   - updates `EXCEL_XLL_SDK_LOCK.json` when source hash drifts.

## 3. Build
1. Default (uses local extracted SDK path convention):
```powershell
powershell -File tools/fp-probe/xll/build-fp-edge-xll.ps1 -Profile release
```
2. Explicit SDK location:
```powershell
powershell -File tools/fp-probe/xll/build-fp-edge-xll.ps1 -Profile release -ExcelXllSdkDir "<path-to-Excel2013XLLSDK>"
```

## 4. Registration Model
1. The XLL self-registers functions in `xlAutoOpen`.
2. Registration call path is `xlfRegister` via official SDK callback bridge (`XLCALL.H` + `XLCALL.CPP`).
3. Probe runner calls only `RegisterXLL` and does not script-register function names.
