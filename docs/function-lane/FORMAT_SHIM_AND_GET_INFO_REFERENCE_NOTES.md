# Format Shim And GET.INFO Reference Notes

Status: `active`
Owner lane: `OxFunc`
Relationship: concrete local test seam for locale/format-sensitive functions and reference note for legacy `GET.*` info/macro surfaces

## 1. Purpose
This note turns the W13 locale/format design pressure into a concrete local testing plan.

The immediate goal is not a full Excel formatting engine. It is a small deterministic shim that OxFunc can call through an explicit seam while we continue empirical comparison against a real local Excel host.

This note also records the current `GET.WORKSPACE` / `GET.CELL` reference leads from the legacy Excel 4 macro corpus and the Excel-DNA `GetInfoFunctions` sample, because those sources are directly relevant to:
1. `VALUE`
2. `TEXT`
3. `DOLLAR`
4. `FIXED`
5. later `CELL` / `GET.*` / displayed-text / configured-format work

## 2. Recommended Small Shim
Use Option 1 in a deliberately small, table-driven form.

Candidate OxFunc-local test context:

```text
TestLocaleFormatContext {
  profile_id: FormatProfileId,
  date_system: WorkbookDateSystem,
  parser: TestLocaleValueParser,
  formatter: TestFormatCodeEngine,
}
```

Candidate facilities:

```text
TestLocaleValueParser
TestFormatCodeEngine
```

This is intentionally not the future full OxFml/FEC formatting world. It is a narrow local shim with deterministic admitted behavior so OxFunc functions can be closed honestly while the larger formatting substrate is still being designed.

## 3. Initial Supported Profiles
The first two profiles should be:
1. `en-US`
2. `current_excel_host`

Reason:
1. `en-US` gives a stable reference profile for tests and docs.
2. `current_excel_host` lets us compare shimmed OxFunc behavior through the XLL path against the actual Excel installation on this machine.

## 4. Current Excel Host Profile
The current host profile should be grounded in empirical `GET.WORKSPACE(37)` observations from the local Excel instance.

Observed `INDEX(GET.WORKSPACE(37), n)` values on `2026-03-13`:
1. `1 -> 1`
2. `2 -> 27`
3. `3 -> .` decimal separator
4. `4 -> <space>` thousands separator
5. `5 -> ,` list separator
6. `6 -> R`
7. `7 -> C`
8. `8 -> r`
9. `9 -> c`
10. `10 -> [`
11. `11 -> ]`
12. `12 -> {`
13. `13 -> }`
14. `14 -> ,`
15. `15 -> ;`
16. `16 -> @`
17. `17 -> /` date separator
18. `18 -> :` time separator
19. `19 -> y`
20. `20 -> m`
21. `21 -> d`
22. `22 -> h`
23. `23 -> m`
24. `24 -> s`
25. `25 -> R` currency symbol
26. `26 -> General`
27. `27 -> 2` currency decimals
28. `28 -> 1` negative currency format
29. `29 -> 2` negative number format

Direct worksheet spot-checks from the same host:
1. `TEXT(0.5,"0%") -> "50%"`
2. `DOLLAR(1234.567,2) -> "R1 234.57"`
3. `FIXED(1234.567,2) -> "1 234.57"`
4. `TEXT(DATE(2024,2,3),"yyyy-mm-dd") -> "2024-02-03"`

## 5. Initial Shim Scope
The first admitted parser/render slice should stay deliberately small.

### 5.1 Parser slice
Support only:
1. plain numeric text
2. grouped numeric text
3. percent text
4. simple currency text
5. one pinned date-text lane per profile where empirically grounded

### 5.2 Renderer slice
Support only:
1. `0`
2. `0.00`
3. `0%`
4. `yyyy-mm-dd`
5. `DOLLAR(value, decimals)`
6. `FIXED(value, decimals, no_commas)`

Everything else should fail explicitly as unsupported in the shim rather than silently pretending to be Excel-complete.

## 6. Why This Shim Is Useful
This shim gives us:
1. a concrete OxFunc-facing seam for locale/format-sensitive functions
2. deterministic tests for admitted lanes
3. continued grounding against the actual Excel host on this machine
4. a bridge into later OxFml/FEC ownership without forcing locale/render logic into per-function kernels

It also keeps the right honesty boundary:
1. OxFunc still owns function semantics and which functions require parse/render facilities.
2. OxFunc still owns array-lift policy where applicable.
3. the shim only stands in for a host-provided parse/render service, not for function semantics themselves.

## 7. Recommended Repo Artifacts
Small next-step artifacts for W13 and adjacent work:
1. a machine-readable `FORMAT_PROFILE_SEED.csv` for `en-US` and `current_excel_host`
2. a small `FORMAT_RENDER_SCENARIO_MANIFEST_SEED.csv`
3. a small `VALUE_PARSE_SCENARIO_MANIFEST_SEED.csv`
4. a Rust test shim implementing the admitted profile rows above
5. XLL comparison rows restricted to the admitted slice

## 8. Macro-Help Source Locations
Relevant local source corpus:
1. `C:\Work\ExcelDna\MacroHelp\Excel 4 Macro Reference.docx`
2. `C:\Work\ExcelDna\MacroHelp\Excel 4 Macro Reference.pdf`
3. `C:\Work\ExcelDna\MacroHelp\XLMACR8.chm`
4. `C:\Work\Excel-DNA\Samples\GetInfoFunctions\GetInfoAddIn.cs`
5. `C:\Work\Excel-DNA\Samples\GetInfoFunctions\GetInfoSample.xls`

The `.docx` copy is immediately extractable and is sufficient for pulling the `GET.*` tables into our working references.

## 9. Extracted GET.WORKSPACE Notes
From the legacy macro-help corpus, `GET.WORKSPACE(37)` is the key entry for format/profile-sensitive host settings.

The extracted table includes slots for:
1. country and version info
2. decimal and thousands separators
3. list separator
4. R1C1 symbols
5. array constant delimiters
6. date/time separators
7. date/time format-symbol letters
8. currency symbol and default currency-decimal settings
9. `General` format name
10. negative-currency and negative-number format selections

This is a strong bridge between:
1. local Excel host configuration
2. the future OxFml/FEC formatting profile world
3. function semantics that depend on parsing, rendering, or displayed text

## 10. Extracted GET.CELL Notes
From the macro-help corpus, `GET.CELL(type_num, reference)` is explicitly defined as returning information about the formatting, location, or contents of a cell.

Extracted type-number highlights:
1. `1` absolute reference text
2. `2` row number
3. `3` column number
4. `4` same as `TYPE(reference)`
5. `5` contents
6. `6` formula text in current A1/R1C1 mode
7. `7` number format text
8. `8` horizontal alignment code
9. `9..12` border-style codes
10. `13` pattern code
11. `14` locked flag
12. `15` hidden-formula flag
13. `16` width-info array
14. `17` row height
15. `18` font name
16. `19` font size
17. `20` bold status

This reinforces that formatting, display, and cell-info semantics should be treated as first-class reference material for later `CELL` and `GET.*` work, not as afterthoughts.

## 11. Excel-DNA Sample Relevance
The Excel-DNA sample at `C:\Work\Excel-DNA\Samples\GetInfoFunctions` exposes wrappers around:
1. `xlfGetCell`
2. `xlfGetDocument`
3. `xlfGetWorkbook`
4. `xlfGetWorkspace`

It also includes a simple `GetListSeparator` example using `GET.WORKSPACE(37)` item `5`.

This sample is useful for:
1. quick empirical workbook experiments without writing new native XLL glue
2. validating how legacy `GET.*` surfaces appear from worksheet formulas
3. building later probes for displayed text, format codes, and cell metadata

## 12. Immediate Doctrine Implication
The small formatting shim should not try to own Excel's full formatting language.

Instead:
1. OxFunc should declare when a function depends on parser help, formatter help, or both.
2. the local shim should cover only the empirically pinned subset.
3. later OxFml/FEC work should own the broader format-language and locale-profile substrate.
4. `CELL` and `GET.*` should be treated as adjacent information/format-observation seams, not as isolated oddities.

## 13. Tester XLL GET.INFO Wrappers
The Rust tester XLL now exposes worksheet-callable wrappers for selected legacy information functions:
1. `ox_GET_CELL`
2. `ox_GET_DOCUMENT`
3. `ox_GET_WORKBOOK`
4. `ox_GET_WORKBOOK_ACTIVE`
5. `ox_GET_WORKSPACE`

These wrappers are implemented in `tools/xll-addin/oxfunc_xll/src/lib.rs` and exercised through `docs/function-lane/XLL_GET_INFO_SCENARIO_MANIFEST_SEED.csv` plus `tools/xll-addin/run-get-info-probe.ps1`.


Update 2026-03-14:
1. The tester-XLL GET.* wrappers required true macro-type registration via the # suffix in 	ype_text; macro_type = 1 on its own was not sufficient for parity.
2. The GET.CELL probe runner also had to translate manifest A1 references into Excel4 Sheet!R1C1 references before ExecuteExcel4Macro(...) would accept them.

