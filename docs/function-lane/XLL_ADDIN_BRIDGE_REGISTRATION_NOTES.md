# XLL Add-in Bridge Registration Notes

Status: `provisional`
Workset: `W9`

## 1. Purpose
Record seed registration posture for U-style vs Q-style surfaces in the OxFunc XLL bridge.

## 2. Registration Matrix Source
1. Registration matrix is generated from core function profiles:
   - source: `crates/oxfunc_core/src/xll_export_specs.rs`
   - generated snapshot: `tools/xll-addin/oxfunc_xll/export_specs.csv`
2. Current generated snapshot includes all catalog functions with profile-derived U variants (`u_arity_N`) and profile-admitted Q variants.

## 3. Registration Call Path
1. Self-registration in `xlAutoOpen`.
2. `xlfRegister` invocation through direct Rust callback binding to Excel `MdCallBack12`.
3. Category label: `OxFunc Bridge`.
4. Source-of-truth export rows are declared in `crates/oxfunc_core/src/xll_export_specs.rs`.
5. Export wrappers and registration rows are generated during build from that core source.
6. CSV snapshot (`tools/xll-addin/oxfunc_xll/export_specs.csv`) is generated from core for audit/review.

## 4. Current Policy Decisions
1. Export all catalog functions to XLL via profile-derived rules.
2. Derive U-vs-Q variants from `FunctionMeta` profile fields, not per-function bridge edits.
3. Keep core function semantics in `oxfunc_core`; XLL layer remains transport + registration + type conversion only.
4. Keep export rows declarative in core; wrappers/rows and CSV snapshots are generated mechanically.
5. Ordinary `volatile_full` U exports now receive `!` in generated registration text from core metadata (for example `ox_NOW`, `ox_TODAY`, `ox_RAND`).

## 5. Follow-on Decisions
1. Confirm whether very-high-arity generated U signatures (for example `SUM` with `u_arity_255`) should remain as-is or be bounded by an explicit export-profile cap; current baseline replay shows `ox_SUM(...)` resolving to `#NAME?` despite generated export presence.
2. Expand core array payload modeling to improve U-path behavior for lookup/array-return families.
3. Add additional bridge conformance scenarios covering non-scalar return lanes under generated exports.
4. Keep broader volatile/thread-safe/macro-type registration-flag mapping beyond ordinary `volatile_full` exports deferred from signature/export generation until W11 evidence closure (`docs/function-lane/XLL_REGISTRATION_FLAG_EVIDENCE_PLAN.md`).
5. Use runtime-only experimental aliases (for example `ox_NOW_F_BASE` vs `ox_NOW_F_VOL`) to gather evidence without altering profile-derived generation.
