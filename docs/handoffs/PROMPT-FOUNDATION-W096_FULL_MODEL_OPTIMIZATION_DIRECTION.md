# Prompt For Foundation: W096 Full-Model Optimization Direction

Use this prompt in the Foundation repo to record the cross-program architecture
direction clarified during OxFunc W096.

```text
Please update the Foundation doctrine and architecture docs to record the
clarified DNA Calc / OxFml / OxFunc full-model optimization direction from
OxFunc W096.

Context:

OxFunc W096 established that the long-term target is not only isolated
function-call correctness. DNA Calc should eventually recalculate large
Excel-compatible models very quickly through graph-level planning, concurrency,
caching, and possible compiled backends. That future optimization must not move
function semantics out of OxFunc or create evaluator-local special cases in
OxFml/DNA Calc.

Direction to encode:

1. OxFunc remains the canonical owner of worksheet value, function, and operator
   semantics.
2. OxFunc owns coercion, array lifting, reference-visible behavior,
   helper/callback semantics, error propagation, host/provider projection,
   volatility, locale dependency, and function-facing FEC dependency metadata.
3. OxFml owns formula structure: parse/bind, lexical slots, LET/LAMBDA binding,
   references, child evaluation order, lazy control forms, compiled formula
   plans, and trace publication policy.
4. FEC/host/DNA Calc layers own workbook-level scheduling, invalidation,
   publication, caching, concurrency, and graph/backend execution strategy.
5. Optimized evaluators should consume OxFunc resolved call-site handles and
   metadata rather than repeatedly passing strings through broad dispatch.
6. Optimized evaluators must not duplicate or special-case function semantics
   for hot functions such as INDEX, HSTACK/VSTACK, arithmetic operators,
   REDUCE/SCAN/MAP/BYROW/BYCOL/MAKEARRAY, or other helper functions.
7. Foundation should treat optimizer metadata as contract-relevant, not merely
   documentation: arity, argument preparation, volatility, determinism, host
   interaction, FEC dependency profiles, callable argument ordinals, reference
   visibility, shape behavior, array lifting, and hoistability under explicit
   runtime-context policy.
8. Value-only versus trace-rich execution is an evaluator/runtime mode decision.
   Trace requirements should not force OxFunc to allocate per-call trace
   structures in hot value paths, and value-only execution must preserve the same
   worksheet-visible semantics.
9. Future concurrent or compiled graph execution is allowed only when OxFunc and
   Foundation metadata prove the required purity, dependency, host-interaction,
   volatile, locale, random/time, and external-provider conditions.

Concrete OxFunc surfaces from W096 that Foundation should reference where useful:

1. `SurfaceCallSite`
2. `SurfaceCallRuntime`
3. `SurfaceCallScratch`
4. `SurfaceCallHoistPolicy`
5. `SurfaceDispatchKey`
6. generated/catalog-index surface dispatch
7. compatibility string APIs as wrappers over resolved dispatch

Please update at least:

1. `CHARTER.md` if it has program/lane ownership doctrine,
2. `ARCHITECTURE_AND_REQUIREMENTS.md` for the evaluation stack and optimization
   architecture,
3. the FEC/F3E interaction/model docs if they define scheduling, dependency, or
   function invocation contracts,
4. any prompt or operations guidance that could otherwise encourage
   evaluator-local function special cases.

Acceptance criteria:

1. The Foundation docs clearly state that high-throughput full-model
   optimization is a core architecture goal.
2. The docs clearly separate formula-plan ownership from function-semantic
   ownership.
3. The docs explicitly forbid moving function-specific semantics into OxFml or
   DNA Calc for speed.
4. The docs identify OxFunc metadata as the future scheduling/hoisting/compiled
   backend contract source.
5. The docs preserve clean-room and Excel-observable semantic identity
   requirements.
6. Any new completion language remains scoped: this direction is an
   architecture target, not a claim that whole-model optimization is already
   complete.
```
