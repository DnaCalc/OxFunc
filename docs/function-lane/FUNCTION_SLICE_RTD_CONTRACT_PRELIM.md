# FUNCTION SLICE - RTD Contract Prelim

## 1. Purpose
Pin the current OxFunc-local semantic contract for `RTD` without collapsing COM server activation and topic lifecycle machinery into OxFunc.

## 2. Current-Baseline Surface
Worksheet surface:
1. `RTD(prog_id, server_name, topic1, [topic2], ...)`

Current admitted OxFunc-local request shape:
1. `prog_id: text`
2. `server_name: text`
3. `topic_strings: ordered text vector`

Arity:
1. minimum `3`
2. maximum `255`

## 3. OxFunc-Owned Semantics
OxFunc owns:
1. arity admission,
2. values-only argument preparation,
3. text coercion of `prog_id`, `server_name`, and all topic arguments,
4. ordered preservation of the topic-string payload,
5. result projection from a host-supplied provider outcome into the worksheet value/error universe.

## 4. Host / OxFml / Application-Owned Machinery
OxFunc does not own:
1. COM activation of the `IRtdServer`,
2. topic subscription tables,
3. topic lifetime tracking,
4. callback threading and `UpdateNotify`,
5. workbook/cell subscription maps,
6. recalculation triggering policy,
7. saved-value / reconnect lifecycle.

Those responsibilities stay above OxFunc, between OxFml and the higher-level host application.

## 5. Candidate Minimal OxFml <-> OxFunc Interface
Current OxFunc-local interface direction:
1. `RtdRequest`
   - `prog_id`
   - `server_name`
   - `topic_strings`
2. `RtdProvider`
   - typed host callback for resolving the current RTD result for a prepared request
3. `RtdProviderResult`
   - `Value(EvalValue)`
   - `NoValueYet`
   - `CapabilityDenied`
   - `ConnectionFailed`
   - `ProviderError(WorksheetErrorCode)`

This is a best-attempt local seam design, not a locked cross-repo ABI.

## 6. Current Result Mapping
Current OxFunc-local worksheet projection:
1. `Value(v)` -> `v`
2. `NoValueYet` -> `#N/A`
3. `CapabilityDenied` -> `#BLOCKED!`
4. `ConnectionFailed` -> `#CONNECT!`
5. `ProviderError(code)` -> `code`

Current local admission/runtime failures:
1. arity mismatch -> `#VALUE!`
2. text coercion failure -> propagated worksheet error when present, otherwise `#VALUE!`
3. no provider wired -> `#VALUE!`

## 7. Evidence
Code:
1. `crates/oxfunc_core/src/functions/rtd_fn.rs`
2. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
3. `crates/oxfunc_core/src/xll_export_specs.rs`

Tests and exercised surfaces:
1. unit tests in `rtd_fn.rs`
2. export-catalog test in `xll_export_specs.rs`
3. dispatch-path regression in `surface_dispatch.rs`

Reference captures:
1. `RTD_REFERENCE_CAPTURE_AND_SEAM_NOTES.md`

## 8. Known Limits
1. This contract does not prove the exact current Excel mapping for every live-server startup/disconnect edge case.
2. The plain XLL bridge in this repo does not supply a real RTD provider, so end-to-end RTD host replay remains above the current OxFunc test seam.
3. Workbook-saved-value and reconnect semantics remain unmodeled in OxFunc.
