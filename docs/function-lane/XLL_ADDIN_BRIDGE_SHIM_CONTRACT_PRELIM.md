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

## 3. Export Set (Profile-Derived)
1. Export rows are generated for every function in the OxFunc function catalog.
2. U-vs-Q variants are derived by rule from `FunctionMeta` profile fields in `crates/oxfunc_core/src/xll_export_specs.rs`.
3. Current generated snapshot is `tools/xll-addin/oxfunc_xll/export_specs.csv` (for example `OX_ABS`, `OX_IF`, `OX_SUM`, `OX_XLOOKUP`, `OX_XMATCH`, with profile-admitted Q variants such as `OX_ABS_Q`, `OX_OP_ADD_Q`, `OX_PI`).

## 4. Shim Mapping (Current Scope)
Bridge policy is declarative per export row:
1. registration row (`export_name`, `worksheet_name`, `type_text`, `arg_names`),
2. bound `function_id` (for core dispatch),
3. U-surface lift policy (`scalar_only` or `unary_scalar_or_array_elementwise`),
4. reference-preservation flag (`preserve_refs`) derived from `arg_preparation_profile`,
5. export wrapper signature kind (`u_arity_N`, `q_unary_number`, `q_binary_number`, `q_nullary_number`).

For `U` surface rows:
1. Incoming `LPXLOPER12` categories handled by generic converter:
   - `xltypeNum` -> number,
   - `xltypeInt` -> number,
   - `xltypeStr` -> UTF-16 text (pascal length-prefixed),
   - `xltypeBool` -> logical,
   - `xltypeErr` -> worksheet error code,
   - `xltypeMissing` -> missing arg,
   - `xltypeNil` -> empty cell,
   - `xltypeMulti` -> shape-only array marker.
2. Reference lanes (generic):
   - if `preserve_refs=false`, `xltypeRef` / `xltypeSRef` are dereferenced through `xlCoerce` before shim translation.
   - if `preserve_refs=true`, reference-like tokens are passed through as `CallArgValue::Reference`.
3. `xltypeMulti` array lanes (policy-driven):
   - for `unary_scalar_or_array_elementwise`, shape-preserving elementwise scalar dispatch by `function_id`.
   - result returned as `xltypeMulti` with Excel-owned lifetime (`xlbitDLLFree` + `xlAutoFree12`).
4. Core invocation is by `function_id` through core dispatch entrypoints, not function-specific bridge logic.
5. Result mapping currently supports:
   - scalar worksheet values,
   - `xltypeMulti` array payloads in admitted lanes,
   - admitted reference results returned as `xltypeSRef` or `xltypeRef` when the shim can preserve that identity.
6. Registration shaping policy for generated U rows:
   - worksheet-callable `type_text` is capped to the current Excel baseline limit (`len <= 255`),
   - high-arity UI-only `arg_names` metadata is omitted when it would exceed Excel's practical dialog limit.

For `Q` surface rows:
1. numeric unary, binary, and nullary calls are routed by `function_id` through core dispatch entrypoints.

## 5. Error Mapping (Seed)
1. OxFunc worksheet errors map to XLL `xlerr*` codes (`#DIV/0!`, `#VALUE!`, `#N/A`, etc.).
2. Coercion/ref-resolution/unsupported shim cases default to `#VALUE!` unless a specific worksheet error is already declared.

## 6. Bounded Lanes
1. Array payload semantics remain shape-bounded in core for several functions; U bridge preserves this boundary.
2. Asynchronous/RTD callback model.
3. Production-hardening beyond current `xlbitDLLFree` + `xlAutoFree12` ownership discipline.
4. Registration-flag mapping (`!`, `$`, `#`) is intentionally not profile-derived yet; W11 uses runtime-only experimental alias registrations for evidence collection.

## 7. Verification-Seam Limitation Disclosure
1. This shim contract is part of the XLL verification seam, not the semantics-owning function layer.
2. Known seam limitations are tracked centrally in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.
3. Any function or packet using XLL evidence must repeat the relevant limitation notes in its own verification record when those limits qualify the claim.
