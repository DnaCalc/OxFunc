# WORKSET - Functional Lambda And Helper Family (W38)

## 1. Purpose
Open the first dedicated packet for Excel's functional helper and lambda family.

This packet exists to make the lambda/helper cluster explicit rather than leaving it as a diffuse "interesting later" area.

## 2. Provenance
Opened after:
1. the OxFml/OxFunc seam round made callable values, helper binding, and first-class lambda values explicit cross-repo topics,
2. the interesting-function review identified the lambda/helper cluster as the highest-value remaining in-core interesting family,
3. `W033` closed the latest small ordinary interesting packet and freed the next interesting batch slot.

Relevant upstream/local context:
1. `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md`
2. `docs/function-lane/OXFML_OXFUNC_NEXT_ROUND_STABILIZATION_TOPICS.md`
3. `docs/upstream/NOTES_FOR_OXFML.md`
4. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
5. `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv`

Backlog ownership note:
1. `W038` remains the provenance/evidence owner and likely execution owner for the callable-helper family.
2. Active current-version backlog tracking now sits in `W051`.

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W38_FUNCTIONAL_LAMBDA_AND_HELPER_INVENTORY.csv`

Current total:
1. `9` functions.

Members:
1. `BYCOL`
2. `BYROW`
3. `ISOMITTED`
4. `LAMBDA`
5. `LET`
6. `MAKEARRAY`
7. `MAP`
8. `REDUCE`
9. `SCAN`

## 4. Why This Packet Matters
1. This is the strongest remaining in-core seam cluster after the recent OxFml/OxFunc note round.
2. It pressures callable values, lexical capture, omission, helper scoping, and lambda invocation semantics.
3. It is one of the best opportunities to define Excel's hidden machinery explicitly rather than treating helper evaluation as opaque host magic.
4. It is also the most likely packet to refine the future library-context and prepared-call boundary without requiring a final transport lock too early.

## 5. In Scope
1. empirical characterization of helper syntax/evaluation behavior for the admitted current-baseline slice,
2. explicit contract work for helper binding, lambda creation, lambda invocation, and omission handling,
3. runtime substrate work for first-class callable values in the semantic value universe,
4. Lean/formal alignment at the executable substrate level for helper/callable semantics,
5. scenario manifests, runtime requirements, execution record, and evidence registry rows,
6. explicit seam statements for what remains in OxFml versus what OxFunc must see as prepared helper/callable artifacts,
7. Defined Name preservation of callable/lambda values on the current Excel-supported surface.

## 6. Out Of Scope
1. broad future callable transport/ABI beyond the current minimum shared callable carrier,
2. final `@` / `SINGLE` placement,
3. unrelated dynamic-array reshaping functions that do not intrinsically require lambda/helper semantics,
4. external-provider or host-query functions,
5. UDF/interoperable callable origins, returns, and transport beyond the Excel-supported Defined Name callable surface.

## 7. Gate Criteria
This workset can only be reported `scope_complete` when:
1. each inventory member has reproducible native evidence for the admitted current-baseline slice,
2. the helper/lambda packet has a shared contract that states:
   - helper-binding order,
   - lexical capture expectations,
   - lambda argument admission,
   - omission semantics,
   - Defined Name callable preservation expectations for the admitted slice,
3. the runtime value universe and callable substrate are updated honestly for the admitted slice,
4. Lean/formal alignment for the primary callable/helper substrate is integrated,
5. no known function-semantic gap remains in declared current-baseline scope for the admitted slice.

## 8. Initial Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`

Current status reading:
1. the admitted Stage 1/2/3 callable-helper slice is now complete for declared current-phase scope across OxFunc runtime/formal/native evidence and the OxFml adapter corpus.
2. formation and bind validation remain explicitly OxFml-owned where Excel rejects before evaluation.
3. richer callable replay/explain/serialization detail remains a cooperative future concern with OxFml, not a blocker to current `W38` closure.

## 9. XLL Seam Note
1. The current XLL bridge does not carry callable worksheet values or workbook Defined Name callable bindings end-to-end.
2. For `W38`, that is an external seam limitation, not an open function-semantic lane.
3. `W38` closure should therefore depend on native worksheet evidence, core/runtime alignment, and formal alignment for the declared slice, not on future XLL callable transport.

Current OxFunc-side completion reading:
1. all nine inventory members now have native current-baseline evidence and OxFunc-side runtime/formal artifacts for the admitted slice.
2. the OxFml/OxFunc callable freeze now keeps the minimum shared carrier and bind/admission ownership honest across the seam.
3. duplicate-name and malformed-helper rejection remain explicit OxFml-owned bind behavior where Excel rejects before evaluation.
4. widening beyond the admitted slice remains evidence-driven future scope, not a blocker to current `W38` closure.
