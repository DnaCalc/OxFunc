# W46 Execution Record

## 1. Packet
1. workset: `W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM`
2. functions: `CALL`, `REGISTER.ID`
3. execution_state: `in_progress`

## 2. Objective
Use the local `XLCALL.H` source to ingest Excel C API built-in function identities into the OxFunc catalog, then define the minimal registration/catalog seam needed for future worksheet `CALL` / `REGISTER.ID` work.

## 3. Local Outputs Produced
1. `docs/function-lane/XLCALL_CODE_CATALOG.csv`
2. `docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md`
3. `docs/worksets/W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM.md`
4. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
5. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
6. `docs/upstream/NOTES_FOR_OXFML.md`
7. `tools/w46-probe/generate-xlcall-code-catalog.ps1`

## 4. Current OxFunc-Seam Result
Current OxFunc reading:
1. OxFunc is steward of the function registration catalog.
2. Built-in `xlf*` identities from `XLCALL.H` are now catalog metadata for current built-in function rows, not alternative primary ids.
3. Host owns raw Excel C API exposure and external invocation.
4. OxFunc should receive stable-id-oriented prepared calls after built-in dispatch is resolved, and later registered-external descriptors once registration support is added.

## 5. Verified Local Behavior
1. local Excel 2013 XLL SDK `XLCALL.H` is present under `.tmp/excelxllsdk_extracted/.../INCLUDE/XLCALL.H`.
2. the local generator parses callable `XLCALL.H` rows reproducibly:
   - `584` built-in `xlf*` rows
   - `403` `xlc*` command rows
   - `20` auxiliary `xlSpecial` callback rows
   - `1` `xlUDF` sentinel row
3. built-in `xlf*` rows are matched against the current OxFunc function catalog where possible.
4. the library-context snapshot now exposes `xlcall_builtin_symbol` and `xlcall_builtin_code` for matched built-in rows such as `FUNC.CALL`, `FUNC.REGISTER.ID`, and ordinary built-ins like `FUNC.SUM`.

## 6. Verification Run
1. `powershell -ExecutionPolicy Bypass -File tools/w46-probe/generate-xlcall-code-catalog.ps1`
2. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`

## 7. Status
1. scope_completeness: `scope_partial`
2. target_completeness: `target_partial`
3. integration_completeness: `partial`
4. open_lanes:
   - no empirical worksheet replay packet yet for `CALL` / `REGISTER.ID`
   - no final registered-external descriptor shape yet
   - no host-backed invocation/runtime implementation yet
