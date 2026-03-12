# String Normalization and Comparison Policy Map

Status: `provisional`
Workset: `W7`
Evidence ID: `W7-STR-BL-20260305`

Scope note:
1. policies below are baseline-scoped to Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, compatibility descriptors captured in `STRING_EXECUTION_RECORD.md`, locale `en-US`.

## 1. Equality Semantics
1. `=` is case-insensitive for tested ASCII pair (`"A"="a"` -> `TRUE`).
2. `EXACT` is case-sensitive (`EXACT("A","a")` -> `FALSE`).
3. `EXACT` textifies numeric/logical scalars and treats blank cells as empty text in the current function-level follow-up.
4. `EXACT` remains code-unit-sensitive in tested Unicode lanes: precomposed and combining forms remained distinct, while identical emoji strings remained equal.
5. Tested accent, punctuation, and edge-space differences remained unequal under `=`.
4. `FIND` and `SEARCH` diverge as expected:
   - `FIND` case-sensitive (error on `"a"` in `"A"`),
   - `SEARCH` case-insensitive (`1` on same pair).

Evidence:
1. `STR8-001..STR8-007`
2. `STR8-017..STR8-018`

## 2. Ordering/Collation (Initial)
1. Tested ordering examples indicate case-insensitive ordering relation behavior in this baseline (`"a"<"B"` -> `TRUE`, `"A">"a"` -> `FALSE`).
2. This is not a full collation closure and should not be generalized beyond baseline until expanded matrix runs.

Evidence:
1. `STR8-025..STR8-026`

## 3. Whitespace and Non-Printable Normalization
1. `TRIM` normalized repeated ASCII spaces (`" A   B "` -> `"A B"`).
2. `TRIM` did not remove NBSP (`U+00A0`) or tab (`U+0009`) in tested rows.
3. `CLEAN` removed low control char (`CHAR(1)`), but did not remove `CHAR(127)` in tested row.
4. W12 follow-up showed `CLEAN` also removes `CHAR(129)`, `141`, `143`, `144`, and `157`.
5. Zero-width space (`U+200B`) survived tested `TRIM` and `CLEAN` paths.

Evidence:
1. `STR8-008..STR8-011`
2. `STR8-033..STR8-036`

## 4. Length Limits and Overflow/Error Behavior
1. Formula-generated text at `32767` characters is admitted (`LEN(REPT("x",32767))=32767`).
2. Formula-generated text above cap yields `#VALUE!` sentinel (`-2146826273`) in tested rows.
3. COM formula-literal admission (`Range.Formula`/`Formula2`) for large inline text showed an observed boundary at `4095` literal chars admitted, `4096+` rejected on set with `0x800A03EC` in this baseline.

Evidence:
1. `STR8-019..STR8-024`
2. Error mapping: `FORMULA_ADMISSION_BEHAVIOR_NOTES.md`

## 5. Unicode Handling
1. `LEN(UNICHAR(128512))=2` in this baseline scope.
2. `UNICODE(UNICHAR(128512))=128512`.
3. Combining sequence (`"e" + U+0301`) length is `2`, distinct from precomposed `U+00E9`.

Evidence:
1. `STR8-012..STR8-016`
2. `STR8-031..STR8-032`

## 6. Boundary Differences
1. Formula overflow path:
   - text over cap errors (`#VALUE!`).
2. Interop set path (`Range.Value2`):
   - over-cap strings are accepted and truncated to `32767` UTF-16 code units.
3. Interop emoji overflow edge:
   - truncation can leave a dangling high-surrogate tail (`UNICODE(RIGHT(A1,1)) -> #VALUE!`, `LEN(RIGHT(A1,1))=1`).
4. Reference reuse path:
   - tested truncation result remained stable through direct references.
5. Persistence path:
   - tested XLSX save/reopen and CSV roundtrip preserved observed values for selected rows.

Evidence:
1. `STR8-027..STR8-038`
2. `STR8-039..STR8-044`
3. `STR8-045..STR8-046`

## 7. Integration Targets
1. W3 value-universe string subtype and boundary tags:
   - treat text as capped UTF-16 code-unit sequence at worksheet/interop boundaries.
2. W4 coercion primitives:
   - coercion functions must not assume canonical Unicode normalization.
3. W6 (`XMATCH`) obligations:
   - matching semantics must explicitly choose between case-insensitive compare behavior and strict compare paths.
