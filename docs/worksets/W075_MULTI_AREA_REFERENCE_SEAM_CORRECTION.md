# WORKSET - Multi-Area Reference Seam Correction (W075)

## 1. Purpose
Own the bounded OxFunc-side correction for first-class same-sheet multi-area
reference handling after the current OxFml upstream note identified that the
shared seam still collapsed some union/reference paths into an `Area` plus raw
parenthesized target reinterpretation.

## 2. Why This Packet Exists
OxFunc already carried the intended `ReferenceKind::MultiArea` shape in the
value model, but the seam remained inconsistent:
1. `OP_UNION_REF` still returned `ReferenceKind::Area`,
2. `INDEX(..., area_num)` and `AREAS` still started from target-string
   reinterpretation rather than the first-class helper APIs,
3. some `ReferenceKind` pattern matches still had no explicit `MultiArea`
   reading.

## 3. Provenance
1. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
2. `docs/bugs/reports/BUGREP-FUNC-003_upstream_note_first_class_multi_area_reference_seam.md`
3. `docs/bugs/streams/BUG-FUNC-003_multi_area_reference_seam_collapses_to_area_string.md`
4. `docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_INDEX_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_CONTRACT_PRELIM.md`

## 4. Scope
In scope:
1. make `OP_UNION_REF` return first-class `ReferenceKind::MultiArea`,
2. make `INDEX(..., area_num)` and `AREAS` consume `multi_area_targets()` rather than the old wrapper convention,
3. keep same-sheet multi-area distinct from mixed-sheet unsupported-source cases
   and from `ThreeD`,
4. update local contracts / current-gap surfaces so the shared seam is reported
   honestly,
5. file the OxFunc -> OxFml reply packet once the local floor is validated.

Out of scope:
1. redefining 3D sheet spans,
2. broad new reference-materialization semantics beyond the named consumers,
3. parser/binder ownership in OxFml,
4. claiming landed-ref closure before the work actually lands on a committed ref.

## 5. Initial Epic Lanes
1. inbound note intake and current-gap registration
2. multi-area runtime correction
3. reference-sensitive consumer alignment
4. truth-surface reconciliation
5. downstream seam handoff

## 6. Closure Condition
`W075` is complete for declared scope only when:
1. local runtime returns `ReferenceKind::MultiArea` for union formation,
2. named consumers (`AREAS`, `INDEX`) consume the first-class multi-area shape and reject the old non-`MultiArea` wrapper carrier,
3. focused local validation is recorded,
4. the downstream OxFunc -> OxFml handoff is filed,
5. W40/W45/W51 and the bug/workset surfaces no longer overclaim the old
   area-string convention.
