# BUG-FUNC-001: Binary operator array-lift value surface gap

## Summary
- **Bug id**: `BUG-FUNC-001`
- **Opened**: 2026-04-07
- **Status**: handed_off
- **Owner workset**: `W073`

## Source Refs
- **Reported against ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Reproduced on ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`
- **Ref notes**: The inbound handoff did not cite a release tag or introducing commit. OxFunc intake reproduced the cited scalar-only surface limit against the current local `HEAD` by reading the current binary numeric, unary numeric, compare/concat, and operator dispatch code.

## Ownership And Root Cause
- **Ownership class**: shared seam gap
- **Root cause class**: initial_impl_gap
- **Root cause summary**: the original OxFunc binary numeric operator surface only coerced scalar prepared values, while the broader seam and closure wording around ordinary operators allowed downstream consumers to expect a wider array-involved transport lane than OxFunc had actually admitted.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: no
- **Spec vague or missing?**: yes
- **Code once correct and later regressed?**: no
- **Likely introduced in ref**: `unknown`
- **Explanation**: unary arithmetic was implemented with explicit array-aware lifting, but binary arithmetic stopped at a scalar prepared-value surface. The surviving arithmetic-family contract text did not explicitly mark binary array-involved transport as out of slice, and `W45` closure wording overclaimed the current non-`@` operator state. This reads as an initial admission/closure gap rather than a proven later regression.

## Reproduction
1. Route array-involved operator calls through the current OxFunc binary arithmetic surfaces named in the OxFml handoff:
   - `={1,2,3;2,3,4}*-1`
   - `={1,2;3,4}+{10,20;30,40}`
   - `={1,2;6,8}/{1,0;3,2}`
2. Expected behavior:
   array-involved ordinary binary arithmetic should travel through the OxFml -> OxFunc value surface as typed worksheet values rather than failing as transport errors.
3. Actual behavior at intake:
   the original OxFunc binary numeric surface only accepted scalar prepared values and therefore collapsed these array-involved lanes into worksheet `Value` failures at the seam.

## Spec And Contract Relationship
- **Spec references**:
  1. `docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md`
  2. `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`
  3. `docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`
  4. `docs/function-lane/W45_EXECUTION_RECORD.md`
- **Spec state at intake**: vague
- **Notes**: the arithmetic-family contract already distinguished unary array lift from binary custom handling, but it did not explicitly say the admitted binary slice was scalar/scalar only. The stronger overclaim was in the surviving `W45` closure wording.

## Investigation Log
1. 2026-04-07: Reviewed inbound OxFml observations and formal handoff packet `HANDOFF-OXFUNC-001`.
2. 2026-04-07: Confirmed the intake state in `crates/oxfunc_core/src/functions/binary_numeric.rs` only coerced scalar prepared values and always returned scalar `EvalValue::Number`.
3. 2026-04-07: Confirmed `crates/oxfunc_core/src/functions/unary_numeric.rs` already performs elementwise array lift for unary numeric operators.
4. 2026-04-07: Confirmed `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs` remains explicitly scalar-only for array inputs.
5. 2026-04-07: Confirmed `crates/oxfunc_core/src/functions/operator_reference_family.rs` is a separate reference-visible structural family, not the same value-surface bug.
6. 2026-04-07: Widened `crates/oxfunc_core/src/functions/binary_numeric.rs` to support elementwise `array/scalar`, `scalar/array`, and same-shape `array/array` binary numeric lifting with shape-mismatch rejection.
7. 2026-04-07: Reconciled `crates/oxfunc_core/src/functions/op_add.rs` onto the same widened binary numeric surface so `FUNC.OP_ADD` no longer retains a separate scalar-only path.
8. 2026-04-07: Added focused dispatcher and operator-family tests for the OxFml handoff cases and passed the focused local validation floor listed below.
9. 2026-04-07: Folded the remaining downstream seam ask into `HO-FN-005`; local OxFunc execution is now validated and handed off pending landed-ref promotion plus OxFml acknowledgment.

## Similar-Risk Scan
### Adjacent families to check
1. unary arithmetic and postfix percent
2. concat and comparison operators
3. reference operators

### Check method
1. inspected the current Rust surfaces for operator-family lifting and argument preparation behavior
2. compared the current function-lane contract text for admitted vs out-of-slice claims
3. checked the surviving `W45` execution record for closure wording that could overclaim beyond the admitted surface

### Results
1. unary arithmetic and postfix percent already use the array-aware unary numeric surface and are not the same proven gap
2. concat and comparison rows are still scalar-only and should remain under explicit adjacent-risk review, but no separate OxFunc bug stream was opened yet because the current concrete downstream failure is the binary arithmetic lane
3. reference operators use `RefsVisibleInAdapter` and remain a different structural family, not the same array-value transport bug
4. `W45` closure wording and `W051` zero-gap wording required qualification once this known gap became explicit

### Follow-on Openings
1. `W073`
2. `BUG-FUNC-002`
3. `W074`

## Fix Plan
1. complete export/publication refresh for the widened operator surface where required by the local consumer-facing snapshot story
2. complete downstream seam reconciliation so OxFml can remove the temporary array fallback honestly
3. keep adjacent operator families under explicit review so compare/concat and any broader operator-family widening remain mismatch-driven rather than silently implied
4. file additional handoff or follow-on bug streams only if the broader family review finds distinct non-local or non-equivalent issues

## Validation
1. focused local validation on the current working tree passed:
   - `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib binary_numeric -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib op_add -- --nocapture`
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_arithmetic_family -- --nocapture`
2. the focused `op_add` validation now also covers the dispatcher-level path through `surface_dispatch::eval_surface_value_call(...)`.
3. `fixed_in_ref` remains `not yet fixed` because the widened surface is still only in the current working tree and has not landed on a commit/tagged ref yet.

## Linked Reports
1. `BUGREP-FUNC-001`

## Evidence
1. `../OxFml/docs/handoffs/HANDOFF_OXFUNC_001_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. `crates/oxfunc_core/src/functions/binary_numeric.rs`
4. `crates/oxfunc_core/src/functions/op_add.rs`
5. `crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
6. `crates/oxfunc_core/src/functions/unary_numeric.rs`
7. `crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
8. `crates/oxfunc_core/src/functions/operator_reference_family.rs`
9. `crates/oxfunc_core/src/functions/surface_dispatch.rs`
10. `docs/worksets/W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] handoff filed if required
- [x] linked reports updated
