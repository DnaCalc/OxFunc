# OxFml / OxFunc Minimum Stabilization Response V1

Status: `provisional`
Owner lane: `OxFunc`
Purpose: provide a narrower OxFunc-side response after OxFml accepted the current stabilization order:
1. external library-context snapshot,
2. availability / feature-gate / provider-failure taxonomy,
3. callable-value minimum carrier.

This note is intentionally narrower than:
1. `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md`
2. `docs/function-lane/OXFML_OXFUNC_NEXT_ROUND_STABILIZATION_TOPICS.md`

It focuses only on what OxFunc can now say concretely without over-freezing transport shape.

## 1. Topic A - Minimum Library-Context Snapshot
### 1.1 Core Position
OxFunc thinks the shared library-context snapshot should be just rich enough to support:
1. parse/bind,
2. early formula rejection where honest,
3. semantic-plan formation,
4. evaluation dispatch,
5. replay/proving-host identity.

It should not yet try to encode:
1. all runtime capability states,
2. all provider/session states,
3. full publication behavior.

### 1.2 Minimum Semantic Fields
For each callable entry, the snapshot should preserve at least:
1. stable callable id
   - canonical function id or operator id
2. callable kind
   - built-in worksheet function
   - built-in operator
   - externally registered function
   - callable value entry if the catalog ever exposes one directly
3. surface names
   - canonical name
   - aliases
   - compatibility names
   - localized names
4. admission/profile facts
   - arity
   - profile/trait references needed for parse/bind and semantic planning
5. static gating facts
   - compatibility gates
   - feature/version gates
   - static add-in/catalog presence truth
6. replay identity fields
   - snapshot id
   - snapshot version or generation

### 1.3 Parse-Time Versus Bind-Time Use
Current OxFunc view:
1. parse time should only need enough to recognize whether a token sequence could name a callable surface in the current localized context.
2. bind time should use the richer fields:
   - canonical id,
   - alias resolution,
   - profile references,
   - static gates,
   - callable kind.
3. evaluation time should consume the same canonical id/profile link rather than redoing surface-name logic.

### 1.4 What Should Stay Out Of The Snapshot
OxFunc does not currently think the library-context snapshot should carry:
1. host-session-denied states,
2. active provider health,
3. active selection,
4. caller cell,
5. host-query result payloads,
6. runtime capability denials.

Those belong later in:
1. host capability views,
2. runtime capability evaluation,
3. prepared-call or publication surfaces.

### 1.5 Current Open Questions For OxFml
1. Does OxFml want one snapshot object or split views for name lookup and semantic traits?
2. What snapshot identity fields would OxFml like preserved in replay/proving-host artifacts?
3. Which profile references need to be directly dereferenceable by OxFml versus only stable ids into OxFunc-owned catalog data?

## 2. Topic B - Availability / Gating / Failure Taxonomy
### 2.1 Core Position
OxFunc thinks the main current seam need is not exhaustive error mapping.
It is explicit classification.

The system should distinguish at least:
1. not known in library context,
2. known but statically gated,
3. known and admissible in catalog terms,
4. known but runtime capability denied,
5. known but provider unavailable,
6. known and dispatched but provider/runtime operation failed after dispatch.

### 2.2 Minimum Taxonomy
Current proposed minimum vocabulary:
1. `unknown_name`
   - not present in the current library context
2. `static_feature_gate`
   - known entry, but blocked by version/channel/feature gate
3. `static_compat_gate`
   - known entry, but blocked by workbook compatibility policy
4. `catalog_present`
   - known and admitted at the library-context level
5. `runtime_capability_denied`
   - known entry, but required runtime host/session capability is unavailable
6. `provider_unavailable`
   - known entry, but external provider/service is not available
7. `provider_failure`
   - dispatch happened, but provider/service failed after entry

### 2.3 Why This Matters
Processed functions already pressure this split:
1. `EUROCONVERT`
2. `RANDARRA`
3. `TRANSLATE`
4. locale/profile-sensitive width-conversion functions
5. later host/add-in/provider-sensitive functions

Without this split, the system risks collapsing:
1. early rejection,
2. `#NAME?`,
3. typed capability denial,
4. provider failure,
into one opaque unavailable state.

### 2.4 OxFunc Boundary Preference
Current OxFunc preference:
1. library-context truth should answer:
   - is the callable known here,
   - what static gates apply.
2. runtime capability view should answer:
   - is the needed host/provider capability available now.
3. post-dispatch result classification should answer:
   - did provider execution fail after honest dispatch.

### 2.5 What Still Should Stay Open
OxFunc does not think we need to lock yet:
1. exact worksheet error mapping for every state,
2. full provider subtype taxonomy,
3. every locale/version/profile cross-product.

We do need the states above to remain distinguishable in:
1. semantic planning,
2. runtime classification,
3. replay/explain artifacts.

## 3. Topic C - Callable-Value Minimum Carrier
### 3.1 Core Position
OxFunc does not currently need a rich callable AST carrier.
It does need callable values to remain first-class semantic values with lexical meaning preserved.

### 3.2 Minimum Facts OxFunc Currently Needs
At minimum, OxFunc would want a callable carrier or callable summary that preserves:
1. stable callable identity
2. callable kind
   - helper-created lambda
   - built-in callable
   - externally registered callable
3. arity or callable parameter profile summary
4. lexical capture preservation
   - even if capture payload itself remains opaque
5. typed invocation route
   - OxFunc must be able to request invocation without re-owning helper syntax
6. publication restriction separate from semantic existence

### 3.3 What OxFunc Does Not Currently Need
OxFunc does not currently think it needs:
1. raw helper AST transport,
2. full closure-environment object graphs,
3. syntax-tree ownership for helper bodies,
4. one final ABI for every callable flavor.

An opaque callable token plus summary plus typed invocation could be sufficient if:
1. lexical meaning is preserved,
2. replay/proving-host artifacts can still identify callable behavior meaningfully enough for seam diagnostics.

### 3.4 Defined Names And Interop
OxFunc now explicitly wants room for callable values to exist as:
1. helper-produced values,
2. function arguments,
3. function return values,
4. defined-name values,
5. UDF interop payloads.

This includes the stated prototype direction where native and user-defined lambdas may travel through an interop carrier such as `IExcelLambda` or equivalent.

Current OxFunc position:
1. "not a normal cell-display value" must not be interpreted as "not a value".
2. publication restrictions belong above the semantic value universe.

### 3.5 Current Open Questions For OxFml
1. Can OxFml expose an opaque callable token plus summary plus typed invocation while still preserving lexical meaning fully?
2. Which callable facts does OxFml want carried into replay/proving-host artifacts?
3. How does OxFml currently want callable defined-name values represented in semantic plans?

## 4. Working Recommendation
For the next seam round, OxFunc suggests:
1. narrow Topic A enough to define a minimum shared library-context snapshot field set,
2. narrow Topic B enough to define a shared availability/gating classification vocabulary,
3. only then narrow Topic C enough to agree what an opaque-but-honest callable minimum carrier must preserve.

That order matches OxFml's current stated stabilization order and should produce the most useful next convergence without forcing premature global closure.
