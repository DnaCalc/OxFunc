# BUGREP-FUNC-029: Bitwise integer admission, shift graph and omitted arguments

Filed2026-09-29 against `11b23504fe8180d08397be5435bd215e1bbb9722` from the fresh black-box Excel campaign.
Status: `triaged`; canonical owner [BUG-FUNC-051](../streams/BUG-FUNC-051_bitwise_admission_shift.md), bead `oxf-mwue.28.3`.

Fractional operands were truncated; 48-bit overflow and shifts49..53 were handled differently from Excel; explicit omitted positions did not become zero. The repaired candidate matches34246 admitted numeric observations and862 typed observations, including independent heldouts. Input-ingress exclusions and candidate hashes are retained.

Reproduction and exact captures: [campaign evidence](../../function-lane/evidence/w111-broad-20260929/README.md).
This report records discovery and candidate validation, not a function-phase
completion claim. Scope and target remain partial; integration is partial.
