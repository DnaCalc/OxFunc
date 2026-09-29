# BUGREP-FUNC-032: BASE and engineering radix bounds, widths and typed arguments

Filed2026-09-29 against `11b23504fe8180d08397be5435bd215e1bbb9722` from the fresh black-box Excel campaign.
Status: `triaged`; canonical owner [BUG-FUNC-054](../streams/BUG-FUNC-054_radix_bounds_and_types.md), bead `oxf-mwue.28.2`.

BASE sign and2^53 limits and zero/optional-width behavior differ; engineering widths must validate1..10 even for negative encodings. Typed probes additionally establish logical rejection, empty text zero, exact whitespace grammar, omitted width defaults and width-first errors. All3249 admissible typed discovery rows now agree;12 malformed no-argument formulas are withheld. Fresh heldouts remain open.

Reproduction and exact captures: [campaign evidence](../../function-lane/evidence/w111-broad-20260929/README.md).
This report records discovery and candidate validation, not a function-phase
completion claim. Scope and target remain partial; integration is partial.
