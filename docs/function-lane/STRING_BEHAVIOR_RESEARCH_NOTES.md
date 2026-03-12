# String Behavior Research Notes (Excel)

Status: `baseline-complete-provisional`
Workset: `W7`
Evidence ID: `W7-STR-BL-20260305`

## 1. Purpose
Capture source-backed claims and empirical findings for Excel string behavior characterization.

This note is the W7 source+evidence ledger feeding `FDEF-032` and W3/W6 dependency updates.

## 2. Baseline Scope
Execution date:
1. `2026-03-05`

Observed environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Compatibility descriptors observed in run rows:
   - `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`
   - `default|CalculationVersion=0|CheckCompatibility=False|FileFormat=6` (CSV reopen lane)
4. Locale profile: `en-US`

## 3. Source Ledger (Spec + Wide Search)
`STR-A/STR-B` claim IDs used in this baseline:

1. `STR-SRC-001`:
   - claim: worksheet cell text limit is `32,767` characters and formula limit is `8,192` characters.
   - source: Microsoft Support, Excel specifications and limits.
2. `STR-SRC-002`:
   - claim: `EXACT` is case-sensitive string comparison.
   - source: Microsoft Support, EXACT function.
3. `STR-SRC-003`:
   - claim: `FIND` is case-sensitive; `SEARCH` is case-insensitive.
   - source: Microsoft Support, FIND and SEARCH function pages.
4. `STR-SRC-004`:
   - claim: `TRIM` removes ASCII spaces but not nonbreaking space (`CHAR(160)`).
   - source: Microsoft Support, TRIM function.
5. `STR-SRC-005`:
   - claim: `CLEAN` removes the first 32 non-printable 7-bit ASCII codes and may require `SUBSTITUTE` for others.
   - source: Microsoft Support, CLEAN function.
6. `STR-SRC-006`:
   - claim: `LEN` behavior depends on compatibility mode for surrogate pairs (Compatibility Version 2 note).
   - source: Microsoft Support, LEN function.
7. `STR-SRC-007`:
   - claim: `UNICODE` returns the code point of the first character.
   - source: Microsoft Support, UNICODE function.
8. `STR-SRC-008`:
   - claim: `UNICHAR` returns a Unicode character for a numeric code point.
   - source: Microsoft Support, UNICHAR function.
9. `STR-SRC-009`:
   - claim: `REPT` returns `#VALUE!` when the resulting text exceeds `32,767` characters.
   - source: Microsoft Support, REPT function.

## 4. Empirical Findings (W7 Baseline)
Empirical runner:
1. `tools/string-probe/run-string-excel-baseline.ps1`

Manifest:
1. `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv` (`STR8-001..STR8-046`)

Key observed outcomes:
1. Equality/comparison:
   - `="A"="a"` -> `TRUE` (`STR8-001`)
   - `EXACT("A","a")` -> `FALSE` (`STR8-002`)
   - accent-sensitive and punctuation-sensitive for tested pairs (`STR8-003..STR8-007`).
   - W12 follow-up also pinned `EXACT(1,"1") -> TRUE`, `EXACT(TRUE,"TRUE") -> TRUE`, `EXACT("",blank_cell) -> TRUE`, `EXACT(UNICHAR(233),"e"&UNICHAR(769)) -> FALSE`, and `EXACT(UNICHAR(128512),UNICHAR(128512)) -> TRUE` under the same local Excel baseline.
2. Whitespace/non-printable:
   - `TRIM(" A   B ")` -> `"A B"` (`STR8-008`).
   - `TRIM` did not remove NBSP or tab in tested rows (`STR8-034`, `STR8-035`).
   - `CLEAN(CHAR(1)&"A")` removed `CHAR(1)`; `CLEAN(CHAR(127)&"A")` did not remove `CHAR(127)` (`STR8-010`, `STR8-011`).
   - W12 follow-up showed `CLEAN` also removes the extra C1 subset `CHAR(129)`, `141`, `143`, `144`, and `157`, while still preserving zero-width space and NBSP in the tested rows.
   - zero-width space survived `TRIM` and `CLEAN` in tested case (`STR8-036`).
3. Unicode granularity:
   - `LEN(UNICHAR(128512))` -> `2` in this baseline (`STR8-012`).
   - `UNICODE(UNICHAR(128512))` -> `128512` (`STR8-013`).
   - precomposed vs combining sequence remained distinct (`STR8-016`, `STR8-031`, `STR8-032`).
4. Limits and overflow:
   - `LEN(REPT("x",32767))` -> `32767` (`STR8-019`).
   - `LEN(REPT("x",32768))` and `LEN(REPT("x",40000))` -> Excel error sentinel `-2146826273` (`#VALUE!`) (`STR8-020`, `STR8-021`).
5. Interop ingress (`Range.Value2` set):
   - 32,768 and 40,000 ASCII characters were accepted and truncated to `32767` without set-time exception (`STR8-028`, `STR8-029`).
   - 40,000 emoji assignment also truncated to `32767` UTF-16 code units (`STR8-030`, `STR8-045`).
   - emoji-overflow tail showed dangling surrogate behavior in this baseline: `UNICODE(RIGHT(A1,1))` -> `#VALUE!`, `LEN(RIGHT(A1,1))` -> `1` (`STR8-045`), while ASCII control case stayed valid (`STR8-046`).
6. Formula admission boundary through COM formula properties:
   - literal formula payload of `4095` characters admitted (`STR8-022`).
   - `4096+` failed on set with `0x800A03EC` (`STR8-023`, `STR8-024`).
7. Boundary persistence:
   - reference reuse preserved truncated length (`STR8-037`, `STR8-038`).
   - XLSX save/reopen preserved tested long-string states (`STR8-039..STR8-041`).
   - CSV roundtrip preserved tested quote/comma/newline/NBSP rows (`STR8-042..STR8-044`).

Error sentinel mapping used:
1. COM integer `-2146826273` corresponds to Excel `#VALUE!` (`ERROR.TYPE=3`), documented in `FORMULA_ADMISSION_BEHAVIOR_NOTES.md`.

## 5. Integration Promotion Candidates
1. `FDEF-032` promotion to `provisional` for build/channel/compat scope above.
2. W3 string subtype update:
   - text cap modeled at `32767` UTF-16 code units,
   - interop ingress may yield dangling-surrogate tail states.
3. W6 precondition update:
   - matching/collation work should assume `=`/`EXACT` split and non-trivial whitespace normalization behavior.
4. W12 text-function update:
   - `EXACT` and `CLEAN` now have function-level follow-up evidence beyond W7’s general string matrix, especially around blank/text coercion and the observed `CLEAN` C1-subset removal behavior.

## 6. Residual Risks and Deferred Expansion
1. Multi-build/channel replay not yet done.
2. Non-default compatibility-version workbook replay not yet done.
3. Non-`en-US` locale replay not yet done.
4. Ordering/collation beyond small probe set remains open.

## 7. Sources
1. https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
2. https://support.microsoft.com/en-au/office/exact-function-d3087698-fc15-4a15-9631-12575cf29926
3. https://support.microsoft.com/en-us/office/find-function-c7912941-af2a-4bdf-a553-d0d89b0a0628
4. https://support.microsoft.com/en-au/office/search-function-9ab04538-0e55-4719-a72e-b6f54513b495
5. https://support.microsoft.com/en-au/office/trim-function-410388fa-c5df-49c6-b16c-9e5630b479f9
6. https://support.microsoft.com/en-us/office/clean-function-26f3a862-4c6c-4c75-ae39-4f8f6af7d3de
7. https://support.microsoft.com/en-au/office/len-function-29236f94-cedc-429d-affd-b5e33d2c67cb
8. https://support.microsoft.com/en-au/office/unicode-function-adb74aaa-a2a5-4dde-aff6-966e4e81f16f
9. https://support.microsoft.com/en-au/office/unichar-function-ffeb64f5-f131-44c6-b332-5cd72f0659b8
10. https://support.microsoft.com/en-gb/office/rept-function-04c4d778-e712-43b4-9c15-d656582bb061
