# XLL Add-in Bridge Shim Contract (Prelim)

Status: `provisional`
Workset: `W9`

## 1. Purpose
Define the seed caller/shim contract for mapping XLL entrypoint arguments into OxFunc function calls without coupling core semantics to XLL ABI details.

## 2. Bridge Components
1. XLL transport/registration bridge:
   - `tools/xll-addin/oxfunc_xll/native/registration_bridge.cpp`
2. Rust shim + core invocation:
   - `tools/xll-addin/oxfunc_xll/src/lib.rs`
3. Core semantics owner:
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
For `ox_ABS` (`U` surface):
1. Incoming `LPXLOPER12` categories handled:
   - `xltypeNum` -> number,
   - `xltypeStr` -> UTF-16 text (pascal length-prefixed),
   - `xltypeBool` -> logical,
   - `xltypeErr` -> worksheet error code,
   - `xltypeMissing` -> missing arg,
   - `xltypeNil` -> empty cell.
2. Reference lanes:
   - `xltypeRef` / `xltypeSRef` are dereferenced through `xlCoerce` before shim translation.
3. `xltypeMulti` array lanes:
   - shape-preserving elementwise mapping through scalar shim (`eval_abs_scalar_value`) per element.
   - result returned as `xltypeMulti` with Excel-owned lifetime (`xlbitDLLFree` + `xlAutoFree12`).
4. Shim invokes OxFunc:
   - scalar path through `eval_abs_scalar_value` in `oxfunc_core`.

For `ox_ABS_Q`:
1. Numeric-only path maps directly to OxFunc kernel (`abs_kernel`) as control surface.

For `ox_PI`:
1. Nullary numeric constant path maps to OxFunc PI evaluator.

## 5. Error Mapping (Seed)
1. OxFunc worksheet errors map to XLL `xlerr*` codes (`#DIV/0!`, `#VALUE!`, `#N/A`, etc.).
2. Coercion/ref-resolution/unsupported shim cases default to `#VALUE!` unless a specific worksheet error is already declared.

## 6. Non-Goals in Seed
1. Full generalization of array-lift policy beyond current unary ABS shape-preserving mapping.
2. Asynchronous/RTD callback model.
3. Production-hardening beyond current scalar static-return + multi `xlAutoFree12` ownership split.
