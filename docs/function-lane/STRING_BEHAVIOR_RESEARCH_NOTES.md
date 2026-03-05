# String Behavior Research Notes (Excel)

Status: `active-research`
Workset: `W7`

## 1. Purpose
Capture source-backed hypotheses and empirical TODO items for Excel string behavior characterization.

This note is not final policy; it is a working input for W7 execution.

## 2. Source Families (Planned)
1. Microsoft specification/support corpus for text functions and limits.
2. Foundation-linked conformance references relevant to formula evaluation and value behavior.
3. Wide-search evidence from public, reproducible sources (with source links and date capture).
4. Empirical workbook observations under explicit version/channel/compatibility metadata.

## 3. Target Questions
1. Are worksheet equality comparisons case-insensitive by default, and where does `EXACT` differ?
2. How are accents/diacritics and punctuation handled in equality and ordering contexts?
3. What are the exact practical and documented limits for cell text and formula-produced text?
4. How do non-printables (`CHAR(0..31)`, NBSP, zero-width) propagate through comparison and text functions?
5. What Unicode granularity does Excel expose (`code unit`, `code point`, surrogate behavior)?
6. Do formula-only, materialized cell, and referenced-value lanes diverge for strings?
7. What persistence/CSV round-trip normalization is observable?

## 4. Execution Lanes
1. `STR-A` spec extraction lane.
2. `STR-B` wide-search lane.
3. `STR-C` worksheet empirical comparison/coercion lane.
4. `STR-D` persistence/text round-trip lane.
5. `STR-E` interop boundary lane (if needed for unresolved cases).

## 5. Immediate TODO
1. Build source ledger with exact URLs, access dates, and extracted claim IDs.
2. Expand and validate `STRING_SCENARIO_MANIFEST_SEED.csv`.
3. Prepare baseline empirical runner output contract aligned with W2 methodology.
4. Define promotion candidates for stable findings and link them to conformance row `FDEF-032`.
