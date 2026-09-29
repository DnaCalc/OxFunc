# BUG-FUNC-060: conditional coercion, shape and missing values

Status: `investigating`; owner W111; bead `oxf-mwue.28.11`.

IF/IFERROR/IFNA discovery exposed 324 differences among 716 typed observations.
The revised prepared-value adapters match those and 96 additional shape controls.
The candidate is frozen before a separate 660-case holdout. Exact fixture values,
formula strings and per-cell errors are retained under
`docs/function-lane/evidence/w111-broad-20260929/conditional/`.

Reference: Excel 16.0 build 20430, 64-bit, Compatibility Version 2; channel is
unverified. Corrections include strict IF logical text, singleton broadcasting,
unused-branch shape contributions, catchable primary padding, and numeric zero
for selected blank/missing arguments. An omitted IF false branch remains FALSE.

HO-FN-023 requests receiving-evaluator assessment. Already evaluated arguments
do not establish expression scheduling, volatility, effects or reference-return
selection. The prior IF/IFERROR phase claims are withdrawn for the expanded scope.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: independent replay, uncommon and
reference-returning forms, host publication, evaluator scheduling, receiving
acknowledgment and integration, combined validation.
