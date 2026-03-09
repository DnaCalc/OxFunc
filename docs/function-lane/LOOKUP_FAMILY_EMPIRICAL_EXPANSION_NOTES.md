# Lookup Family Empirical Expansion Notes

Status: `active-planning`
Scope: `MATCH`, `XMATCH`, `XLOOKUP`

## 1. Purpose
Turn the current W6/W10 lookup scaffolding into a broader empirical matrix that can drive full Excel-parity work rather than isolated seed rows.

## 2. Current State
1. Built-in Excel replay already exists:
   - `docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S2_SCENARIO_MANIFEST_SEED.csv`
   - `docs/function-lane/W10_S4_SCENARIO_MANIFEST_SEED.csv`
   - `tools/xmatch-probe/*`
   - `tools/w10-probe/*`
2. Current replay is primarily built-in Excel baseline capture, not broad same-sheet differential verification against `ox_XMATCH`, `ox_MATCH`, and `ox_XLOOKUP`.
3. XLL seam evidence remains separate from function-semantic closure and must not be used as a substitute for built-in differential comparison.

## 3. Official-Source Anchors
1. `MATCH`:
   - Microsoft Support says the default `match_type` is `1`, text matching is case-insensitive, and wildcards are documented for `match_type = 0`.
   - Source: https://support.microsoft.com/en-us/office/match-function-e8dffd45-c762-47d6-bf89-533f4a37673a
2. `XMATCH`:
   - Microsoft Support says default `match_mode` is exact, wildcard mode is `2`, and binary search modes require sorted input and otherwise return invalid results.
   - Source: https://support.microsoft.com/en-us/office/xmatch-function-d966da31-7a6b-4a13-a1c6-5a33ed6a0312
3. `XLOOKUP`:
   - Microsoft Support documents `if_not_found`, wildcard and binary modes, 2D row/column return examples, and range-return composition examples.
   - Source: https://support.microsoft.com/en-us/office/xlookup-function-b7fd680e-6d10-43e6-84f9-88eae8bf5929

## 4. Differential-Comparison Target
1. For semantic verification, prefer worksheets that place built-in and Ox XLL formulas side by side on identical inputs.
2. Minimal pattern:
   - built-in formula cell
   - `ox_...` formula cell
   - comparison cell (`=EXACT(...)`, `=`, `ISNA`, `ERROR.TYPE`, `CELL("address", ...)`, or range-composition sentinel)
3. For `XLOOKUP`, include both value equality and reference-identity observability:
   - `CELL("address", XLOOKUP(...))`
   - `CELL("address", ox_XLOOKUP(...))`
   - `SUM(XLOOKUP(...):XLOOKUP(...))`
   - `SUM(ox_XLOOKUP(...):ox_XLOOKUP(...))`

## 5. Priority Matrix
1. Wildcards and escaping:
   - `MATCH("a*",{"abc","ade"},0)`
   - `XMATCH("a~*",{"abc","a*"},2)`
   - `XMATCH("a~~b",{"a~b","ab"},2)`
   - `XLOOKUP("a?",{"ab","a?"},{10,20},,2)`
2. Duplicate handling with search direction:
   - `XMATCH(2,{2,1,2},0,1)`
   - `XMATCH(2,{2,1,2},0,-1)`
   - `XLOOKUP(2,{1,2,2},B1:D1,,0,-1)`
3. Approximate modes:
   - exact hit plus gap-below plus gap-above for ascending and descending cases
   - include `MATCH(...,1)`, `MATCH(...,-1)`, `XMATCH(...,1)`, `XMATCH(...,-1)`, and `XLOOKUP(...,,,1,1)` / `XLOOKUP(...,,,-1,1)` style rows
4. Binary modes:
   - sorted ascending and sorted descending rows for exact and approximate cases
   - unsorted rows recorded as empirical “invalid results,” not assumed stable semantics
5. Blank and empty-string lanes:
   - lookup value blank
   - lookup array contains true empty cells
   - lookup array contains `""`
   - direct omitted optional args versus explicit empty-string fallback
6. Cross-type and error lanes:
   - text needle vs numeric haystack
   - logical lanes
   - embedded error values in lookup arrays
   - embedded error or blank values in return arrays
7. Collation lanes:
   - case difference
   - accent difference
   - composed vs decomposed Unicode
8. Shape and reference lanes:
   - `XLOOKUP` vertical lookup with 2D return matrix
   - `XLOOKUP` horizontal lookup with 2D return matrix
   - reference-return identity and range composition
   - true 2D lookup-array rejection or other observed behavior for `MATCH`/`XMATCH`

## 6. Immediate Manifest Expansion Targets
1. Expand `docs/function-lane/XMATCH_SCENARIO_MANIFEST_SEED.csv` for:
   - wildcard escaping
   - duplicate/reverse rows
   - binary unsorted rows
   - richer blank/collation rows
2. Expand `docs/function-lane/W10_S2_SCENARIO_MANIFEST_SEED.csv` for:
   - wildcard `MATCH`
   - approximate descending rows
   - blank/cross-type rows
3. Expand `docs/function-lane/W10_S4_SCENARIO_MANIFEST_SEED.csv` for:
   - 2D return-array rows
   - built-in vs Ox side-by-side comparisons
   - explicit `if_not_found` omission versus provided fallback rows

## 7. Open Semantic Risks
1. Current runtime still needs broader collation and cross-type parity.
2. Binary-mode semantics need more empirical coverage, especially unsorted and duplicate cases.
3. Reference-return behavior needs broader differential proof through both built-in and Ox XLL entrypoints.
