# Function Slice Contract (Preliminary) - Operator Arithmetic Family

Status: `provisional`
Workset: `W45`
Primary Rows: `OP_UNARY_PLUS`, `OP_NEGATE`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_POWER`, `OP_PERCENT`

## 1. Scope
1. admit the first executable `W45` slice for non-`@` operator semantics,
2. cover the arithmetic/unary/postfix numeric family on the current reference baseline,
3. keep reference-composition and comparison/concatenation operators outside this contract.

## 2. Admitted Current-Baseline Slice
1. `OP_UNARY_PLUS`
   - values-only numeric coercion,
   - elementwise array lift on the current OxFunc-local substrate.
2. `OP_NEGATE`
   - values-only numeric coercion,
   - elementwise array lift on the current OxFunc-local substrate.
3. `OP_PERCENT`
   - postfix numeric scaling by `1/100`,
   - elementwise array lift on the admitted local slice.
4. `OP_SUBTRACT`
   - binary numeric coercion over the values-only path.
5. `OP_MULTIPLY`
   - binary numeric coercion over the values-only path.
6. `OP_DIVIDE`
   - binary numeric coercion,
   - divide-by-zero returns `#DIV/0!`.
7. `OP_POWER`
   - binary numeric coercion,
   - shares the admitted `POWER` kernel domain behavior for the current baseline, including `0^-n -> #DIV/0!` and real-domain `NaN` cases as `#NUM!`.

## 3. Metadata Shape
1. unary rows:
   - `arg_preparation_profile = values_only_pre_adapter`
   - `coercion_lift_profile = unary_numeric_scalar_or_array_elementwise`
   - `kernel_signature_class = num_to_num`
2. binary rows:
   - `arg_preparation_profile = values_only_pre_adapter`
   - `coercion_lift_profile = custom`
   - `kernel_signature_class = nums_to_num`
3. all rows:
   - `deterministic`
   - `nonvolatile`
   - `host_interaction = none`
   - `thread_safety = safe_pure`

## 4. Explicitly Out Of Slice
1. comparison operators,
2. concatenation operator,
3. reference composition operators,
4. `@`,
5. full operator parser ownership and precedence lock with OxFml,
6. native Excel evidence beyond the seeded Wave-A packet.

## 5. Evidence Basis
1. Rust runtime/tests: `crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
2. dispatch/export: `crates/oxfunc_core/src/functions/surface_dispatch.rs`; `crates/oxfunc_core/src/xll_export_specs.rs`
3. Lean metadata/kernel alignment: `formal/lean/OxFunc/Functions/OperatorArithmeticFamily.lean`
4. native packet seed: `docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv`
5. native runtime harness: `tools/w45-probe/run-w45-wavea-operator-arithmetic-baseline.ps1`
6. packet execution record: `docs/function-lane/W45_EXECUTION_RECORD.md`
