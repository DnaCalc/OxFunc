# W16 Batch 31 - Text Scalar Miscellaneous Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH31-TEXT-SCALAR-MISC-20260315`

## Scope
1. `CHAR`
2. `CODE`
3. `LOWER`
4. `UPPER`
5. `TRIM`
6. `REPT`

## Native Excel Baseline
Probe artifact:
1. `.tmp/w16-batch31-text-scalar-misc-probe.csv`

Pinned lanes:
1. `CHAR(65.9) -> "A"`
2. `CHAR(0) -> #VALUE!`
3. `CODE("AB") -> 65`
4. `CODE("") -> #VALUE!`
5. `LOWER(TRUE) -> "true"`
6. `UPPER(TRUE) -> "TRUE"`
7. `TRIM(" A   B ") -> "A B"`
8. `TRIM(CHAR(160)&"A"&CHAR(160)) -> CHAR(160)&"A"&CHAR(160)`
9. `REPT("ab",2.9) -> "abab"`
10. `REPT("a",32768) -> #VALUE!`
11. `REPT("a",-1) -> #VALUE!`
12. `REPT(TRUE,2) -> "TRUETRUE"`

## Current Implementation Notes
1. `CHAR`, `CODE`, `LOWER`, `UPPER`, `TRIM`, and `REPT` all use the ordinary values-only preparation seam.
2. `CHAR` truncates toward zero before range validation and admits only `1..255`.
3. `CODE` reads only the first UTF-16 code unit and rejects empty text with `#VALUE!`.
4. `TRIM` collapses ASCII space (`32`) runs but leaves non-breaking space (`160`) untouched.
5. `REPT` truncates `number_times` toward zero, rejects negatives with `#VALUE!`, and enforces Excel's `32767` UTF-16-unit output ceiling.
