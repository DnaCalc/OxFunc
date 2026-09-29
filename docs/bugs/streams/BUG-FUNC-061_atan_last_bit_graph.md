# BUG-FUNC-061: ATAN last-bit arithmetic graph

Status: `investigating`; owner W111; bead `oxf-mwue.28.12`.

ATAN public FPATAN candidate matches 1224 discovery rows and 297 independent typed cases. A frozen independent 29573-row numeric cohort exposes two one-ULP counterexamples. Preserve the failed candidate, repeat the exact inputs and discriminate arithmetic graphs; do not promote whole-function or alternate-platform parity.

Reference: public-interface Excel 16.0 build 20430, 64-bit, Compatibility Version
2; update channel is unverified. Evidence is retained under
`docs/function-lane/evidence/w111-broad-20260929/atan/`.
Only black-box observations and public arithmetic operations inform this work.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: the stated residuals and refined
independent validation, context/platform limits, evaluator-visible assessment,
formal alignment and combined verification.
