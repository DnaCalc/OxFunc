# BUGREP-FUNC-033: SLN sign admission and stored arithmetic publication

Filed2026-09-29 against `11b23504fe8180d08397be5435bd215e1bbb9722` from the fresh black-box Excel campaign.
Status: `triaged`; canonical owner [BUG-FUNC-055](../streams/BUG-FUNC-055_sln_arithmetic_admission.md), bead `oxf-mwue.28.6`.

Negative cost/salvage/life and tiny nonzero life were incorrectly rejected. Discovery1205 and discriminator399 identify a stored subtraction, extended-precision quotient and final zero/subnormal publication. Candidate typed39 agree; independent7250 numeric holdout is pending.

Reproduction and exact captures: [campaign evidence](../../function-lane/evidence/w111-broad-20260929/README.md).
This report records discovery and candidate validation, not a function-phase
completion claim. Scope and target remain partial; integration is partial.
