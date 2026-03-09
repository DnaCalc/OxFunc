# Text Function Empirical Expansion Notes

Status: `active-planning`
Scope: `TEXTJOIN`, `EXACT`, `CLEAN`

## 1. Purpose
Bridge the W7 string baseline into a function-focused empirical matrix for the W12 text family and the shared Excel text-coercion seam.

## 2. Current State
1. W7 already captures broad worksheet string behavior in:
   - `docs/function-lane/STRING_BEHAVIOR_RESEARCH_NOTES.md`
   - `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`
   - `tools/string-probe/*`
2. W12 now has runtime implementations for `TEXTJOIN`, `EXACT`, and `CLEAN`, but the current empirical pack is still thin for function-completion purposes.
3. The main open cross-function seam is Excel-grade textification:
   - number/logical to text rendering,
   - length-limit behavior,
   - blank versus empty-string behavior,
   - Unicode/control-character boundaries.

## 3. Official-Source Anchors
1. `TEXTJOIN`:
   - Microsoft Support says it accepts strings and ranges, converts numeric delimiters to text, and returns `#VALUE!` when the result exceeds `32767` characters.
   - Source: https://support.microsoft.com/en-gb/office/textjoin-function-357b449a-ec91-49d0-80c3-0e8fc845691c
2. `EXACT`:
   - Microsoft Support says it is case-sensitive and ignores formatting differences.
   - Source: https://support.microsoft.com/en-us/office/exact-function-d3087698-fc15-4a15-9631-12575cf29926
3. `CLEAN`:
   - Microsoft Support says it removes the first 32 non-printable 7-bit ASCII codes and does not remove certain additional Unicode/control characters.
   - Source: https://support.microsoft.com/en-us/office/clean-function-26f3a862-4c6c-4c75-ae39-4f8f6af7d3de

## 4. Differential-Comparison Target
1. Prefer paired worksheets with built-in and Ox XLL formulas side by side:
   - `TEXTJOIN(...)` versus `ox_TEXTJOIN(...)`
   - `EXACT(...)` versus `ox_EXACT(...)`
   - `CLEAN(...)` versus `ox_CLEAN(...)`
2. Include explicit observability cells for:
   - visible text
   - `LEN(...)`
   - `UNICODE(LEFT(...,1))` or `UNICODE(RIGHT(...,1))` where needed
   - `ERROR.TYPE(...)` for overflow/error lanes

## 5. Priority Matrix
1. `TEXTJOIN` coercion and flattening:
   - scalar text, number, logical, empty cell, and empty string inputs
   - range inputs
   - array constant inputs
   - spilled-array inputs
   - row-major flattening order
   - `ignore_empty = TRUE` versus `FALSE`
2. `TEXTJOIN` delimiter behavior:
   - text delimiter
   - empty delimiter
   - numeric delimiter
   - logical delimiter
3. `TEXTJOIN` limit behavior:
   - result length exactly `32767`
   - result length `32768`
   - long-result error observability through `ERROR.TYPE`
4. `EXACT` comparison lanes:
   - ASCII case difference
   - accent difference
   - composed versus decomposed Unicode
   - emoji/surrogate-pair equality
   - empty string versus true blank
   - number/logical-to-text coercion boundaries
5. `CLEAN` control-character lanes:
   - `CHAR(0..31)` spot checks
   - `CHAR(127)`
   - `CHAR(129)`, `141`, `143`, `144`, `157`
   - zero-width space
   - NBSP and tab interactions with `CLEAN` and `LEN`

## 6. Immediate Manifest Expansion Targets
1. Extend `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv` with function-specific rows for:
   - `TEXTJOIN` flattening and overflow
   - `EXACT` coercion and blank/empty-string distinctions
   - `CLEAN` additional control-character rows
2. Add W12/W15-specific side-by-side differential rows where built-in and `ox_` functions can be compared directly under the XLL seam.
3. Reuse W7 observability patterns (`LEN`, `UNICODE`, `ERROR.TYPE`) instead of inventing new probe-only conventions.

## 7. Open Semantic Risks
1. Current runtime textification is still not a proven Excel “General” formatting engine.
2. Text limit behavior needs direct function-family proof, not just general W7 string proof.
3. Unicode normalization/collation questions remain open and must stay explicit in contracts until empirically pinned.
