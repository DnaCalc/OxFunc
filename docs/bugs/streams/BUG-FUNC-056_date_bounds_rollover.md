# BUG-FUNC-056: Date serial bounds, conversion and rollover

Status: `investigating`; owner W111; bead `oxf-mwue.28.7`.

The numeric discovery exposes missing upper and raw-negative bounds, half-second serial conversion, 1900 rollover differences and fractional selector conversion. Repeated unsupported WEEKNUM selectors produce contradictory Excel answers for identical inputs; those observations remain retained and cannot establish a deterministic repair.

Baseline: `11b23504fe8180d08397be5435bd215e1bbb9722`. Candidate source is the 2026-09-29 working tree.
Live reference: Excel16.0 build20430,64-bit,CV2,1900date system; channel unverified.
Evidence: [campaign](../../function-lane/evidence/w111-broad-20260929/README.md)
and family observations under `docs/function-lane/evidence/w111-broad-20260929/dates`.
Only public interfaces and black-box observations are used. Numeric bits and exact
text ingress are part of qualification, not inferred from formula appearance.

Cross-repo impact: numeric-kernel repairs preserve current signatures; the implicit
text parser's locale and clock dependency is an open evaluator-facing question.
Any context-contract change needs an OxFml handoff and acknowledgment.

`scope_completeness: scope_partial`; `target_completeness: target_partial`;
`integration_completeness: partial`. Open lanes: residual behavior, independent
heldouts, all typed axes, formal alignment, combined validation and OPERATIONS12/14
audits. No whole-function completion claim is made.
