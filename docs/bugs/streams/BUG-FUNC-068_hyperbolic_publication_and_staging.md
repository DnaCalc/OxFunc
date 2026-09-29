# BUG-FUNC-068: Hyperbolic publication and staged arithmetic

Status: `investigating`; owner W111; bead `oxf-mwue.28.18`.

COSH matches 32,090 discovery and independent numeric observations. SINH and CSCH retained six failures in the first independent bank. The small-input TANH denominator refinement matches all discovery rows and reduces the earlier independent failures to six TANH and five COTH. A subsequent 7,142-row bank per function exposes 52 TANH and 50 COTH failures; direct paired SINH/COSH controls locate all 52 TANH failures in SINH with denominator one. These failures remain under general arithmetic-graph investigation; no input-specific correction or whole-function promotion is used.

Reference: Excel 16.0 build 20430, 64-bit, workbook Compatibility Version 2;
channel unverified. Evidence uses normal binary64 inputs assigned through
Value2 and exact returned bits. Public interfaces only. Retained evidence:
`docs/function-lane/evidence/w111-broad-20260929/hyperbolic/`.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: stated numerical residuals,
fresh independent validation, typed preparation, primitive/platform alignment,
evaluator integration and combined campaign verification.
