# HO-FN-006 - Multi-area reference seam correction

## 1. Direction
- **From**: `OxFunc`
- **To**: `OxFml`
- **Filed date**: `2026-04-07`
- **Source workset**: `W075`
- **Related bugs**: `BUG-FUNC-003`

## 2. Purpose
Confirm the local OxFunc correction for first-class same-sheet multi-area
reference transport and identify the remaining cross-repo lane as landed-ref
promotion plus downstream acknowledgment.

## 3. Packet Summary
OxFunc now treats same-sheet union composition as a first-class multi-area seam
shape rather than an `Area` plus raw parenthesized target convention:
1. `OP_UNION_REF` now returns `ReferenceKind::MultiArea`,
2. existing multi-area operands are flattened rather than nested,
3. `AREAS` and `INDEX(..., area_num)` now consume first-class `MultiArea`,
4. the old non-`MultiArea` parenthesized wrapper is now rejected locally rather
   than decoded as a compatibility path.

## 4. Concrete Local Outcomes
1. `OP_UNION_REF(A1:A2,G1:G2)` now yields
   `ReferenceLike { kind: MultiArea, target: "(A1:A2,G1:G2)" }`
2. `INDEX((A1:A2,G1:G2),2,1,2)` still selects `G2`
3. mixed-sheet multi-area reference materialization remains an explicit
   unsupported-source lane in current local `INDEX` handling
4. implicit intersection now treats `MultiArea` explicitly rather than through
   an accidental non-exhaustive match
5. the old non-`MultiArea` parenthesized wrapper carrier is no longer decoded by
   local named consumers; it is rejected instead

## 5. Validation Run
1. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib operator_reference_family -- --nocapture`
3. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib index -- --nocapture`
4. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib reference_metadata_family -- --nocapture`
5. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib surface_dispatch -- --nocapture`
6. `powershell -ExecutionPolicy Bypass -File tools/w40-probe/run-w40-reference-metadata-baseline.ps1`
7. `powershell -ExecutionPolicy Bypass -File tools/w45-probe/run-w45-wavec-operator-reference-baseline.ps1`

## 6. OxFml Ask
1. confirm the corrected local shape is sufficient for OxFml to remove its
   remaining same-sheet multi-area bridge pressure,
2. treat `ReferenceKind::MultiArea` as the shared seam carrier for same-sheet
   multi-area references,
3. report any remaining OxFml-side mismatch separately from the removed legacy
   wrapper convention.
