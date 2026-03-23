# W46 Execution Record

## 1. Packet
1. workset: `W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM`
2. functions: `CALL`, `REGISTER.ID`
3. execution_state: `in_progress`

## 2. Objective
Move `W046` from catalog-only seam notes to a real OxFunc-side runtime packet:
1. ingest `XLCALL.H` built-in identities,
2. pin an admitted Excel baseline for `REGISTER.ID` / `CALL`,
3. add the minimal OxFunc runtime seam that normalizes requests and projects results while leaving raw external invocation above OxFunc.

## 3. Local Outputs Produced
1. `docs/function-lane/XLCALL_CODE_CATALOG.csv`
2. `docs/function-lane/FUNCTION_SLICE_CALL_REGISTER_ID_UDF_REGISTRATION_SEAM_PRELIM.md`
3. `docs/function-lane/W46_SCENARIO_MANIFEST_SEED.csv`
4. `docs/function-lane/W46_RUNTIME_REQUIREMENTS.md`
5. `docs/worksets/W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM.md`
6. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
7. `tools/w46-probe/generate-xlcall-code-catalog.ps1`
8. `tools/w46-probe/run-w46-call-register-id-baseline.ps1`
9. `.tmp/w46-call-register-id-results.csv`
10. `crates/oxfunc_core/src/functions/call_register_id_family.rs`
11. `formal/lean/OxFunc/Functions/CallRegisterIdFamily.lean`
12. `tools/xll-addin/oxfunc_xll/export_specs.csv`

## 4. Current OxFunc-Seam Result
Current OxFunc reading:
1. OxFunc is steward of the function registration catalog.
2. Built-in `xlf*` identities from `XLCALL.H` are catalog metadata for current built-in rows, not alternative primary ids.
3. `REGISTER.ID` normalizes a typed registration request and relies on host/OxFml to resolve it to a `RegisteredExternalDescriptor`.
4. `CALL` normalizes either:
   - a numeric register-id target, or
   - a direct `{ library, procedure, optional type_text }` target.
5. Host/OxFml owns raw Excel C API exposure and actual external invocation.
6. OxFunc owns only request normalization and worksheet result projection.

## 5. Verified Local Behavior
### 5.1 XLCALL Identity Layer
1. local Excel 2013 XLL SDK `XLCALL.H` is present under `.tmp/excelxllsdk_extracted/.../INCLUDE/XLCALL.H`.
2. the local generator parses callable `XLCALL.H` rows reproducibly:
   - `584` built-in `xlf*` rows
   - `403` `xlc*` command rows
   - `20` auxiliary `xlSpecial` callback rows
   - `1` `xlUDF` sentinel row
3. built-in `xlf*` rows are matched against the current OxFunc function catalog where possible.
4. the library-context snapshot exposes `xlcall_builtin_symbol` and `xlcall_builtin_code` for matched built-in rows such as `FUNC.CALL`, `FUNC.REGISTER.ID`, and ordinary built-ins like `FUNC.SUM`.

### 5.2 Native Excel Baseline
Pinned by `.tmp/w46-call-register-id-results.csv`:
1. `REGISTER.ID("Kernel32","GetTickCount","J!")` returns a numeric register id.
2. `CALL(register_id)` succeeds for seeded `GetTickCount`.
3. `CALL("Kernel32","GetTickCount","J!")` succeeds directly.
4. `CALL("Kernel32","MulDiv","JJJJ",6,7,3)` returns `14`.
5. `CALL(register_id,6,7,3)` returns `14` for the seeded `MulDiv` lane.
6. The seeded zero-argument `GetTickCount` lane also succeeds when `type_text` is omitted.

### 5.3 Core Runtime
1. `call_register_id_family.rs` defines:
   - `RegisterIdRequest`
   - `RegisteredExternalDescriptor`
   - `RegisteredExternalCallRequest`
   - `RegisteredExternalProvider`
2. OxFunc core now has executable `REGISTER.ID` and `CALL` runtime surfaces for the admitted slice.
3. `CALL` preserves trailing invocation args as raw `CallArgValue` so host/OxFml can own actual external call marshaling.

## 6. Verification Run
1. `powershell -ExecutionPolicy Bypass -File tools/w46-probe/generate-xlcall-code-catalog.ps1`
2. `powershell -ExecutionPolicy Bypass -File tools/w46-probe/run-w46-call-register-id-baseline.ps1`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml call_register_id_family -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
5. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
6. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`
7. `powershell -ExecutionPolicy Bypass -File tools/w44-probe/generate-w44-library-context-snapshot.ps1`
8. `lake build`

## 7. Status
1. scope_completeness: `scope_partial`
2. target_completeness: `target_partial`
3. integration_completeness: `partial`
4. open_lanes:
   - no host-backed `RegisteredExternalProvider` exists in-repo yet
   - broader argument-bearing omitted-`type_text` matrix is not pinned
   - worksheet-vs-macro-sheet admission/version matrix is not fully pinned
   - final registered-external runtime-snapshot row shape is not locked yet
