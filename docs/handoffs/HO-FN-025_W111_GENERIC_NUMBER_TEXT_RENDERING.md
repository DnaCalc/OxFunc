# HO-FN-025: shared number-to-text preparation

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.15`.

Generic text preparation previously rendered numbers with Rust's shortest
decimal display. Exact-ingress Excel observations instead support the observed
ADDRESS numeric-sheet rendering policy: a 15-digit initial representation,
fixed versus scientific width rules, and a further digit reduction for large
exponents. Discovery contains 6,961 generic/ADDRESS observations; a frozen fresh
independent sample adds 8,208 exact matches with no exclusions. Deliberately
difficult decimal-midpoint inputs remain under separate probing, so this is
an evidenced candidate rather than universal formatting identity.

See the [impact assessment](../function-lane/evidence/w111-broad-20260929/numeric-to-text/IMPACT_ASSESSMENT.md)
for all 28 affected source modules and the [retained evidence](../function-lane/evidence/w111-broad-20260929/numeric-to-text/README.md).
The change factors a private finite-number helper into the Number branch of
shared text coercion and ADDRESS. Other scalar kinds and reference preparation
retain their policies. COMPLEX has distinct observed coefficient rendering and
does not use this generic conversion. Nonfinite/subnormal direct carrier values
are outside the admitted Value2 evidence.

The evaluator-visible effect extends beyond returned text: several consumers
parse selectors or forward strings to host services. Matching the shared helper
does not characterize those downstream services. The reference is Excel 16.0
build 20430, CV2, with the recorded oracle locale; no new hardcoded locale or
current-year contract is introduced.

Receiving-side work: retain binary64 input bits until this conversion, exercise
generic coercion through evaluator and reference/array preparation, distinguish
generic coercion from explicit formatting and COMPLEX, assess host-service
consumers, and acknowledge the shared policy change. Record integration evidence
before either repository promotes this seam.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: exceptional midpoint arithmetic,
unexercised consumers, explicit serializers, locale/compatibility context,
receiving acknowledgment and integration.
