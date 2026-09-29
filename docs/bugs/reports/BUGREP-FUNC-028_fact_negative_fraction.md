# BUGREP-FUNC-028: FACT negative fractions admitted before truncation

Filed2026-09-29 against `11b23504fe8180d08397be5435bd215e1bbb9722` from the fresh black-box Excel campaign.
Status: `triaged`; canonical owner [BUG-FUNC-050](../streams/BUG-FUNC-050_fact_negative_fraction.md), bead `oxf-mwue.28.1`.

FACT(-0.1) returned 1; live Excel returns #NUM!. Sign admission must precede truncation. Discovery1209 and independent heldout1509 agree after the candidate repair. The unchanged multiplication kernel and its remaining type/reference axes still require current-phase assessment.

Reproduction and exact captures: [campaign evidence](../../function-lane/evidence/w111-broad-20260929/README.md).
This report records discovery and candidate validation, not a function-phase
completion claim. Scope and target remain partial; integration is partial.
