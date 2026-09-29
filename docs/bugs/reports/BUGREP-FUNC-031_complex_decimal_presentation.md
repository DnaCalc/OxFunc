# BUGREP-FUNC-031: COMPLEX and shared imaginary-family decimal presentation

Filed2026-09-29 against `11b23504fe8180d08397be5435bd215e1bbb9722` from the fresh black-box Excel campaign.
Status: `triaged`; canonical owner [BUG-FUNC-053](../streams/BUG-FUNC-053_complex_decimal_presentation.md), bead `oxf-mwue.28.5`.

Near-integer snapping, decimal digit selection and notation differ. A first candidate passed5544 targeted rows but a4375-row independent holdout exposed a repeated midpoint failure. Further280 informative probes distinguish extended-precision multiplication from division. All failures remain retained; no family exactness claim.

Reproduction and exact captures: [campaign evidence](../../function-lane/evidence/w111-broad-20260929/README.md).
This report records discovery and candidate validation, not a function-phase
completion claim. Scope and target remain partial; integration is partial.
