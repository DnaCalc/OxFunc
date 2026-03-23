# WORKSET - Return Surface And Publication Hint Freeze (W48)

## 1. Purpose
Lock the first shared return-surface split for the already-covered OxFunc scope:
1. ordinary value,
2. value plus presentation hint,
3. typed host/provider outcome projection.

## 2. Provenance
Opened after:
1. `NOW`, `TODAY`, and `HYPERLINK` now expose `ValueWithPresentation` on the OxFunc side,
2. `RTD` and other covered host/provider seams already project typed outcomes into worksheet-visible results,
3. the latest OxFml note identified the returned surface split as the next lock lane after the snapshot/provider freeze direction converged,
4. the final OxFml update for this exchange accepted the current returned-value split as the first freeze candidate.

Relevant context:
1. `docs/upstream/NOTES_FOR_OXFML.md`
2. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
3. `docs/function-lane/VALUE_UNIVERSE_PRELIM_SPEC.md`
4. `docs/function-lane/FUNCTION_SLICE_NOW_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_TODAY_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_HYPERLINK_IMAGE_VALUE_MODEL_PRELIM.md`
7. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`

## 3. Scope
This packet owns:
1. the shared interpretation of ordinary `EvalValue`,
2. the shared interpretation of `ExtendedValue::ValueWithPresentation`,
3. the current typed host/provider outcome projection family for the already-covered scope,
4. the split between value semantics and host-side publication/application of presentation hints.

## 4. Out Of Scope
1. final callable publication policy,
2. generalized future provider outcome families beyond current covered seams,
3. any requirement that OxFunc itself apply presentation hints.

Clarification:
1. `IMAGE` rich-value return/publication work remains in the current overall program scope,
2. but `W048` only owns the shared return-surface freeze that `IMAGE` should align to,
3. not the full packet-specific `IMAGE` closure.

## 5. Expected Deliverables
1. one shared return-surface contract note,
2. one mapping from covered functions to return-surface class,
3. one narrowed outbound note section for the next OxFml sync,
4. any required local value-universe naming cleanup,
5. one explicit statement of the accepted first freeze candidate for the returned split.

## 6. Initial Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `partial`
5. open_lanes:
   - current frozen return split is pinned locally but not yet acknowledged as the shared first bounded consumer model

## 7. Current Freeze Candidate Reading
After the final OxFml update in this exchange, the current first freeze candidate for `W048` is:
1. ordinary value,
2. `ValueWithPresentation`,
3. typed host/provider outcome projection,
4. no further factorization unless concrete implementation evidence exposes a mismatch.
