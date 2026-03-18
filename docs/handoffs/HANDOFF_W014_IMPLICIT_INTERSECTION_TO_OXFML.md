# Handoff - W14 Implicit Intersection Operator To OxFml

Status: `filed`
Source lane: `OxFunc`
Source workset: `W14`
Target lane: `OxFml`
Target workset: `TBD`

## 1. Scope And Profile Bounds
Affected domains:
1. formula-language parse/bind retention of explicit `@`
2. FEC/F3E prepared-operand/result vocabulary
3. replay/trace placement for scalarization

Current scope bound:
1. current reference Excel baseline only
2. dual-axis behavior tracking still required:
   - Excel app version/channel
   - workbook Compatibility Version

## 2. Core Message
`@` is not safely reconstructible after provenance erasure.

Why:
1. official-support behavior distinguishes range/reference inputs from array inputs:
   - range/reference selects by caller row or caller column,
   - array payload selects top-left.
2. if OxFml turns both into the same array payload before OxFunc sees them, OxFunc cannot model the documented behavior.

## 3. Proposed Normative Text
Candidate target areas:
1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
2. `docs/spec/fec-f3e/FEC_F3E_REDESIGN_SPEC.md`

Candidate additions:
1. explicit `@` survives parse/bind as an evaluable node or as an equivalent explicitly traced scalarization step.
2. the operand reaching `@` must preserve whether it is:
   - reference-like,
   - spill-anchor/spilled-range reference,
   - already materialized array payload.
3. caller anchor/context must be available at the point where `@` scalarization is decided.
4. scalarization caused by explicit `@` must be trace-distinguishable from ordinary dereference and ordinary spill publication.

## 4. Evidence And Replay Links
Current OxFunc anchors:
1. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md` section 9
2. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv` row `FDEF-018`
3. `docs/function-lane/IMPLICIT_INTERSECTION_OPERATOR_INVESTIGATION.md`
4. `docs/function-lane/W14_IMPLICIT_INTERSECTION_SCENARIO_MANIFEST_SEED.csv`
5. `docs/upstream/NOTES_FOR_OXFML.md` section 14

Current OxFml anchors:
1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md` rule `FML-R-003`
2. pass-2 notes already showing normalization-sensitive `@` cases in the same document

Foundation evidence lineage already referenced by local rows:
1. `ECS-004`
2. `ECS-007`
3. `ECS-EB-038`

## 5. Requested OxFml Decisions
1. Will `@` remain an explicit evaluable node at the OxFunc boundary, or does OxFml want to evaluate it upstream?
2. If upstream, what trace/result carrier records the scalarization event and its provenance?
3. What prepared-operand vocabulary is the minimum acceptable shape for preserving range-vs-array semantics?

## 6. Risk If Deferred
1. OxFunc may accidentally implement a lossy top-left-only behavior that disagrees with caller-relative range semantics.
2. Later retrofit will be expensive because parse, prepared-call, and trace contracts all become part of the fix.
3. Differential testing against Excel will be hard to interpret if `@` disappears upstream without provenance.
