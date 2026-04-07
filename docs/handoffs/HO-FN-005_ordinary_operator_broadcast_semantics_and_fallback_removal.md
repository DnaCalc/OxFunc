# HO-FN-005 - Ordinary operator broadcast semantics and fallback-removal packet

## 1. Direction
- **From**: `OxFunc`
- **To**: `OxFml`
- **Filed date**: `2026-04-07`
- **Source workset**: `W074`
- **Related bugs**: `BUG-FUNC-001`, `BUG-FUNC-002`

## 2. Purpose
Provide the OxFunc-side ordinary-operator broadcast packet needed for OxFml to
remove its temporary operator fallback honestly.

## 3. Packet Summary
OxFunc local runtime and native Excel evidence now agree on the current
ordinary-operator broadcast rule:
1. singleton dimensions broadcast across the opposing dimension,
2. row-vs-column combinations spill as 2-D outer-product grids,
3. coordinates neither operand can supply return `#N/A`,
4. the rule now applies locally across ordinary arithmetic and compare/concat.

## 4. Concrete Excel-Proven Examples
Observed through refreshed `W45` probes:
1. `={1,2}+{1;2}` -> `2|3|3|4`
2. `={1,2}+{1,2,3}` -> `2|4|#N/A`
3. `={"a","b"}&{"x";"y"}` -> `ax|bx|ay|by`
4. `={1,2}={1;2}` -> `TRUE|FALSE|FALSE|TRUE`
5. `={"a","b"}&{"x","y","z"}` -> `ax|by|#N/A`

## 5. OxFunc Local Changes
1. ordinary binary arithmetic now uses a shared broadcast helper rather than
   same-shape-only array pairing,
2. compare/concat now accept array inputs on that same broadcast rule,
3. `W45` probe manifests and runners now record spill-grid text and shape.

## 6. Validation Run
Local OxFunc validation completed on the current working tree:
1. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib binary_numeric -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib op_add -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_arithmetic_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_compare_concat_family -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
7. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavea-operator-arithmetic-baseline.ps1`
8. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-waveb-operator-compare-concat-baseline.ps1`
9. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`

## 7. OxFml Ask
1. confirm OxFml can consume the broadened ordinary-operator value surface
   without the current temporary fallback,
2. remove or narrow that fallback once the OxFunc packet lands on an agreed ref,
3. report any remaining OxFml-side parser/binder/publication limits separately
   rather than keeping them collapsed into the old scalar-only operator story.

## 8. Affected OxFunc Truth Surfaces
1. `docs/worksets/W074_ORDINARY_OPERATOR_BROADCAST_RECONCILIATION.md`
2. `docs/bugs/streams/BUG-FUNC-002_ordinary_operator_broadcast_semantics_gap.md`
3. `docs/function-lane/W45_EXECUTION_RECORD.md`
4. `docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`
