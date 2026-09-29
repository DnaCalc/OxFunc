# BUG-FUNC-057: Implicit numeric text grammar and locale/date context

Status: `investigating`; owner W111; bead `oxf-mwue.28.8`.

The exact-ingress typed sweep has1208 differing rows among2544 across53functions. Percent, parentheses, locale-specific grouping/currency, Unicode digits, date/time and ASCII-only whitespace distinguish Excel from Rust trim/parse. ADDRESS720controls show missing-year text depends on current host date. Locale/clock dependencies require cross-repo assessment; no hardcoded date or unsupported provider substitution is authorized by this evidence.

Baseline: `11b23504fe8180d08397be5435bd215e1bbb9722`. Candidate source is the 2026-09-29 working tree.
Live reference: Excel16.0 build20430,64-bit,CV2,1900date system; channel unverified.
Evidence: [campaign](../../function-lane/evidence/w111-broad-20260929/README.md)
and family observations under `docs/function-lane/evidence/w111-broad-20260929/numeric-text`.
Only public interfaces and black-box observations are used. Numeric bits and exact
text ingress are part of qualification, not inferred from formula appearance.

Cross-repo impact: numeric-kernel repairs preserve current signatures; the implicit
text parser's locale and clock dependency is an open evaluator-facing question.
Any context-contract change needs an OxFml handoff and acknowledgment.

`scope_completeness: scope_partial`; `target_completeness: target_partial`;
`integration_completeness: partial`. Open lanes: residual behavior, independent
heldouts, all typed axes, formal alignment, combined validation and OPERATIONS12/14
audits. No whole-function completion claim is made.
