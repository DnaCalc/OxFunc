# XLL Add-in Bridge Execution Record

Status: `complete-provisional`
Workset: `W9`
Evidence ID: `W9-XLL-BL-20260308`

## 1. Purpose
Track reproducible build/registration/baseline-validation evidence for the Rust-based `OxFunc64.xll` bridge.

## 2. Executed Baseline Scope
Execution date:
1. `2026-03-08`
2. `2026-03-11` (bridge follow-up rerun after reference-resolution and reference-return shim expansion)
3. `2026-03-11` (registration callability follow-up after capped U-arity generation and high-arity `arg_names` omission)

Environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Locale profile: `en-US`

Inputs:
1. Manifest:
   - `docs/function-lane/XLL_ADDIN_BRIDGE_VALIDATION_SCENARIO_MANIFEST_SEED.csv`
2. Bridge build:
   - `tools/xll-addin/build-oxfunc-xll.ps1`
   - `tools/xll-addin/oxfunc_xll/*`
3. Baseline runner:
   - `tools/xll-addin/run-oxfunc-xll-bridge-baseline.ps1`

Outputs:
1. XLL artifact:
   - `tools/xll-addin/bin/OxFunc64.xll`
2. Baseline result rows:
   - `.tmp/oxfunc-xll-bridge-results.csv`
   - `.tmp/xll-housekeeping/oxfunc-xll-bridge-results.csv`
   - `.tmp/xll-arity-fix/oxfunc-xll-bridge-results.csv`
   - `.tmp/textjoin-closeout/oxfunc-xll-bridge-results.csv`
   - `.tmp/exact-clean-closeout/oxfunc-xll-bridge-results.csv`
3. Dedicated nil-propagation follow-up:
   - `docs/function-lane/XLL_NIL_PROPAGATION_EXECUTION_RECORD.md`
   - `docs/function-lane/XLL_NIL_PROPAGATION_SCENARIO_MANIFEST_SEED.csv`
   - `.tmp/xll-nil-probe-results.csv`

## 3. Gate Tracking
### G1 - Bridge Scaffold Closure
1. Status: `closed`.
2. Evidence:
   - `tools/xll-addin/oxfunc_xll/Cargo.toml`
   - `tools/xll-addin/oxfunc_xll/build.rs`
   - `crates/oxfunc_core/src/xll_export_specs.rs`
   - `tools/xll-addin/oxfunc_xll/export_specs.csv`
   - `tools/xll-addin/build-oxfunc-xll.ps1`
   - `tools/xll-addin/sync-export-specs.ps1`
   - built artifact `tools/xll-addin/bin/OxFunc64.xll`

### G2 - Shim Contract Closure
1. Status: `closed`.
2. Evidence:
   - `docs/function-lane/XLL_ADDIN_BRIDGE_SHIM_CONTRACT_PRELIM.md`
   - `tools/xll-addin/oxfunc_xll/src/lib.rs`
   - `crates/oxfunc_core/src/xll_export_specs.rs`
   - `tools/xll-addin/oxfunc_xll/export_specs.csv`
   - `crates/oxfunc_core/src/functions/surface_dispatch.rs`

### G3 - Registration Closure
1. Status: `closed`.
2. Evidence:
   - `docs/function-lane/XLL_ADDIN_BRIDGE_REGISTRATION_NOTES.md`
   - runtime `RegisterXLL` success in baseline runner.
   - callable generated exports from profile-derived table (`tools/xll-addin/oxfunc_xll/export_specs.csv`).

### G4 - Differential Workbook Closure
1. Status: `closed-provisional`.
2. Evidence:
   - `docs/function-lane/XLL_ADDIN_BRIDGE_VALIDATION_SCENARIO_MANIFEST_SEED.csv`
   - `.tmp/oxfunc-xll-bridge-results.csv`
   - `.tmp/xll-housekeeping/oxfunc-xll-bridge-results.csv`
   - `.tmp/w12-final-xll/oxfunc-xll-bridge-results.csv`
3. Outcomes:
   - rows: `28`
   - relation_status `matched`: `28`
   - relation_status `mismatched`: `0`
   - scalar ABS/PI rows remain green, spill-sensitive row `W9-XLL-007` remains parity-closed, aggregate provenance rows `W9-XLL-008..010` are callable and parity-closed through the bridge, reference-return address parity remains green in `W9-XLL-011`, `TEXTJOIN` non-error flattening parity is green in `W9-XLL-012..013`, `EXACT` textification/Unicode rows are green in `W9-XLL-014..015`, `CLEAN` control-removal rows are now both parity-closed in `W9-XLL-016..017`, and the expanded W12-facing rows for `AVERAGE`, `COUNT`, `COUNTA`, `IFERROR`, `ROUND`, `AND`, `DATE`, `OFFSET`, and `HSTACK` are green in `W9-XLL-018..028`.

### G5 - Separation Closure
1. Status: `closed`.
2. Evidence:
   - bridge dependencies isolated under `tools/xll-addin/oxfunc_xll`.
   - `crates/oxfunc_core` has no XLL SDK transport dependency.

## 4. Key Findings
1. Seed scalar parity remains strong for ABS and PI paths (`W9-XLL-001..006`).
2. U-style bridge surface can dereference simple references (`ox_ABS(A1)` parity observed).
3. U-style bridge continues to support shape-preserving `xltypeMulti` elementwise mapping for ABS (`W9-XLL-007` parity-closed).
4. The current shim can now carry admitted reference results back to Excel strongly enough for `CELL("address", ox_XLOOKUP(...))` parity in `W9-XLL-011`.
5. Export wrappers and registration rows are generated from core `FunctionMeta` profile rules through `xll_export_specs`, reducing hand-authored bridge code.
6. Very-high-arity worksheet exports need explicit registration shaping in the generated XLL surface:
   - keeping `type_text.len() <= 255` is necessary for worksheet callability in the current baseline,
   - omitting oversized `arg_names` strings is sufficient for the current bridge because they are UI-only metadata,
   - after applying that policy, `ox_SUM(...)` aggregate rows are callable and parity-closed.
7. Fixed-width U exports also need trailing omitted-argument trimming before core dispatch:
   - after trimming trailing `xltypeMissing` inputs down to the effective call arity, variadic XLL functions such as `ox_TEXTJOIN(...)` align with built-in non-error flattening semantics instead of materializing spurious trailing delimiters.
8. Rebuilding the XLL after the W12 closeout removed two bridge-only mismatches that had looked like semantic gaps from the outside:
   - `DATE(1900,1,0)` now matches built-in Excel through `ox_DATE(...)`,
   - and the extra-C1 `CLEAN` row now also matches built-in Excel through `ox_CLEAN(...)`.
9. Raw `xltypeNil` behavior is now better characterized at the bridge boundary:
   - direct scalar raw `xltypeNil` returns publish as numeric zero,
   - outer functions/operators observe the scalarized result as number/zero rather than `empty_cell`,
   - raw `xltypeNil` array elements can still survive inside returned arrays until scalarization/publication.

## 5. Follow-on Bounded Lanes
1. registration-flag mapping (`!`, `$`, `#`) stays deferred from profile-derived generation until W11 evidence closure.
2. W11 evidence lane uses runtime-only experimental aliases so profile-to-signature generation remains unchanged during probe collection.

## 6. XLL Verification-Seam Limitations
1. This record is a seam record, not a function-semantic closure record.
2. Known seam limitations are tracked in `docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md`.
3. Relevant current limits for W9:
   - registration-flag behavior is not yet part of normal profile-derived export generation,
   - reference-return and non-scalar payload parity remain bounded outside the currently green lookup-family scope,
   - concurrency/thread-safety evidence is not demonstrated by this bridge baseline.
