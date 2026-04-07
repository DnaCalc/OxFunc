# WORKSET - Operator Value Surface And Array-Lift Expansion (W073)

## 1. Purpose
Own the OxFunc-side follow-up for the confirmed ordinary-operator seam gap where
array-involved binary arithmetic cannot yet travel through the current OxFml ->
OxFunc prepared-call/value surface without downstream fallback.

This workset exists to turn the inbound OxFml operator handoff into a bounded
OxFunc execution owner with explicit scope, adjacent-risk review, and honest
current-gap reporting.

## 2. Why This Packet Exists
Current OxFunc already exposes ordinary operator rows and already supports:
1. unary arithmetic and postfix percent through an array-aware unary numeric
   surface,
2. scalar binary arithmetic through the current binary numeric surface,
3. scalar compare/concat and structural reference operators through separate
   surfaces.

What the current OxFunc surface does not yet do honestly is:
1. carry array-involved binary arithmetic through the same admitted value
   surface,
2. make the current ordinary-operator family boundaries explicit enough to stop
   downstream over-read of the admitted slice,
3. reconcile the current-gap and closure surfaces after the OxFml handoff made
   this limitation concrete.

## 3. Provenance
Primary intake and evidence inputs:
1. `../OxFml/docs/handoffs/HANDOFF_OXFUNC_001_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. `docs/bugs/reports/BUGREP-FUNC-001_operator_value_surface_and_array_lift_handoff.md`
4. `docs/bugs/streams/BUG-FUNC-001_binary_operator_array_lift_value_surface_gap.md`
5. `docs/function-lane/FUNCTION_SLICE_OPERATOR_ARITHMETIC_FAMILY_CONTRACT_PRELIM.md`
6. `docs/function-lane/W45_EXECUTION_RECORD.md`
7. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`

## 4. Scope
In scope:
1. define the honest current OxFunc-side owner for `BUG-FUNC-001`,
2. widen or otherwise correct the OxFunc-side value surface for admitted
   array-involved ordinary binary arithmetic,
3. make the affected operator-family boundaries explicit in local contracts and
   closure records,
4. re-check adjacent operator families so unary arithmetic, postfix percent,
   compare/concat, and reference operators are classified honestly rather than
   implied by stale packet closure wording,
5. remove the need for the current downstream compatibility fallback when the
   OxFunc-side surface is ready,
6. update any required bug, handoff, and current-gap artifacts as the work
   advances.

Out of scope:
1. changing OxFml parser/token/binder ownership,
2. treating broad operator-family admission as complete without exercised
   evidence,
3. closing unrelated compare/concat or reference-operator topics that do not
   prove to share this failure mode,
4. claiming replay or validation closure before the code/spec/test path lands.

## 5. Initial Epic Lanes
1. current-surface intake and exact-gap characterization
2. binary operator value-surface widening
3. adjacent-family classification and similar-risk follow-up
4. downstream seam reconciliation and fallback-removal proof
5. current-gap and closure-surface reconciliation

## 6. Deliverables
This packet should produce:
1. an honest bounded owner for `BUG-FUNC-001`,
2. the required OxFunc-side code and contract changes for the admitted binary
   operator array-lift lane,
3. updated local current-gap and closure surfaces once the gap is repaired or
   otherwise reclassified,
4. any required downstream handoff or acknowledgement updates tied to the seam
   outcome,
5. validation evidence strong enough to remove the downstream temporary
   compatibility fallback honestly.

## 7. Closure Condition
`W073` is complete for declared scope only when:
1. the admitted binary operator array-lift lane is implemented or honestly
   reclassified,
2. local validation for the changed operator surfaces is recorded,
3. `BUG-FUNC-001` has root cause, similar-risk scan, validation, and linked
   artifact updates recorded,
4. current-gap and closure surfaces no longer overclaim the ordinary-operator
   state,
5. any required downstream handoff acknowledgement is recorded.

## 8. Current Intake Read
Current local read at intake:
1. the immediate proven bug is the scalar-only binary numeric surface,
2. unary arithmetic and postfix percent already have local elementwise lift and
   are not the same proven defect,
3. compare/concat remain scalar-only and are adjacent risk rather than current
   proof of the same bug,
4. reference operators remain a different `RefsVisibleInAdapter` family,
5. `W45` and `W051` required qualification once the OxFml handoff made this
   current-gap explicit.
