# BUG-FUNC-066: ASIN reused-factor sign asymmetry and ACOS staged subtraction

Status: `investigating`; owner W111; bead `oxf-mwue.28.16`.

ASIN reuses the stored first factor when forming its second factor, explaining observed non-odd rounding without a sign-specific exception. Staged square root/division and reduced arctangent match 22192 retained observations; staged ACOS subtraction matches 14232. The latest frozen independent capture adds 8052 inputs per function. Numerical primitive formal identity, other platforms and full surrounding coercion remain separate open lanes.

Reference: public-interface Excel 16.0 build 20430, 64-bit, workbook Compatibility
Version 2; channel unverified. Retained evidence:
`docs/function-lane/evidence/w111-broad-20260929/asin/`.
All observations use exact numeric Value 2 inputs or explicitly qualified typed
fixtures. Discovery and failed independent candidates remain distinct.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: stated residuals, independent
refinement validation, primitive/platform alignment, preparation/host context,
evaluator integration and combined campaign verification. No whole-function
completion claim is made.
