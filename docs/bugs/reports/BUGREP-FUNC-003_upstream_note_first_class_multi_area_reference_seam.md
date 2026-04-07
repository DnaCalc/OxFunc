# BUGREP-FUNC-003: OxFml upstream note on first-class same-sheet multi-area references

## Summary
- **Report id**: `BUGREP-FUNC-003`
- **Filed**: 2026-04-07
- **Status**: triaged
- **Canonical bug**: `BUG-FUNC-003`

## Source
- **Source channel**: upstream note
- **Source artifact**: `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
- **Reported against ref**: `9e9c573a46d97e248a0373938fb53dcac916fac2`
- **Reported against kind**: `commit`

## Reported Problem
OxFml reported that the shared reference seam should treat same-sheet multi-area
references as first-class `ReferenceKind::MultiArea` values end-to-end. The
current OxFunc branch already exposes the intended type shape in
`crates/oxfunc_core/src/value.rs`, but `OP_UNION_REF` still returned
`ReferenceKind::Area` with a parenthesized target string and some consumers
still reparsed raw target strings instead of consulting the `MultiArea`
helpers first.

## Intake Notes
1. The note is specific and actionable enough to treat as a real local defect,
   not only as doctrine commentary.
2. The concrete OxFunc-sensitive lanes named by OxFml are:
   - `OP_UNION_REF`
   - `AREAS`
   - `INDEX(..., area_num)`
   - resolver normalization / capability checks
   - any adapter mapping that pattern-matches `ReferenceKind`
3. The bug was opened as `BUG-FUNC-003` and bounded owner `W075`.

