# W16 Batch 61 - Text Delimiter Functions

Status: `packet-evidenced`
Workset: `W16`

## Scope
1. `TEXTAFTER`
2. `TEXTBEFORE`

## Current Implementation Notes
1. This family now participates in shared dispatch, export-spec, and root Lean import surfaces; the earlier self-contained wording is obsolete.
2. `TEXTAFTER` and `TEXTBEFORE` are modeled on the ordinary values-only preparation seam with `2..6` argument admission.
3. The current runtime covers the primary scalar baseline:
   - positive and negative `instance_num`
   - ASCII-only case-insensitive `match_mode = 1`
   - `match_end = 1` as a synthetic end-of-text delimiter
   - documented empty-delimiter polarity
   - explicit `if_not_found` fallback
4. `instance_num = 0` and invalid `match_mode` / `match_end` flags map to `#VALUE!`; unresolved no-match maps to `#N/A`.
5. `W24` Batch 03 supplied the native worksheet packet and exposed a real adapter-order bug, now corrected so the optional arguments follow Excel's order `(...,[instance_num],[match_mode],[match_end],[if_not_found])`.

## Seeded Unit Lanes
1. `TEXTAFTER("One,Two,Three", ",", 1) -> "Two,Three"`
2. `TEXTAFTER("One,Two,Three", ",", -1) -> "Three"`
3. `TEXTBEFORE("One,Two,Three", ",", 1) -> "One"`
4. `TEXTBEFORE("One,Two,Three", ",", -1) -> "One,Two"`
5. `TEXTAFTER("Socrates", " ", 1, "fallback", 0, 1) -> ""`
6. `TEXTBEFORE("Socrates", " ", 1, "fallback", 0, 1) -> "Socrates"`
7. `TEXTAFTER("abc", "/", 1, 7) -> 7`
