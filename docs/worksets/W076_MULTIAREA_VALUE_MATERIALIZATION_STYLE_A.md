# WORKSET - Multi-Area Value Materialization Style A (W076)

## 1. Purpose
Own the bounded OxFunc-side follow-on requested by
`HANDOFF-OXFUNC-002`: same-sheet first-class `ReferenceKind::MultiArea`
should materialize into value payloads through OxFunc-owned resolver-driven
combination semantics rather than an OxFml-local aggregation helper.

## 2. Why This Packet Exists
`W075` corrected multi-area reference identity and named reference-visible
consumers, but value-required lanes still depended on downstream local helper
logic in OxFml:
1. OxFunc preserved and returned first-class `MultiArea`,
2. OxFunc reference-visible consumers (`OP_UNION_REF`, `AREAS`, `INDEX`) were
   locally honest,
3. but OxFunc value-preparation paths still dereferenced only one reference
   target at a time and did not own multi-area combination semantics.

## 3. Provenance
1. `../OxFml/docs/handoffs/HANDOFF_OXFUNC_002_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. `docs/worksets/W075_MULTI_AREA_REFERENCE_SEAM_CORRECTION.md`
4. `docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`
5. `docs/handoffs/HO-FN-006_multi_area_reference_seam_correction.md`

## 4. Scope
In scope:
1. add one OxFunc-owned helper that materializes same-sheet `MultiArea` through
   the existing `ReferenceResolver`,
2. preserve member order and combine resolved payloads into one row-major row
   vector for current value-required lanes,
3. route current value-preparation callers through that helper rather than
   requiring OxFml-local aggregation,
4. keep mixed-sheet multi-area and 3D references explicitly distinct from this
   admitted slice,
5. file the OxFunc -> OxFml reply packet once the local floor is validated.

Out of scope:
1. redefining `MultiArea` reference identity itself; `W075` remains the owner of
   that correction,
2. broad new multi-area semantics beyond current value-required lanes,
3. mixed-sheet multi-area value materialization,
4. parser/binder ownership in OxFml,
5. claiming landed-ref closure before the work lands on a committed ref.

## 5. Initial Epic Lanes
1. handoff intake and local owner registration
2. resolver-driven multi-area materialization helper
3. values-only / lookup / aggregate caller alignment
4. focused local validation
5. downstream reply handoff

## 6. Closure Condition
`W076` is complete for declared scope only when:
1. same-sheet `MultiArea` materializes through OxFunc-owned resolver-driven
   combination semantics,
2. current value-required callers no longer need a downstream OxFml-local helper
   for the admitted slice,
3. mixed-sheet multi-area remains an explicit rejection path,
4. focused local validation is recorded,
5. the downstream OxFunc -> OxFml reply handoff is filed.

## 7. Current Reading
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - landed-ref promotion for the local Style A implementation
   - downstream OxFml acknowledgment and helper removal under `HO-FN-007`
