# W28 Execution Record - Function Name Localization Library Discovery

Status: `complete`
Workset: `W28`

## 1. Purpose
Record the closure of the official-support discovery and initial harvest packet for the Excel function-name localization library.

## 2. Executed Packet
Artifacts created or updated:
1. `tools/w28-probe/run-w28-support-function-localization-harvest.ps1`
2. `tools/w28-probe/run-w28-function-name-resolution-probe.ps1`
3. `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv`
4. `docs/function-lane/W28_SUPPORT_FUNCTION_CATALOG_RECONCILIATION.csv`
5. `docs/function-lane/W28_FUNCTION_NAME_EXISTENCE_PROBE_RESULTS.csv`
6. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
7. `.tmp/w28-support-function-localization-locales.csv`
8. `.tmp/w28-support-function-localization-summary.json`
9. `docs/worksets/W028_FUNCTION_NAME_LOCALIZATION_LIBRARY_DISCOVERY.md`
10. `docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_DISCOVERY_PRELIM.md`
11. `docs/function-lane/W28_SUPPORT_FUNCTION_LOCALIZATION_LOCALE_SEED.csv`
12. `docs/function-lane/W28_EXECUTION_RECORD.md`

## 3. Empirical Discovery Result
1. The canonical Microsoft support page still resolves through article GUID `b3944572-255d-4efb-bb96-c6d90033e188`.
2. The current page publishes `40` localized alternates through `hreflang`.
3. The harvest captured `20,360` localized function-name rows across those locale pages.
4. The current English page yielded `509` unique function-name rows on the present baseline.
5. The older Foundation catalog still stands at `500` named rows, so the support surface is no longer a simple one-to-one match to that older freeze.

## 4. Direct Excel Resolution
From `docs/function-lane/W28_FUNCTION_NAME_EXISTENCE_PROBE_RESULTS.csv`:
1. `BETA.INV` exists on the installed Excel baseline.
2. `BETA.INVn` does not exist and returns `#NAME?`, so it is treated as a support-page typo rather than a real function.
3. `IMAGINARY` exists on the installed Excel baseline.
4. `FORECAST` exists on the installed Excel baseline.
5. `FORECAST.LINEAR` also exists on the installed Excel baseline, even though the support harvest only yielded `FORECAST` as the visible article-name row.
6. `ISBLANK`, `ISERR`, `ISERROR`, `ISLOGICAL`, `ISNA`, `ISNONTEXT`, `ISNUMBER`, `ISODD`, `ISREF`, and `ISTEXT` all exist on the installed Excel baseline.

## 5. Reconciliation Findings
From `docs/function-lane/W28_SUPPORT_FUNCTION_CATALOG_RECONCILIATION.csv`:
1. `497` names are present in both the current support harvest and the older Foundation catalog.
2. `12` names are support-only relative to that older catalog.
3. `3` names are Foundation-only relative to the current support harvest.
4. Notable current-support anomalies include:
   - `BETA.INVn` as a support-page spelling anomaly on the current English page,
   - the old local/Foundation typo `RANDARRA`,
   - `IMAGINARY` currently absent from the harvested support page.
5. Those anomalies are now resolved empirically:
   - keep `BETA.INV`,
   - keep `IMAGINARY`,
   - drop `BETA.INVn`,
   - drop `RANDARRA`,
   - add `FORECAST`, `FORECAST.LINEAR`, and the ten missing `IS*` functions to the OxFunc-local current-baseline catalog.
6. The corrected OxFunc-local current-baseline catalog now contains `511` names in `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`.

## 6. Scope Result
1. `W28` is now complete for its declared discovery-and-library-seed scope.
2. It does not claim final multilingual parse/bind integration or a fully stabilized library-context carrier.
3. The future normative pass against `MS-OE376` remains a separate follow-on obligation, already recorded in the packet note.

## 7. Verification Run
1. `powershell -ExecutionPolicy Bypass -File tools/w28-probe/run-w28-support-function-localization-harvest.ps1`
2. `powershell -ExecutionPolicy Bypass -File tools/w28-probe/run-w28-function-name-resolution-probe.ps1`

## 8. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W28` scope
