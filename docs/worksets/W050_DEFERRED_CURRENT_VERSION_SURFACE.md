# WORKSET - Deferred Current-Version Surface (W50)

## 1. Purpose
Centralize the Excel function rows that are explicitly deferred from the current OxFunc completion target.

This packet exists to stop older family packets from doubling as the active deferred-scope tracker.

## 2. Provenance
This packet consolidates deferred-current-version ownership from:
1. `W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md`
2. `W025_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_OUTLIERS.md`
3. `W036_DEFERRED_PROVIDER_LANGUAGE_CAPABILITY_BASELINE.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`

Current total:
1. `17` function rows.
2. `0` operator rows.

Members:
1. `COPILOT`
2. `CUBEKPIMEMBER`
3. `CUBEMEMBER`
4. `CUBEMEMBERPROPERTY`
5. `CUBERANKEDMEMBER`
6. `CUBESET`
7. `CUBESETCOUNT`
8. `CUBEVALUE`
9. `DETECTLANGUAGE`
10. `ENCODEURL`
11. `EUROCONVERT`
12. `FILTERXML`
13. `GETPIVOTDATA`
14. `PHONETIC`
15. `STOCKHISTORY`
16. `TRANSLATE`
17. `WEBSERVICE`

## 4. Current-Version Rule
For the current version target:
1. all `W041` family members are treated as deferred,
2. the extracted `W036` `TRANSLATE` provider-language seam is also treated as deferred from the current completion target,
3. `EUROCONVERT` is also deferred,
4. no other function or operator row should be treated as deferred unless this packet is updated explicitly.

## 5. Ownership Rule
1. `W50` is the canonical current-version deferred list.
2. The older family packets remain provenance/evidence owners for family-specific work and prior classification.
3. Changes to deferred scope should be reflected here first, then back-propagated to planning summaries.

## 6. Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes: none for the list-definition task
