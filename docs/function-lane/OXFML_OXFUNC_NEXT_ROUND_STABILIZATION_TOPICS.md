# OxFml / OxFunc Next-Round Stabilization Topics

Status: `provisional`
Owner lane: `OxFunc`
Purpose: narrow the broader hidden-machinery seam model into the few topics that are most ready for concrete cross-repo stabilization in the next round.

## 1. Why This Narrowing Exists
The central seam model in:
1. `docs/function-lane/OXFML_OXFUNC_HIDDEN_MACHINERY_SEAM_EXPLICIT_MODEL.md`

is intentionally broad.

That is useful for mapping the hidden machinery, but the next integration round should focus on the smallest set of topics where:
1. semantic need is already clear,
2. OxFml has now acknowledged the direction,
3. transport can be narrowed somewhat without pretending the whole seam is solved.

## 2. Topics To Stabilize First
The current best candidates are:
1. external library-context snapshot,
2. callable-value minimum carrier,
3. availability / feature-gate / provider-failure taxonomy.

These three topics have the best ratio of:
1. immediate design value,
2. already-processed cross-repo acknowledgment,
3. ability to improve both parse/bind and evaluation without reopening the whole seam at once.

## 3. Topic A - External Library-Context Snapshot
### 3.1 Why First
This topic is now pressure-tested by:
1. OxFunc ownership of function/operator catalog semantics and profiles,
2. need for OxFml parse/bind and early rejection,
3. dynamic function registration,
4. localized function names,
5. desire to keep OxFunc runtime-state-free.

### 3.2 Semantic Minimum
The shared snapshot should be able to answer:
1. which names are known in the current context,
2. canonical function/operator id,
3. alias and compatibility names,
4. localized surface names,
5. arity/profile and capability declarations,
6. feature/version/profile gates,
7. whether a function is built-in vs externally registered.

### 3.3 What Can Progress Now
We can stabilize:
1. the fact that OxFml should bind against an external context snapshot rather than a hidden global registry,
2. the minimum semantic fields above,
3. the distinction between library-context truth and runtime capability truth.

We should not yet try to freeze:
1. exact Rust/JSON/ABI carrier types,
2. final mutability/update API,
3. exact localized-name table mechanics.

### 3.4 Concrete Questions For OxFml
1. Which of the minimum fields above must be visible at parse time versus bind time?
2. Does OxFml want one snapshot object or split name-table / profile-table views?
3. What snapshot identity/version fields would OxFml want for replay/proving-host artifacts?

## 4. Topic B - Callable-Value Minimum Carrier
### 4.1 Why First
This topic is now pressure-tested by:
1. exercised `LET` / `LAMBDA` local floors in OxFml,
2. lexical capture requirements,
3. higher-order and curry-style behavior,
4. user-defined and native lambda interop direction,
5. the need to treat callable values as first-class semantic values even if not ordinary cell-display values.

### 4.2 Semantic Minimum
The seam should preserve at least:
1. callable identity,
2. callable kind:
   - native helper lambda,
   - built-in callable,
   - externally registered callable,
3. lexical capture preservation,
4. parameter-count / callable-profile summary,
5. typed invocation path,
6. publication restriction being separate from semantic existence.

### 4.3 What Can Progress Now
We can stabilize:
1. callable values are first-class semantic values,
2. publication restrictions do not mean "not a value",
3. OxFunc should not need raw helper AST ownership,
4. the carrier may remain minimal if typed invocation is available.

We should not yet try to freeze:
1. final callable ABI,
2. exact COM carrier design,
3. whether all callable values share one concrete runtime representation.

### 4.4 Concrete Questions For OxFml
1. Can OxFml expose an opaque callable token plus summary plus typed invocation and still preserve lexical meaning?
2. Which callable facts must survive into replay/proving-host artifacts?
3. How does OxFml currently want defined-name callable values to appear in semantic plans?

## 5. Topic C - Availability / Gating / Failure Taxonomy
### 5.1 Why First
This topic is now pressure-tested by processed functions including:
1. `EUROCONVERT`,
2. `RANDARRAY`,
3. `TRANSLATE`,
4. locale/profile-sensitive width-conversion functions,
5. other feature- and provider-sensitive surfaces.

### 5.2 Semantic Minimum
The seam should distinguish at least:
1. known in library context,
2. unknown name,
3. feature-gated,
4. compatibility-gated,
5. host-profile unavailable,
6. add-in absent,
7. provider unavailable,
8. capability denied at runtime,
9. provider failure after successful dispatch.

### 5.3 What Can Progress Now
We can stabilize:
1. this taxonomy as a shared semantic distinction set,
2. the rule that library-context availability truth is not the same as runtime capability/provider truth,
3. the need for replay/explain artifacts to preserve those distinctions.

We should not yet try to freeze:
1. the exact worksheet-surface mapping for every state,
2. every version/channel/profile cross-product,
3. all provider-specific failure subclasses.

### 5.4 Concrete Questions For OxFml
1. Which states should OxFml surface before evaluation begins?
2. Which states belong only in runtime capability views?
3. How should early formula rejection differ from runtime `#NAME?`, typed denial, and provider-failure outcomes?

## 6. Topics To Keep Open For Later
These remain important, but should not be the first stabilization pass:
1. final provenance vocabulary in full,
2. final placement of `@`,
3. final `_xlfn.SINGLE(...)` round-trip handling,
4. broader host-query carrier shapes,
5. full operator/literal catalog split,
6. full publication model for callable values.

## 7. Best Current Division Of Work
### 7.1 OxFunc Can Progress Now
1. sharpen the semantic minima for the three topics above,
2. keep mapping processed function evidence back to those topics,
3. avoid overcommitting transport shape,
4. keep value-universe and capability distinctions explicit.

### 7.2 OxFml Can Progress Now
1. say where each distinction naturally lives:
   - library context,
   - semantic plan,
   - prepared argument/result,
   - host capability view,
   - publication/replay artifact.
2. identify which of the three topics should be narrowed first in canonical docs,
3. keep the rest open deliberately rather than accidentally.

## 8. Working Recommendation
For the next cross-repo seam round:
1. converge first on topic A and topic C enough to support parse/bind and runtime classification coherently,
2. converge second on topic B enough to avoid losing callable-value semantics,
3. leave the broader seam model in place as the context for later rounds rather than trying to close everything at once.
