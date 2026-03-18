# OxFml / OxFunc Minimum Stabilization Response V2

Status: `provisional`
Owner lane: `OxFunc`
Purpose: tighten the next-round seam discussion after the latest OxFml processing, focusing on:
1. operator admission vs grammar ownership,
2. minimum library-context snapshot fields,
3. minimum availability-classification mapping.

This note is intentionally narrower than:
1. `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md`
2. `docs/function-lane/OXFML_OXFUNC_NEXT_ROUND_STABILIZATION_TOPICS.md`
3. `docs/function-lane/OXFML_OXFUNC_MINIMUM_STABILIZATION_RESPONSE_V1.md`

## 1. Topic A - Operator Admission Versus Grammar Ownership
### 1.1 Core Position
OxFunc agrees that the operator/literal/value-universe tension should remain explicit.

Current best split:
1. OxFml owns:
   - lexical operator tokens,
   - precedence and associativity,
   - parse structure,
   - localized separators and literal spelling.
2. OxFunc owns:
   - canonical operator ids,
   - operator semantic meaning,
   - operator result classification,
   - operator profile metadata where operator behavior depends on the function catalog world.

### 1.2 Working Boundary
The cleanest current working rule is:
1. grammar decides whether a token sequence parses as an operator form,
2. library context decides whether the parsed operator form is admitted in the active semantic world,
3. OxFunc decides what the admitted operator means.

This is most relevant for:
1. version-gated operators,
2. compatibility-sensitive operator aliases,
3. operators that are syntax on the surface but semantically catalog-owned in OxFunc.

### 1.3 What OxFunc Thinks Should Be Catalog-Visible
OxFml should be able to ask the library context at least:
1. canonical operator id for a parsed operator form,
2. whether that operator is admitted in the active feature/compatibility profile,
3. any compatibility alias relationship that affects normalization or round-trip,
4. whether the operator is treated as an OxFunc-owned semantic operator rather than only parser punctuation.

### 1.4 What OxFunc Thinks Should Stay Grammar-Owned
OxFunc does not currently need ownership of:
1. precedence tables,
2. associativity tables,
3. exact token spelling rules,
4. localized separator grammar,
5. literal token boundary rules.

### 1.5 Open Question For OxFml
Current narrow question:
1. does OxFml want operator admission to be entirely derived from library-context facts after parse, or are there operator forms it expects to remain grammar-owned all the way through without catalog mediation?

## 2. Topic B - Minimum Library-Context Snapshot Fields
### 2.1 Refined Minimum Set
OxFunc now thinks the minimum shared library-context snapshot fields can be stated more concretely.

Per callable/operator entry:
1. `stable_id`
   - canonical function id or operator id
2. `entry_kind`
   - built_in_function
   - built_in_operator
   - external_registered_function
3. `surface_names`
   - canonical name
   - aliases
   - compatibility names
   - localized names
4. `arity_shape`
   - enough for parse/bind and early rejection
5. `profile_refs`
   - stable references to OxFunc-owned semantic/admission profile data
6. `static_gates`
   - feature/version gates
   - compatibility gates
   - static catalog/add-in presence truth
7. `snapshot_identity`
   - snapshot id
   - snapshot generation/version

### 2.2 Parse/Bind/Eval Use
Current refined view:
1. parse needs localized-name recognition only,
2. bind needs `stable_id`, `entry_kind`, `arity_shape`, `profile_refs`, and `static_gates`,
3. evaluation should consume only stable ids/profile refs, not re-run name resolution.

### 2.3 Explicit Exclusions
This minimum snapshot still should not carry:
1. caller cell or active selection,
2. runtime provider health,
3. runtime capability denial,
4. host-query payload facts,
5. final publication hints,
6. dynamic execution results.

### 2.4 Open Questions For OxFml
1. Is `arity_shape` enough, or does OxFml need a slightly richer parse-visible admission summary?
2. Are `profile_refs` best modeled as stable symbolic ids or as inlined summary descriptors in OxFml artifacts?
3. Does OxFml want one snapshot identity per formula compile session or per catalog generation?

## 3. Topic C - Minimum Availability Classification Mapping
### 3.1 Core Position
OxFunc thinks the next useful step is not more taxonomy names.
It is a mapping from classification stage to classification state.

### 3.2 Stage Mapping
Current proposed stage split:
1. parse/bind stage:
   - `unknown_name`
   - `catalog_present`
   - `static_feature_gate`
   - `static_compat_gate`
2. runtime capability stage:
   - `runtime_capability_denied`
   - `provider_unavailable`
3. post-dispatch result stage:
   - `provider_failure`

### 3.3 Why This Mapping Helps
This keeps distinct:
1. "not known here at all"
2. "known, but statically gated"
3. "known, but runtime service unavailable"
4. "known and dispatched, but service failed"

That is the minimum separation needed to avoid collapsing:
1. early formula rejection,
2. runtime `#NAME?`,
3. typed denial,
4. provider-failure explanations.

### 3.4 What OxFunc Is Not Trying To Lock Yet
This note does not try to lock:
1. exact worksheet error projection per state,
2. every provider subtype,
3. every locale/version/profile special case.

It only tries to preserve stage-correct classification.

### 3.5 Narrow Questions For OxFml
1. Does OxFml agree that `unknown_name` and static gating belong before runtime capability checks?
2. Does OxFml want `provider_unavailable` classified with generic runtime capability denial or separately?
3. In replay/explain artifacts, does OxFml want stage-of-classification recorded explicitly, or is the state label alone enough?

## 4. Current OxFunc Recommendation
For the next seam round, OxFunc recommends:
1. narrow the operator-admission boundary enough that OxFml and OxFunc agree what must be library-context-visible after parse,
2. narrow the library-context snapshot enough to support bind/eval identity without runtime leakage,
3. narrow the availability mapping enough to preserve stage-correct truth,
4. keep callable-value carrier design deliberately one step looser until the surrounding catalog/admission surfaces are more stable.
