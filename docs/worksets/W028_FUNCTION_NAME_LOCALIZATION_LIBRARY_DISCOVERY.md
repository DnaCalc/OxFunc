# WORKSET - Function Name Localization Library Discovery (W28)

## 1. Purpose
Discover and characterize the official Microsoft support surfaces that expose Excel function names across localized languages, then use that discovery to seed an OxFunc function-name localization library.

This packet is discovery and inventory work first.
It does not claim a finished localization library in the opening pass.

## 2. Provenance
Opened after the current OxFml/OxFunc seam round made localized function-name tables an explicit library-context topic.

Primary upstream/public sources:
1. `https://support.microsoft.com/en-us/office/excel-functions-alphabetical-b3944572-255d-4efb-bb96-c6d90033e188`
2. localized alternates discovered from that page's `hreflang` links
3. `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/6c085406-a698-4e12-9d4d-c3b0ee3dbc4a`

Seed local artifacts:
1. `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_DISCOVERY_PRELIM.md`
2. `docs/function-lane/W28_SUPPORT_FUNCTION_LOCALIZATION_LOCALE_SEED.csv`
3. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`

## 3. Scope
1. discover the canonical Microsoft support article(s) that enumerate Excel functions,
2. determine how localized variants of those articles are discovered reliably,
3. capture the currently visible locale set and article identity strategy,
4. seed an OxFunc-local function-name localization library plan,
5. compare the harvested current function list against OxFunc's current catalog/classification state,
6. identify version-marker and compatibility-marker information available on the official list pages.

## 4. In Scope
1. article-guid and `hreflang` discovery work,
2. locale-url seed capture,
3. initial current-baseline function-count discovery,
4. planning for localized-name harvesting into a future library-context artifact,
5. documenting how support-page locale coverage does or does not align with Office locale-code references,
6. planning a later pass through `MS-OE376` to annotate normative declared function variations against the harvested support-page inventory and OxFunc's current catalog,
7. resolving the current W28 anomaly set against live Excel and promoting a corrected OxFunc-local current-baseline function catalog.

## 5. Out of Scope
1. claiming a finished cross-locale function-name library in this opening pass,
2. locking the final OxFml/OxFunc library-context carrier for localized names,
3. version-complete closure for every historical Excel function introduction marker,
4. implementation of runtime localized parse/bind against the harvested library.

## 6. Initial Findings
1. The current official English article exposes a stable article GUID:
   - `b3944572-255d-4efb-bb96-c6d90033e188`
2. The page itself publishes explicit localized alternates through `hreflang` links.
3. The right localized page should therefore be discovered from the canonical article and its alternates, not guessed from a locale-code list alone.
4. The current page exposes `40` localized alternates in its own HTML.
5. The current English article body exposes roughly `513` function links on the present baseline, though deduping and cleanup are still needed before treating that as a final canonical count.
6. The page explicitly says that version markers indicate when a function was introduced.
7. The Open Specifications locale note is useful as a normalization/reference source, but it is not by itself the authoritative list of support locales for this support article.

## 7. Deliverables
1. discovery note for the current support-surface strategy,
2. locale-seed artifact,
3. planned target shape for a function-name localization library,
4. reconciliation note against the current OxFunc catalog/classification inventory,
5. explicit future-pass note for `MS-OE376` normative variation matching,
6. a corrected OxFunc-local current-baseline catalog artifact derived from the older imported snapshot plus W28 empirical resolution.

## 8. Gate Criteria
This workset can only be reported `scope_complete` when:
1. the official page-discovery strategy is documented clearly,
2. the current locale seed is captured reproducibly,
3. the current function-list extraction route is documented clearly enough to replay,
4. the intended library artifact shape is stated,
5. OxFunc has an explicit note about how this discovery feeds future localized-name tables in the library-context seam.
6. the packet records that a later pass must reconcile the support-surface inventory against `MS-OE376` function declarations and add comments for normative variations where declared behavior or naming differs.

## 9. Execution Result
1. `W28` now has a reproducible harvest runner:
   - `tools/w28-probe/run-w28-support-function-localization-harvest.ps1`
2. The packet now carries:
   - `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv`
   - `docs/function-lane/W28_SUPPORT_FUNCTION_CATALOG_RECONCILIATION.csv`
   - `.tmp/w28-support-function-localization-summary.json`
3. Current harvested facts:
   - `40` locale pages,
   - `20,360` localized function-name rows,
   - `509` current English unique function-name rows,
   - `12` support-only names relative to the older `500`-row Foundation catalog,
   - `3` Foundation-only names relative to the current support page.
4. Notable anomalies are now explicit:
   - current support-page `BETA.INVn`,
   - old catalog/local typo `RANDARRA`,
   - `IMAGINARY` currently absent from the current support page harvest.
5. Direct Excel existence probing on `2026-03-19` resolved the anomaly set:
   - `BETA.INV` exists while `BETA.INVn` does not and is treated as a support-page typo,
   - `IMAGINARY` exists,
   - `FORECAST` exists,
   - `FORECAST.LINEAR` also exists even though it was not harvested as a separate support-page name row,
   - `ISBLANK`, `ISERR`, `ISERROR`, `ISLOGICAL`, `ISNA`, `ISNONTEXT`, `ISNUMBER`, `ISODD`, `ISREF`, and `ISTEXT` all exist.
6. OxFunc now carries a corrected local current-baseline catalog:
   - `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
7. That corrected local catalog currently contains `511` names:
   - the older imported `500` rows,
   - minus the false typo row `RANDARRA`,
   - plus `FORECAST`, `FORECAST.LINEAR`, and the ten missing `IS*` functions.

## 10. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W28` scope
