# W28 Function Name Localization Discovery - Preliminary Note

Status: `complete-for-declared-discovery-scope`
Owner lane: `OxFunc`

## 1. Purpose
Record the first reproducible discovery facts for harvesting official localized Excel function names from Microsoft support.

## 2. Current Best Discovery Rule
Use the canonical English support article as the starting point:
1. `https://support.microsoft.com/en-us/office/excel-functions-alphabetical-b3944572-255d-4efb-bb96-c6d90033e188`

Then discover the localized equivalents from that page's published `hreflang` alternates.

Working implication:
1. do not try to derive the right localized function-list page from locale codes alone,
2. instead treat the article GUID plus the page's own alternate links as the authoritative discovery route for this support surface.

## 3. Why This Looks Reliable
The current HTML exposes:
1. a canonical link back to the English article,
2. an explicit `awa-articleGuid` / asset id for the article GUID,
3. explicit `hreflang` links for localized variants.

That gives a stable tuple:
1. article GUID,
2. locale code,
3. localized article URL.

## 4. Current Harvest Findings
### 4.1 Locale coverage
The current article advertises `40` localized alternates through `hreflang`, and the `W28` runner harvested all `40` locale pages.

Seed artifact:
1. `docs/function-lane/W28_SUPPORT_FUNCTION_LOCALIZATION_LOCALE_SEED.csv`
2. `.tmp/w28-support-function-localization-locales.csv`

### 4.2 Function-list extraction
The current English article body now harvests as `509` unique function-name rows on the current baseline, and the full multilingual harvest produced `20,360` localized rows in `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv`.

Important caution:
1. this is a current support-surface count, not by itself the canonical OxFunc catalog truth,
2. the current support page contains at least one visible naming anomaly: `BETA.INVn`,
3. the harvest-vs-catalog reconciliation now also shows `IMAGINARY` absent from the current support page while the older catalog still carried the old local typo `RANDARRA`,
4. later passes must normalize duplicates, compatibility aliases, and support-surface anomalies before using the count as a program-level truth.

### 4.3 Version markers
The article includes an explicit note that version markers indicate the version of Excel in which a function was introduced.

Current implication:
1. localized-name harvesting should be paired with marker harvesting where practical,
2. this can become a useful input to the OxFunc library-context snapshot later.

## 5. Relationship To Open Specifications Locale Lists
The LCID / ST_LangCode note at:
1. `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/6c085406-a698-4e12-9d4d-c3b0ee3dbc4a`

is useful as a language-code normalization reference, but it is not sufficient by itself to discover the right support article URLs.

Current best reading:
1. use Microsoft support `hreflang` links as the authoritative article-discovery source,
2. use Open Specifications locale references as secondary normalization/reference material.

## 5A. Required Later Spec Pass
This packet should later include a deliberate pass through:
1. `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376`

for the function declarations relevant to the harvested list.

Minimum expectation for that later pass:
1. match the harvested support-surface inventory against the normative function declarations,
2. add comments where the spec declares normative variations, compatibility forms, or version-scoped differences,
3. record mismatches between support-page naming/version notes and the normative declaration surface rather than silently choosing one source,
4. feed those comments back into the future function-name localization library and catalog notes.

## 6. Expected Future Artifact Shape
This discovery work should lead to a library artifact that can eventually carry at least:
1. canonical OxFunc function id,
2. canonical English worksheet name,
3. compatibility aliases,
4. localized worksheet names by locale,
5. source article GUID and localized support URL,
6. optional version-marker metadata,
7. evidence timestamp / extraction provenance.

Current seed artifact:
1. `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv`

## 7. Current Open Questions
1. how much of the support-page body can be harvested deterministically without brittle HTML scraping,
2. whether individual per-function articles expose their own localized-name/marker surfaces more reliably than the alphabetical index alone,
3. how localized names should be normalized when multiple English-region pages share the same worksheet function names,
4. how this library should intersect with dynamic user-defined function registrations later.

## 8. Post-Harvest Empirical Resolution
Direct Excel probing on `2026-03-19` resolved the immediate anomaly set:
1. `BETA.INV` exists while `BETA.INVn` is a support-page typo.
2. `IMAGINARY` exists.
3. `FORECAST` and `FORECAST.LINEAR` both exist.
4. The missing `IS*` functions (`ISBLANK`, `ISERR`, `ISERROR`, `ISLOGICAL`, `ISNA`, `ISNONTEXT`, `ISNUMBER`, `ISODD`, `ISREF`, `ISTEXT`) are all real.
5. OxFunc now carries a corrected local current-baseline catalog in `FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv` with `511` names.
