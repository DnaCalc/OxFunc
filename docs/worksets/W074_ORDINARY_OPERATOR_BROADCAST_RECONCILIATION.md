# WORKSET - Ordinary Operator Broadcast Reconciliation (W074)

## 1. Purpose
Own the broadened ordinary-operator execution pass opened after local Excel
comparison showed that the current OxFunc operator value surface still
understates Excel's array broadcast semantics.

This packet exists to turn the post-`W073` empirical finding into one bounded
owner for:
1. arithmetic broadcast completion,
2. compare/concat broadcast widening,
3. refreshed operator-family empirical evidence,
4. downstream OxFml seam reconciliation and fallback-removal proof.

## 2. Why This Packet Exists
`W073` repaired the immediate scalar-only arithmetic lane that OxFml reported,
but local Excel comparison on the current baseline showed that ordinary
operators go further than same-shape array lift:
1. row-by-column operator inputs spill as broadcast/outer-product grids,
2. singleton dimensions broadcast across the larger opposing dimension,
3. coordinates that neither operand can supply surface `#N/A` rather than
   collapsing the whole operator result.

That means the current OxFunc operator packet still has a known semantic gap
even after the first `W073` widening.

## 3. Provenance
Primary intake and evidence inputs:
1. `docs/bugs/reports/BUGREP-FUNC-002_local_excel_probe_operator_broadcast_semantics.md`
2. `docs/bugs/streams/BUG-FUNC-002_ordinary_operator_broadcast_semantics_gap.md`
3. `docs/bugs/streams/BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md`
4. `docs/function-lane/W45_EXECUTION_RECORD.md`
5. `docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_OPERATOR_COMPARE_CONCAT_FAMILY_CONTRACT_PRELIM.md`
7. `docs/worksets/W073_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`

## 4. Scope
In scope:
1. characterize the current Excel broadcast rule for ordinary arithmetic and
   compare/concat operators using the existing `W45` probe plumbing,
2. widen OxFunc ordinary binary operator value surfaces to that broadcast rule,
3. keep unary/postfix arithmetic and structural reference operators under
   explicit revalidation so the packet stays operator-family coherent,
4. refresh operator-family contracts, execution records, bug streams, and
   current-gap wording to match the broadened evidence,
5. file any required OxFunc -> OxFml seam handoff so downstream temporary
   fallbacks can be removed honestly.

Out of scope:
1. parser/token/binder ownership in OxFml,
2. `@` / implicit intersection,
3. unrelated callable/helper or reference-resolution seam debates,
4. claiming full ordinary-operator closure without exercised native probe
   evidence on the broadened array lanes.

## 5. Initial Epic Lanes
1. empirical broadcast rule characterization
2. ordinary operator runtime widening
3. operator-family probe refresh and validation
4. downstream seam handoff and fallback-removal proof
5. truth-surface reconciliation

## 6. Deliverables
This packet should produce:
1. a bounded owner for `BUG-FUNC-002`,
2. updated arithmetic and compare/concat runtime surfaces aligned to the
   observed broadcast rule,
3. refreshed `W45` manifests/runners/results for broadcast-sensitive operator
   lanes,
4. updated workset/bug/current-gap/closure surfaces that no longer overstate
   ordinary operator semantics,
5. an OxFunc -> OxFml handoff covering the reconciled operator broadcast lane if
   the seam contract changed materially.

## 7. Closure Condition
`W074` is complete for declared scope only when:
1. the current ordinary arithmetic and compare/concat operator surfaces follow
   the observed broadcast rule or are honestly reclassified,
2. focused Rust validation and refreshed native `W45` probe runs are recorded,
3. `BUG-FUNC-002` has root cause, similar-risk scan, validation, and linked
   artifact updates recorded,
4. any required OxFunc -> OxFml handoff is filed and registered,
5. `W45`, `W051`, bug streams, and feature/workset index surfaces no longer
   overclaim the ordinary-operator state.
