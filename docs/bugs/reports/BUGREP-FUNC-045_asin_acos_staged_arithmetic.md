# BUGREP-FUNC-045: ASIN reused-factor sign asymmetry and ACOS staged subtraction

Status: `triaged`; canonical stream `BUG-FUNC-066`; owner `oxf-mwue.28.16`.

ASIN reuses the stored first factor when forming its second factor, explaining observed non-odd rounding without a sign-specific exception. Staged square root/division and reduced arctangent match22192 retained observations; staged ACOS subtraction matches14232. The latest frozen independent capture adds8052 inputs per function. Numerical primitive formal identity, other platforms and full surrounding coercion remain separate open lanes.

All three completion axes remain partial; open lanes are listed in the canonical stream.
