# W16 Batch 1 - Unary Numeric Family Notes

Status: `active`
Owner lane: `OxFunc`
Relationship: first executable family inside `W016`

## 1. Family Members
1. `ACOS`
2. `ACOSH`
3. `ATAN`
4. `COS`
5. `COSH`
6. `DEGREES`
7. `EXP`
8. `RADIANS`
9. `SINH`
10. `TAN`
11. `TANH`

## 2. Why This Family Starts First
1. all eleven functions are deterministic, nonvolatile, host-independent, and values-only at the adapter seam,
2. all eleven fit the same unary numeric scalar-or-array-elementwise coercion/lift pattern,
3. the pure `num -> num` subset is eligible for ordinary `U` exports and generated `Q` unary-number exports,
4. the domain-sensitive subset (`ACOS`, `ACOSH`) stays in the same helper path with explicit `#NUM!` mapping,
5. this family gives breadth without opening new caller-context, host-query, or reference-return seams.

## 3. Current Local Artifact Shape
Runtime artifacts:
1. `crates/oxfunc_core/src/functions/unary_numeric.rs`
2. `crates/oxfunc_core/src/functions/acos.rs`
3. `crates/oxfunc_core/src/functions/acosh.rs`
4. `crates/oxfunc_core/src/functions/atan.rs`
5. `crates/oxfunc_core/src/functions/cos.rs`
6. `crates/oxfunc_core/src/functions/cosh.rs`
7. `crates/oxfunc_core/src/functions/degrees.rs`
8. `crates/oxfunc_core/src/functions/exp_fn.rs`
9. `crates/oxfunc_core/src/functions/radians.rs`
10. `crates/oxfunc_core/src/functions/sinh.rs`
11. `crates/oxfunc_core/src/functions/tan.rs`
12. `crates/oxfunc_core/src/functions/tanh.rs`
13. dispatch/export integration in `surface_dispatch.rs` and `xll_export_specs.rs`

Formal artifacts:
1. `formal/lean/OxFunc/Functions/Acos.lean`
2. `formal/lean/OxFunc/Functions/Acosh.lean`
3. `formal/lean/OxFunc/Functions/Atan.lean`
4. `formal/lean/OxFunc/Functions/Cos.lean`
5. `formal/lean/OxFunc/Functions/Cosh.lean`
6. `formal/lean/OxFunc/Functions/Degrees.lean`
7. `formal/lean/OxFunc/Functions/Exp.lean`
8. `formal/lean/OxFunc/Functions/Radians.lean`
9. `formal/lean/OxFunc/Functions/Sinh.lean`
10. `formal/lean/OxFunc/Functions/Tan.lean`
11. `formal/lean/OxFunc/Functions/Tanh.lean`

## 4. Current Verified Floor
1. the helper seam admits numeric text and array elementwise lifting with per-element `#VALUE!` mapping for bad text,
2. each function kernel is wired through surface dispatch,
3. each function is present in generated XLL export specs,
4. `cargo test`, `cargo check`, and `lake build` are green on the current local tree.

## 5. What Is Still Open
1. packet-level empirical replay for these eight functions has not yet been added,
2. per-function contract docs for the family are not yet expanded beyond the batch note and code metadata,
3. no `function-phase-complete` claim is made yet for any member of this batch.
