# W43 Execution Record

## 1. Packet
1. workset: `W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM`
2. function: `RTD`
3. execution_state: `in_progress`

## 2. Objective
Establish the minimal honest OxFunc role for `RTD`, then implement that local seam without taking ownership of COM activation, topic lifetime tracking, or host-driven recalc/update machinery.

## 3. Local Outputs Produced
Reference/seam surfaces:
1. `docs/function-lane/RTD_REFERENCE_CAPTURE_AND_SEAM_NOTES.md`
2. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`
3. `docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md`
4. `docs/function-lane/W43_RTD_COM_AND_TOPIC_LIFECYCLE_INVENTORY.csv`

Core implementation:
1. `crates/oxfunc_core/src/functions/rtd_fn.rs`
2. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
3. `crates/oxfunc_core/src/xll_export_specs.rs`
4. `tools/xll-addin/oxfunc_xll/export_specs.csv`

## 4. Implemented OxFunc-Local Interface
Current best-attempt local interface:
1. `RtdRequest`
   - `prog_id`
   - `server_name`
   - `topic_strings`
2. `RtdProvider`
   - host-supplied resolution callback
3. `RtdProviderResult`
   - `Value`
   - `NoValueYet`
   - `CapabilityDenied`
   - `ConnectionFailed`
   - `ProviderError`

This lets OxFunc:
1. admit and normalize the `RTD` call,
2. preserve the ordered topic payload,
3. project the host/provider outcome into worksheet values and errors.

## 5. Verified Local Behavior
Verified by tests:
1. `prog_id`, `server_name`, and ordered topic strings are preserved.
2. numbers and blanks in topic positions coerce to text.
3. provider-supplied scalar and array payloads pass through into the worksheet value universe.
4. `NoValueYet` maps to `#N/A`.
5. `CapabilityDenied` maps to `#BLOCKED!`.
6. `ConnectionFailed` maps to `#CONNECT!`.
7. missing provider wiring currently maps to `#VALUE!`.
8. `RTD` is present in the export catalog and dispatch table.

## 6. Verification Run
1. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml rtd_fn -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml eval_surface_value_call_with_callable_supports_map_helper_surface -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml all_catalog_functions_have_at_least_one_export -- --nocapture`
5. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
6. `powershell -ExecutionPolicy Bypass -File tools/xll-addin/sync-export-specs.ps1`

## 7. Status
1. scope_completeness: `scope_partial`
2. target_completeness: `target_partial`
3. integration_completeness: `partial`
4. open_lanes:
   - exact current-baseline Excel outcome matrix for live-server startup/disconnect/save-value lanes is not yet empirically pinned
   - no dedicated native RTD worksheet replay packet exists yet
   - the cross-repo OxFml <-> OxFunc RTD handoff note is not yet written

## 8. Not Open Lanes
The following are not counted as OxFunc-local open semantic lanes:
1. COM activation
2. topic registration ownership
3. callback threading
4. update notification scheduling
5. host-side cell subscription maps
6. plain XLL bridge inability to stand up a real RTD host provider

Those are intentionally above OxFunc or outside the current harness seam.
