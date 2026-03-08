# XLL Add-in Bridge Shim Contract (Prelim)

Status: `provisional`
Workset: `W9`

## 1. Purpose
Define the seed caller/shim contract for mapping XLL entrypoint arguments into OxFunc function calls without coupling core semantics to XLL ABI details.

## 2. Bridge Components
1. Rust-only XLL transport, registration, and type-conversion layer:
   - `tools/xll-addin/oxfunc_xll/src/lib.rs`
2. Generated export/registration layer:
   - `tools/xll-addin/oxfunc_xll/build.rs`
3. Core source-of-truth for export rows:
   - `crates/oxfunc_core/src/xll_export_specs.rs`
4. Generated CSV snapshot (for review/replay artifacts):
   - `tools/xll-addin/oxfunc_xll/export_specs.csv`
5. Core dispatch/semantics owner:
   - `crates/oxfunc_core/src/functions/surface_dispatch.rs`
6. Function kernels/adapters owner:
   - `crates/oxfunc_core`

## 3. Seed Export Set
1. `ox_ABS`:
   - registration export: `OX_ABS`
   - registration type text: `QU`
   - worksheet function name: `ox_ABS`
2. `ox_ABS_Q`:
   - registration export: `OX_ABS_Q`
   - registration type text: `BB`
   - worksheet function name: `ox_ABS_Q`
3. `ox_PI`:
   - registration export: `OX_PI`
   - registration type text: `B`
   - worksheet function name: `ox_PI`

## 4. Shim Mapping (Seed Scope)
Bridge policy is declarative per export row:
1. registration row (`export_name`, `worksheet_name`, `type_text`, `arg_names`),
2. bound `function_id` (for core dispatch),
3. U-surface lift policy (`scalar_only` or `unary_scalar_or_array_elementwise`).
4. export wrapper signature kind (`u_unary`, `q_unary_number`, `q_nullary_number`).

For `U` surface rows:
1. Incoming `LPXLOPER12` categories handled by generic converter:
   - `xltypeNum` -> number,
   - `xltypeStr` -> UTF-16 text (pascal length-prefixed),
   - `xltypeBool` -> logical,
   - `xltypeErr` -> worksheet error code,
   - `xltypeMissing` -> missing arg,
   - `xltypeNil` -> empty cell.
2. Reference lanes (generic):
   - `xltypeRef` / `xltypeSRef` are dereferenced through `xlCoerce` before shim translation.
3. `xltypeMulti` array lanes (policy-driven):
   - for `unary_scalar_or_array_elementwise`, shape-preserving elementwise scalar dispatch by `function_id`.
   - result returned as `xltypeMulti` with Excel-owned lifetime (`xlbitDLLFree` + `xlAutoFree12`).
4. Core invocation is by `function_id` through core dispatch entrypoints, not function-specific bridge logic.

For `Q` surface rows:
1. numeric unary and nullary calls are routed by `function_id` through core dispatch entrypoints.

## 5. Error Mapping (Seed)
1. OxFunc worksheet errors map to XLL `xlerr*` codes (`#DIV/0!`, `#VALUE!`, `#N/A`, etc.).
2. Coercion/ref-resolution/unsupported shim cases default to `#VALUE!` unless a specific worksheet error is already declared.

## 6. Non-Goals in Seed
1. Full multi-arity wrapper generation beyond current supported entry kinds (`u_unary`, `q_unary_number`, `q_nullary_number`).
2. Asynchronous/RTD callback model.
3. Production-hardening beyond current `xlbitDLLFree` + `xlAutoFree12` ownership discipline.
