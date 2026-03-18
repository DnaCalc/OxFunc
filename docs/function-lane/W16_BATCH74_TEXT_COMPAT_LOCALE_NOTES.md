# W16 Batch 74 - Text Compat Locale Family

Status: `in_progress-self-contained`
Workset: `W16`
Evidence ID: `W16-BATCH74-TEXT-COMPAT-LOCALE-20260316`

## Scope
1. `ASC`
2. `DBCS`
3. `JIS`

## Shape
1. This batch is a bounded current-baseline width-conversion family over prepared text inputs.
2. `ASC` narrows full-width ASCII, full-width space, and current-baseline full-width katakana forms to half-width output.
3. `JIS` widens half-width ASCII, half-width space, and half-width katakana, including voiced and semi-voiced kana pairs, to full-width output.
4. `DBCS` currently shares the same widening kernel as `JIS` for the admitted slice.
5. Characters outside the admitted width-conversion set are preserved unchanged.

## Executable Coverage
1. Rust unit tests pin ASCII/full-width space roundtrips, katakana voiced-pair roundtrips, `DBCS == JIS` on the admitted slice, and prepared-text coercion of numbers, logicals, and blanks.
2. The Lean file is metadata-only and mirrors the bounded `FunctionMeta` shape for the family.

## Open Lanes
1. Broader locale-conditioned behavior outside the admitted width-conversion set is not claimed here.
2. Full empirical Excel characterization for rare kana marks and non-Japanese locale variance remains separate work.
3. Shared dispatch/catalog/root-import wiring remains out of scope for this owned-file pass.
