# HO-FN-007 - Multi-area value materialization Style A

## 1. Direction
- **From**: `OxFunc`
- **To**: `OxFml`
- **Filed date**: `2026-04-07`
- **Source workset**: `W076`
- **Related inbound handoff**: `HANDOFF-OXFUNC-002`

## 2. Purpose
Confirm the local OxFunc-side implementation of the requested Style A seam:
same-sheet `ReferenceKind::MultiArea` now materializes through OxFunc-owned
resolver-driven combination semantics in current value-required lanes.

## 3. Packet Summary
OxFunc now owns the admitted same-sheet `MultiArea` value-materialization rule
over the existing `ReferenceResolver` interface:
1. same-sheet `MultiArea` is split through `multi_area_targets()` and recursive
   same-sheet part collection,
2. each member target is resolved as an ordinary single-target reference through
   the existing resolver,
3. resolved member payloads are combined in OxFunc as one row-major row vector
   in member order,
4. mixed-sheet multi-area remains an explicit rejected lane,
5. reference-visible consumers still preserve `MultiArea` identity until a
   value-required lane actually materializes it.

## 4. Concrete Local Outcomes
1. value-required OxFunc paths now materialize same-sheet `MultiArea` locally
   instead of relying on an OxFml-local helper,
2. `SUM((Alpha!A1:A2,Alpha!B2))` now works through OxFunc-owned multi-area
   materialization on the local runtime side,
3. error cells inside a materialized multi-area payload remain preserved as
   cells and continue to propagate through aggregate semantics,
4. `XMATCH`/lookup-vector preparation now sees the combined row-vector payload
   for same-sheet `MultiArea`,
5. mixed-sheet multi-area materialization still surfaces
   `ProviderFailure("mixed_sheet_multi_area")` locally.

## 5. Validation Run
1. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib resolver -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib adapters -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib sum -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib xmatch_surface -- --nocapture`
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_reference_family -- --nocapture`
7. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib index -- --nocapture`
8. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
9. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`
10. `powershell -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`

All listed validation passed on the current working tree.

## 6. OxFml Ask
1. remove the remaining OxFml-local same-sheet multi-area value-materialization
   helper for the admitted slice,
2. consume the OxFunc-owned Style A materialization path as the current shared
   seam floor,
3. report any remaining OxFml-side mismatch separately from the removed local
   aggregation helper.
