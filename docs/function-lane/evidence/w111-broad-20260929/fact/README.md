# FACT sign admission and numerical replay

State: `in_progress`; `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Owner: `oxf-mwue.28.1`, BUG-FUNC-050. Open lanes: full typed/context coverage,
combined validation and completion audits; no whole-function promotion.

The current reference is Excel 16.0 build 20430, 64-bit, Compatibility Version 2,
1900 date system, channel unverified. Arguments are exact binary64 Value2 inputs.
The numerical repair rejects the original negative sign before truncation:
`FACT(-0.1)` is `#NUM!`. Positive fractions truncate; values at or above 171 reject
before integer conversion. FACTDOUBLE has a different negative-domain rule and
was not changed to inherit this one.

The retained reverse product computes `n * (n-1) * ... * 2`, publishing each
binary64 multiplication. `w111_fact_live_replay.rs` exercises actual dispatch
against 1,209 discovery and 1,509 independent heldout observations: 2,718 exact
results, including typed worksheet errors as well as numerical output bits.
Six focused runtime tests cover admission, overflow and large reverse products.
Lean `Fact.lean` includes the sign/bounds model and an executable reverse binary64
product binding with captured 25!, 100! and 170! output encodings.

The repair preserves Rust signatures and existing function metadata. Shared
numeric-text grammar and locale/evaluation-date preparation remain qualified by
BUG-FUNC-057 and HO-FN-022. XLL host-context recreation does not establish the
missing core preparation contract. Locale and alternate-version sweeps remain
separate validation phases; known current-reference text-context gaps still
prevent a whole-function claim.
