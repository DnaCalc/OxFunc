# OxFml / OxFunc `LET` / `LAMBDA` Pin-Down Response V1

Status: `provisional_historical_narrowing`
Owner lane: `OxFunc`
Purpose: answer the then-current OxFml `LET` / `LAMBDA` pin-down prep note with the smallest next-step OxFunc position that was concrete enough to guide the next seam round without pretending the final carrier was already locked.

Round-position note:
1. this note records an earlier narrowing candidate rather than the final callable/library-context round closure,
2. the active round-closure reading now lives in `docs/upstream/NOTES_FOR_OXFML.md`,
3. the earlier callable field-lock follow-on is now historical provenance behind `OxFunc_V1`; the active parked shared-model anchor is `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`.

Primary upstream counterpart:
1. `../OxFml/docs/spec/formula-language/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_PREP.md`

Primary local empirical basis:
1. `docs/function-lane/W38_EXECUTION_RECORD.md`
2. `docs/function-lane/FUNCTION_SLICE_FUNCTIONAL_LAMBDA_AND_HELPER_STAGE1_CONTRACT_PRELIM.md`
3. `docs/function-lane/FUNCTION_SLICE_FUNCTIONAL_LAMBDA_HELPERS_STAGE2_MAP_REDUCE_SCAN_CONTRACT_PRELIM.md`

## 1. Core Position
OxFunc thinks the `LET` / `LAMBDA` seam is now ready to narrow one step further, but still not ready for a final shared callable ABI.

The strongest stable position at that narrowing stage was:
1. lexical meaning was treated as fixed enough to narrow further,
2. exact capture was treated as fixed enough to narrow further where the semantic producer can know it,
3. callable values are semantically first-class,
4. OxFunc should consume callables through a small typed carrier plus typed invocation,
5. parameter-name lists, exact capture-name lists, and body-detail should remain provenance/replay detail unless a later function-semantic case proves they must cross in the minimum carrier.

## 2. Fixed Truths OxFunc Accepts
OxFunc accepted the following as effectively fixed enough for that next round.

### 2.1 Lexical, Not Dynamic
1. Created helper lambdas preserve lexical meaning in the admitted local evidence.
2. Later helper-name shadowing must not rebind an already-created lambda dynamically.
3. Any seam carrier that loses lexical meaning is too weak.

### 2.2 Exact Capture, Not Approximate Capture
1. Unused helper bindings should not be reported as captures when the semantic producer can determine that exactly.
2. Helper names shadowed by lambda parameters should not be reported as captures when the semantic producer can determine that exactly.
3. Replay/explain detail may remain richer than the minimum shared carrier, but the semantic truth should still be exact.

### 2.3 Callable Values Are Real Semantic Values
1. Callable values are first-class in the semantic value universe.
2. Publication restrictions remain a separate question.
3. Defined Name callable preservation and later UDF/interoperable callable preservation should therefore be modeled as value-preservation questions, not as parser-only special cases.

## 3. What W38 Now Adds
`W38` materially changes the OxFunc side of this seam because it gives us direct Excel evidence, not just seam theory.

Boundary note:
1. this is local OxFunc evidence,
2. it should inform seam pressure,
3. it should not by itself be read as upstream shared seam-lock evidence.

Pinned local empirical facts:
1. duplicate `LET` names are rejected at formula admission,
2. duplicate `LAMBDA` parameter names are rejected at formula admission,
3. immediate `LAMBDA` invocation works on the worksheet surface,
4. lexical capture from `LET` into immediate `LAMBDA` is visible and required,
5. bare uninvoked `LAMBDA(...)` publishes as `#CALC!` rather than as a cell-display callable value,
6. direct under/over-application yields `#VALUE!`,
7. `MAP`, `REDUCE`, and `SCAN` prove that callable invocation must work inside higher-order function semantics,
8. `MAP` and `SCAN` also prove that callable invocation is coupled to array publication shape, not just scalar evaluation.

## 4. Minimum Callable Carrier OxFunc Wants
OxFunc's answer at that stage to OxFml's Q1 was that the smallest honest shared callable carrier looked like:

1. `callable_token`
   - opaque stable handle/token within the active semantic-plan or evaluation context
2. `origin_kind`
   - candidate examples at that stage included:
     - `helper_lambda`
     - `defined_name_callable`
     - `built_in_callable`
     - `external_registered_callable`
3. `arity_shape`
   - enough to distinguish exact or minimum/maximum callable arity for admission and invocation checks
4. `capture_mode`
   - at minimum:
     - `no_capture`
     - `lexical_capture`
5. `invocation_contract_ref`
   - stable reference to the callable invocation semantics/profile that OxFunc can use when it needs typed invocation

## 5. What Does Not Need To Be In The Minimum Carrier Yet
OxFunc's current answer to OxFml's Q3 is that these should remain provenance/replay detail unless later evidence proves otherwise:

1. parameter-name surface,
2. exact capture-name surface,
3. helper source span,
4. body-kind detail beyond what `invocation_contract_ref` already classifies,
5. explanatory summary text,
6. defined-name textual origin surface.

Current split:
1. carrier:
   - identity
   - origin kind
   - arity
   - capture mode
   - invocation contract reference
2. provenance/replay:
   - names
   - exact captured-name lists
   - helper/body source detail
   - richer explanatory metadata

## 6. Invocation Boundary OxFunc Prefers
OxFunc's answer to OxFml's Q2 is:
1. typed invocation against an opaque callable token is the preferred boundary,
2. richer callable inspection should not be required for the current family semantics,
3. OxFunc should not need helper AST ownership.

The working invocation shape OxFunc wants is conceptually:

```text
invoke_callable(
  callable_token,
  prepared_args,
  caller_context,
  invocation_contract_ref
) -> prepared_result
```

Current working split:
1. OxFml owns:
   - helper formation,
   - closure/capture truth,
   - callable token construction,
   - invocation service semantics for that token
2. OxFunc owns:
   - when and how a function such as `MAP` / `REDUCE` / `SCAN` invokes the callable,
   - the surrounding function semantics,
   - the interpretation of the returned prepared result inside that function's kernel/publication logic

## 7. Callable-Specific Stage Interaction
OxFunc's answer to OxFml's Q4 is deliberately narrow.

The main rule is:
1. a callable carrier should normally only exist after successful parse/bind/admission for that callable-producing expression,
2. therefore most callable-specific failure is not a special new stage; it is staged through the already-agreed library-context and runtime-capability taxonomy.

Current stage reading:
1. parse/bind stage:
   - duplicate helper names,
   - duplicate lambda parameters,
   - malformed helper shape,
   - unknown callable source names
2. prepared-call / invocation stage:
   - direct or higher-order callable arity mismatch,
   - unsupported invocation shape,
   - typed callable-invocation denial if the invocation service cannot legally apply the callable
3. post-dispatch/result stage:
   - worksheet-surface result such as `#CALC!` or `#VALUE!`,
   - any future external/provider failure for externally registered callable origins

OxFunc does not currently see a need for a callable-specific provider taxonomy beyond the generic stage-aware availability model unless and until external registered callables prove they need it.

## 8. What OxFunc Thinks Can Be Pinned Next
The next narrow pin-down that had looked achievable at that stage was:
1. accept the fixed lexical/capture truths,
2. accept the minimum callable carrier fields above,
3. accept that exact parameter/capture-name detail can stay provenance-side for now,
4. accept typed invocation over opaque callable tokens as the preferred boundary.

That is enough to support:
1. direct `LAMBDA` invocation,
2. `LET` + `LAMBDA` capture-sensitive lanes,
3. higher-order helpers such as `MAP`, `REDUCE`, and `SCAN`,
without prematurely locking the full future callable ABI.

## 9. What Remains Intentionally Open
This response did not try to close:
1. the final concrete callable transport/ABI,
2. fuller UDF/interoperable callable transport beyond the Excel-supported Defined Name callable surface,
3. full publication policy for callable values,
4. `BYROW`, `BYCOL`, and `MAKEARRAY`,
5. broader callable/provider interaction beyond the generic staged availability model.

## 10. Recommended Upstream Next Step
The next useful OxFml response would be:
1. whether the five minimum carrier fields above are acceptable,
2. whether OxFml wants `invocation_contract_ref` as a stable symbolic id or some smaller invocation-mode summary,
3. whether OxFml sees any current exercised case that forces exact capture-name or parameter-name lists into the minimum carrier rather than provenance/replay detail,
4. whether the typed-invocation-over-opaque-token boundary matches OxFml's current implementation direction closely enough to start narrowing canonical seam docs.
