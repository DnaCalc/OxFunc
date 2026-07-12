# Welinder Gnumeric Blog — Excel-Behavior Mining Digest

Mined 2026-07-13 from https://blogs.gnome.org/mortenw/category/gnumeric/ (12 relevant of 22 posts). Primary-source reverse-engineering of Excel's exact behavior by Gnumeric maintainer Morten Welinder. See memory [[welinder-gnumeric-blog]].

## Top actionable takeaways

1. LOG10/LOG bit-exactness hinges on the exact scaling constant and multiply-vs-divide: probe whether Excel uses log(x)*0.4342944819032518276511289 (multiply by 1/ln10) vs division — but re-derive direct-vs-inverse per x87 80-bit precision, since Welinder's double-based ranking may not hold for Excel's transcendental chain (the 9.4e-17 vs 2.5e-17 gap is exactly the ±1-ULP class MEMORY forbids accepting).
2. COUNTIF/SUMIF do WHOLE-CELL text matching (/^pat$/) while DCOUNT/DSUM do PREFIX/begins-with matching (/^pat/) — encode anchoring as a per-family FunctionSpec value-fact; a unified criteria code path gets this wrong.
3. SUM has a provenance-dependent string-coercion asymmetry: text read through a cell reference is IGNORED, but an inline literal string arg is coerced to a number and included (non-parseable inline string = Excel error, not 0). Generalize to AVERAGE/COUNT/PRODUCT.
4. Criterion '=' matches BLANK cells, not empty strings; criteria are a mini-language (strip leading > < >= <= <> =, coerce number/date/boolean-looking values to typed equality, support * and ? wildcards) — pin locale/number-parse to avoid the German-decimal and float-round-trip divergences.
5. Excel's TEXT engine fails its own spec on ~12 cases (mostly avoidable overflows in fixed-denominator fraction formats like 0/128); bit-exactness means reproducing Excel's bugs, so treat live Excel output as canonical, never the documented format spec.
6. Booleans coerce to 1/0 in arithmetic/direct args but are a distinct type for type-checking (and ignored inside ranges by SUM/AVERAGE) — the standard leaves this undefined, so Excel's exact rule must be reverse-engineered and carried as a FunctionSpec value-fact; the value model needs error/empty/array as first-class kinds.
7. Build an ssdiff-style cell-by-cell diff with a neon-highlight mode as the discrepancy harness, and for every closed divergence add a CLASS-level guard that scans all functions — not just a single fixed catalog row.
8. Round-trip / internal-consistency tests prove NOTHING about Excel-conformance (you could swap multiply and divide and still round-trip) — only differential comparison against real Excel, plus mpmath/GSL high-precision oracles for constant-representation error, is valid.
9. VLOOKUP/HLOOKUP/MATCH text comparison is case-insensitive and collation-based (case-fold before collate) — verify OxFunc's approximate-match ordering matches Excel's collation, not raw byte order.
10. Implement the number-format section model (pos;neg;zero;text, empty section = blank, text passthrough) and its tokenization ambiguities (';' as fill char after '*', [Blue] color, [$-40b] hex-LCID localized month names, quoted literals) if/when TEXT and cell formatting come into scope — the highest-risk regression-prone subsystem.

## Posts worth a full human read

- Spreadsheet Function Semantics — Welinder on COUNTIF/SUMIF vs DCOUNT/DSUM criteria matching
- Floating-Point Accuracy For Scaling Numbers (Welinder, 2016)
- OpenDocument for Spreadsheets (Morten Welinder, 2005)
- Formatting Numbers (Morten Welinder, gnome.org blog, 2007-02-20)

# Welinder / Gnumeric Blog — Excel-Behavior Digest for OxFunc ("Make Excel Explicit")

Synthesized from 12 relevant posts on Morten Welinder's Gnumeric blog. Every specific Excel-behavior fact is preserved, grouped by topic, each tagged with its source post. Welinder built Gnumeric's function/format engines by reverse-engineering Excel and cross-checking against high-precision oracles (GSL) — the same posture OxFunc takes, so his concrete findings are directly reusable.

---

## 1. Floating-Point & Accuracy

**Source: "Floating-Point Accuracy For Scaling Numbers" (2016)** — https://blogs.gnome.org/mortenw/2016/03/11/floating-point-accuracy-for-scaling-numbers/

- Two mathematically identical formulas can differ in the last bits purely from how a scaling constant is stored. Example: `log10(x) = log(x)/log(10)` vs `log(x) * (1/log(10))` produce different last bits.
- `log(10)` is near a worst case for `double`: its true value sits just above the midpoint between the two nearest representable doubles → relative representation error ≈ **9.4e-17** (near the max ~½ ULP).
- The reciprocal `1/log(10) = 0.4342944819032518276511289` has representation error only ≈ **2.5e-17** — about ¼ of the error of storing `log(10)` directly.
- Therefore compute `log10` by **multiplying by a precomputed reciprocal constant** rather than dividing. Verbatim code: `static double l10i = 0.4342944819032518276511289; return log(x) * l10i;` Stated payoff: "several extra correct bits."
- Per-constant direct-vs-inverse ranking (which representation is more accurate in `double`):
  - **Better stored DIRECTLY** (divide by, or use as-is): `pi`, `EulerGamma` (Euler–Mascheroni γ), `log10(2)`.
  - **Better stored as the INVERSE** (multiply by `1/c`): `e`, `log(2)`, `log(10)`, `sqrt(5)`, `sqrt(pi)`, `sqrt(2*pi)`.
  - **Essentially a TIE**: `sqrt(2)`, `sqrt(3)`, and any integer or half-integer power of two.
- These rankings are **precision-dependent**: they change for `float` or `long double`. A constant better inverted in `double` may not be in `long double` — so the optimal representation must be chosen per target precision (load-bearing for Excel's x87 80-bit path).
- The argument is purely about the constant's representation error (where the true value lands vs the double midpoint), independent of the accuracy of `log(x)` itself.
- Discovery method: found by using **GSL's complex log10 test values as a reference oracle** against Gnumeric's implementation.

**Source: "OOXML vs ODF" (2007)** — https://blogs.gnome.org/mortenw/2007/09/11/ooxml-vs-odf/

- Rounding error is an **engine/implementation issue, not a file-format issue**. OpenOffice Calc "suffers from exactly the same rounding issue that Excel does. How could it be any different when both are based on floating point numbers?"
- The file format stores the numeric result as an IEEE double but does **not** define the computation that produced it. Neither OOXML nor ODF pins down bit-exact arithmetic. → Bit-exactness is inherently a calc-engine reverse-engineering task, orthogonal to format.

---

## 2. Number Formatting

**Source: "Formatting Numbers" (2007)** — https://blogs.gnome.org/mortenw/2007/02/20/formatting-numbers/

- A number format maps a `(value, format)` pair to a display string; the value can be number, text, or boolean (`3.14`, `"xyz"`, `TRUE`). **The same format-string engine backs Excel's `TEXT()` function** — so `TEXT()` is the formatting oracle.
- Format strings are split into sections by semicolons `;`, role is **positional**:
  - section1 = non-negative/positive value
  - section2 = negative value
  - section3 = zero
  - section4 = text
- An **empty section** (nothing between two semicolons) means **display nothing** for that value class.
- **Text values with no explicit text section are passed through "as-is"** (unchanged).
- `*c` is a **fill/repeat operator**: repeats character `c` to fill the remaining cell width. Real parsing ambiguity: in `*;` the semicolon after `*` is a **fill character, not a section separator**.
- Color is a bracketed name per section, e.g. `[Blue]`.
- Bracketed locale override `[$-40b]`: hex `40b` is the Windows LCID for **Finnish**, forcing month/date name rendering in that language while the rest uses the current UI language.
- Literal text is embedded via a **quoted string** inside the format.
- Date field codes: `dd` = two-digit day, `mmmm` = full month name, `yyyy` = four-digit year.
- **Fixed-denominator fraction display**: `0/128` = render as the nearest 128th, as a fraction.
- Full worked example: `dd-mmmm-yyyy[$-40b]/dd-mmmm-yyyy[Whitestone"76"]*;;0/128[Blue]` — render a date twice (current language + Finnish), fill leftover width on the right with semicolons, all white; negatives in blue as nearest 128th **without the minus sign**; non-numbers left as-is.
- Format-string parsing rules are "really complicated and very much undocumented," with inconsistent documentation across sources.
- **Excel's own `TEXT` has bugs**: on Welinder's TEXT test workbook Excel scored **Pass 594 / Fail 12**, and "most of the failures are avoidable overflows in fraction formats" (fixed-denominator forms like `0/128`).
- Excel "can be wrong even though it is nominally defining the semantics" — Excel is the de-facto spec yet doesn't always follow it, so bit-exact replication must **copy Excel's mistakes**, not the nominal rule.
- Comparative TEXT-workbook scores (diagnostic): Gnumeric 606/0; Excel 594/12; OpenOffice.org 221/69788 (OOo's huge count partly from unsupported array formulas in the harness).

**Source: "Code Quality, Part II" (2010)** — https://blogs.gnome.org/mortenw/2010/09/29/code-quality-part-ii/

- Number rendering (the format-string code that renders values to displayed strings) is a "hairy piece of code," historically scary to modify because there was "basically no way of making sure no new errors were introduced" without automated tests → number-formatting is a high-risk, regression-prone subsystem demanding exhaustive coverage.

---

## 3. Function Semantics & Value Coercion

### Criteria / aggregate functions (COUNTIF/SUMIF vs DCOUNT/DSUM)

**Source: "Spreadsheet Function Semantics" (2016)** — https://blogs.gnome.org/mortenw/2016/06/22/spreadsheet-function-semantics/

- Excel has **TWO distinct criteria families with DIFFERENT matching semantics**: the database "D" functions (DCOUNT, DSUM, DAVERAGE, …) and the "IF" functions (COUNTIF, SUMIF, …). The same criterion string can mean different things in each.
- **Wildcard/pattern anchoring differs**. For criterion `foo*bar`:
  - D-functions match `/^foo.*bar/` (grep) — **anchored at START only = prefix / begins-with**; trailing cell content is allowed.
  - IF-functions match `/^foo.*bar$/` — **anchored at BOTH ends = whole-cell match**.
  - Verbatim: "For the 'D' functions it means `/^foo.*bar/` in grep terms, whereas for the 'if' functions it means `/^foo.*bar$/`."
- A criterion of `=` does **not** search for empty strings — it searches for **blank (empty) cells**. Verbatim: "'=' does not mean to look for empty strings. Instead it means to look for blank cells."
- Criteria parseable as numbers, dates, or booleans are treated as **typed-value equality (type coercion)**, not literal-string comparison.
- Criteria are strings embedded in the formula (`12`, `">0"`, `"<=12.5"`, `"=Oink"`, `"Foo*bar"`). Leading comparison operators (`>`, `<`, `>=`, `<=`, `<>`, `=`) are parsed out and applied as typed comparisons.
- `*` and `?` are wildcards in both families (with the family-specific anchoring above).
- **Locale hazard**: numbers written into a criterion string and parsed back depend on locale (decimal separator) — "mail the spreadsheet to Germany and get different results" — and can cause floating-point precision loss when a number round-trips through its string form inside the criterion.
- LibreOffice Calc does **not** match Excel: it fails on D-function anchoring, DCOUNT strictness, wildcards generally, boolean handling (localc has no booleans), and the array-formula case.

### SUM / aggregate string coercion asymmetry

**Source: "OpenDocument for Spreadsheets" (2005)** — https://blogs.gnome.org/mortenw/2005/06/16/opendocument-for-spreadsheets/

- **Load-bearing quirk**: Excel `SUM()` **ignores** string literals like `"42"` living in **referenced cells** (they contribute 0), but when the same string is passed as a **direct/inline argument** to `SUM()`, Excel **coerces it to a number and includes it**. Same text value, opposite treatment based on provenance (reference vs inline literal).
- Contrast engines on the identical case: OpenOffice-calc **errors** on the direct-string case; Gnumeric **ignores all strings** for SUM (both provenances). Three engines, three rules; Excel's = coerce-direct-string / ignore-referenced-string.
- (Implied) A non-numeric-parseable direct string argument should be an **Excel error**, not silently 0.
- OpenOffice-calc coerces `"1"+1` to boolean **TRUE** (not `2`) — a type-coercion quirk flagged as a contrast.
- Excel's value model needs **error values, `empty`/blank, and arrays** as first-class types, alongside number/date/time/boolean/string — coercion and aggregation rules branch on value kind.

### Boolean type ambiguity

**Source: "ODF Plus Ten Years" (2015)** — https://blogs.gnome.org/mortenw/2015/04/17/odf-plus-ten-years/

- OpenFormula leaves it an **implementation choice** whether logical TRUE/FALSE are numbers or a distinct type: "it allows a choice whether logical values are numbers or their own distinct type." Boolean coercion in arithmetic is **not pinned by the standard** and must be reverse-engineered from Excel (Excel: booleans coerce to 1/0 in arithmetic and direct args, but are a distinct type for type-checking / ignored-as-distinct inside ranges by SUM/AVERAGE).
- ODF "strict" conformance **cannot represent error values** (#N/A, #DIV/0!, #VALUE!) — error values are behaviors, not just display.

### Lookup / reference functions

**Source: "Common Subexpressions" (2005)** — https://blogs.gnome.org/mortenw/2005/08/02/common-subexpressions/

- VLOOKUP/HLOOKUP string matching goes through a **collation comparison** (Gnumeric: `g_utf8_collate`), not raw byte comparison → locale/collation-aware ordering and **case-insensitive** matching (case-fold before collate). Relevant to approximate-match (`range_lookup=TRUE`) ordering and MATCH.
- Comparison is character-by-character and short-circuits at the first differing character (only initial chars usually needed).
- `INDIRECT` converts a string result into a cell reference at eval time — "the single most ugly feature of spreadsheet semantics." `INDEX` is the cleaner alternative ("most uses of INDIRECT … would be far better handled as INDEX calls"). (This is OxFml/OxCalc reference-resolution scope, not OxFunc value-eval.)

---

## 4. Dates / Serial Numbers

**Source: "Formatting Numbers" (2007)** — https://blogs.gnome.org/mortenw/2007/02/20/formatting-numbers/

- Dates render through the number-format engine: field codes `dd` (two-digit day), `mmmm` (full month name), `yyyy` (four-digit year).
- Month-name **localization is per-format** via bracketed LCID overrides, e.g. `[$-40b]` = Finnish month names, independent of the UI language used for the rest of the string. A single format can emit the same date twice in two languages.

*(No post in this corpus covers the 1900/1904 epoch, the Feb-29-1900 leap-year bug, or serial-number arithmetic directly — those remain to be mined elsewhere.)*

---

## 5. Recalc & Dependencies

**Source: "Common Subexpressions" (2005)** — https://blogs.gnome.org/mortenw/2005/08/02/common-subexpressions/

- Title is a false friend: "common subexpressions" is used in the **performance** sense (redundant repeated work), NOT the numerical sense. **No** discussion of IEEE 754, x87 extended precision, rounding, or CSE altering floating-point results.
- A sheet with many VLOOKUP/HLOOKUP calls against the same table recomputes the table's collation/sort keys **once per lookup call with no memoization** — a redundant-recompute performance issue.
- Durable design idea: a from-scratch eval engine would compile expressions to bytecode with common subexpressions explicitly eliminated (recalc/eval-engine architecture, not bit-exact results).

**Source: "ODF Plus Ten Years" (2015)** — https://blogs.gnome.org/mortenw/2015/04/17/odf-plus-ten-years/

- ODF has **no shared formulas**: identical formulas across a range are stored as separate strings and re-parsed per cell — "tens of thousands of times is common." (OOXML/xlsx **does** have shared-formula representation.)

**Source: "Spreadsheets and the Command Line" (2013) / "Testing is not an Option!" (2006)**

- `ssconvert` forcing evaluation of **all** cells between import and export is used as a **full-recalc smoke test** — round-trip conversion exercises the whole calc core plus importers/exporters in one pass.

---

## 6. File Formats (OOXML / ODS)

**Source: "ODF Plus Ten Years" (2015)** — https://blogs.gnome.org/mortenw/2015/04/17/odf-plus-ten-years/

- ODF "strict" (the schema-backed variant) **cannot represent error values**; no producer uses strict — everyone uses "extended" (strict + arbitrary tags/attributes).
- ODF has **no shared formulas** (re-parse per cell).
- ODF does **not store sheet dimensions/extent** → "two different spreadsheets that compute completely different things but save to identical ODF files"; used-range must be inferred.
- For 10 years "no-one in the ODS world has been performing even basic document validation" — real files often fail schema validation; handle what implementations emit, not what the standard says.
- "Extended" ODF permits arbitrary tags → LibreOffice/OpenOffice and Gnumeric invented **incompatible extensions for the same features**; a feature can have mutually-unreadable serializations.
- OpenFormula arrived "5–10 years too late"; base ODF was designed without spreadsheets in mind → boolean-type ambiguity and formula-model gaps.

**Source: "Writing Tests is Humbling" (2014)** — https://blogs.gnome.org/mortenw/2014/03/11/writing-tests-is-humbling/

- xls/biff8 (Excel 97-2003): fixed max sheet size **65536 rows × 256 columns**.
- xls/biff7 (Excel 95): **cannot store arbitrary Unicode strings** (codepage-limited).
- xlsx (OOXML): **cannot store solver parameters** (model state lost on save).
- ods: cannot store a **patterned cell background** or the **sheet size**.
- "Excel and LibreOffice are what really defines xls/xlsx and ods formats" — the application, not the spec, is authoritative.
- Real round-trip bugs found: diagonal cell-border direction flipped on load+save; a hang writing certain strings to xls/biff7; regressions introduced while fixing other bugs; truncated strings surfaced by loading in LibreOffice.

**Source: "ODF Plus Five Years" (2010)** — https://blogs.gnome.org/mortenw/2010/02/10/odf-plus-five-years/

- ODF spreadsheet behavior is de facto defined by "whatever OpenOffice happens to implement," not the spec. "Just like XLS is whatever Excel says it is."
- OpenDocument (~2005 and still 2010) **omitted formula syntax AND semantics** entirely.
- OpenOffice changed its ODF formula syntax at least once, **breaking Gnumeric's import** (GNOME bug 570890); it introduced a **new formula namespace** → correct import must be namespace/version-aware. No stable spec to conform to. (Text documents had no such problems — the gap is spreadsheet-specific.)

**Source: "OOXML vs ODF" (2007)** — https://blogs.gnome.org/mortenw/2007/09/11/ooxml-vs-odf/

- ODF 1.1 did not specify a formula grammar: "there still seems to [be] no syntax for 2+2." ODF then "doesn't actually have non-trivial formulas" → backward compatibility with legacy binary spreadsheets is an open problem.

**Source: "OpenDocument for Spreadsheets" (2005)** — https://blogs.gnome.org/mortenw/2005/06/16/opendocument-for-spreadsheets/

- ODF v1.0 cell references are wrapped in **square brackets with a leading dot** (`[.A1]`); argument separator is the **semicolon** `;`, not a comma. Example: `=sum([.A1:.A5])` ≡ `=sum([.A1];[.A2];[.A3];[.A4];[.A5])`.
- ODF v1.0 documents formula behavior on only ~2 examples (pages 184–186), both `sum()` — "sum is the only function we get to know about." Two conforming implementations couldn't reliably read each other's spreadsheets. "Probably easier to interoperate via the xls format."

---

## 7. Testing Methodology

**Source: "Spreadsheet Function Semantics" (2016)**

- Method for discovering the criteria-family differences: a dedicated test spreadsheet that exercises the criteria functions and uses an **array formula to COUNT the number of failing cases** in the sheet. Extend the test file to find more.

**Source: "Formatting Numbers" (2007)**

- Use `TEXT()` as the **oracle for the whole number-formatting engine**: one workbook of `TEXT(value, format)` cases with expected strings scores any engine pass/fail (Gnumeric 606/0, Excel 594/12, OOo 221/69788). Treat live Excel output — including its ~12 fraction-format failures — as canonical, not the documented spec.

**Source: "Floating-Point Accuracy For Scaling Numbers" (2016)**

- Use an **independent high-precision library (GSL / mpmath) as an oracle** to expose a target implementation's constant-representation error (reusable differential-testing pattern).

**Source: "Writing Tests is Humbling" (2014)**

- A **round-trip / self-consistency test does NOT prove correctness** — "you could swap multiplication and division and still get a perfect round-trip." Correctness is only validated against the real Excel/LibreOffice application. A passing internal suite proves nothing about Excel-conformance.

**Source: "Code Quality, Part II" (2010)**

- When a bug is found, don't just fix the instance — **write a test that mechanically scans ALL functions for the whole class** ("write a test that checks all the function help texts for this kind of error"). Each closed divergence should leave a class-level guard.
- Stress-test file-format ingestion by **fuzzing that keeps containers syntactically valid** (garble XML/ZIP contents while preserving valid XML and valid ZIP) so the parser is exercised, not rejected at the envelope.
- Layer runtime checkers (Valgrind/Purify/glib memory checker) and static analyzers (gcc -Wall, clang analyzer, Coverity, sparse) into the **regular automated** suite. Caveat: clang/Coverity "have pretty high false report rates," and "we still let mistakes through."
- Per-function argument metadata (arity/named args) drifts out of sync easily (help texts referred to nonexistent args, missed args).

**Source: "Spreadsheets and the Command Line" (2013)**

- Gnumeric CLI tools as a testing-harness template: `ssconvert` (format conversion / merge / PDF), `ssgrep` (grep inside spreadsheet files), `ssindex` (text extraction), and especially **`ssdiff`** — cell-by-cell spreadsheet diff with three output modes: plain-text diff, XML diff, and **a copy of one input with differing cells highlighted in neon yellow**. This is the exact shape of harness for surfacing Excel-vs-OxFunc discrepancies at scale (analogous to the discrepancy catalog).

**Source: "Testing is not an Option!" (2006)**

- `ssconvert` forces evaluation of every cell between import and export → a **full-recalc smoke test** that hits the calc core plus importers/exporters in one pass. Wired into `make distcheck` so testing is mandatory/automatic (a 1.7.3 regression that broke evaluation forced an emergency 1.7.4 release; automation was the response). Add valgrind + importer tests to the automated suite.

---

## 8. Excel Quirks & Known Bugs (cross-cutting)

- **Excel `TEXT` fails its own spec** on ~12 cases, mostly "avoidable overflows in fraction formats" (fixed-denominator forms like `0/128`). Bit-exact means reproducing these bugs. *("Formatting Numbers", 2007)*
- **Format-string tokenization ambiguities**: `;` as a fill char after `*` vs a section separator; empty section = blank output; text passthrough when no text section. *("Formatting Numbers", 2007)*
- **COUNTIF/SUMIF do whole-cell matching; DCOUNT/DSUM do prefix (begins-with) matching** — a unified code path gets this wrong. *("Spreadsheet Function Semantics", 2016)*
- **Criterion `=` matches blank cells, not empty strings.** *("Spreadsheet Function Semantics", 2016)*
- **SUM coerces inline string args but ignores referenced strings** — provenance-dependent coercion. *("OpenDocument for Spreadsheets", 2005)*
- **Booleans coerce to 1/0 but are a distinct type for type-checking** — the standard leaves this undefined; Excel's rule must be reverse-engineered. *("ODF Plus Ten Years", 2015)*
- **VLOOKUP/HLOOKUP/MATCH text compare is case-insensitive and collation-based.** *("Common Subexpressions", 2005)*
- **Excel is the de-facto spec yet is sometimes wrong** — replicate its behavior, not the documented rule. *(multiple: "Formatting Numbers" 2007, "Writing Tests is Humbling" 2014, "ODF Plus Five Years" 2010)*

---

## Actionable for OxFunc

### Bit-exact numerics
- **LOG10/LOG family**: test whether Excel realizes `log10` as `log(x) * (1/ln10)` (multiply by reciprocal) vs a division, and pin the exact constant. Concrete probe: `0.4342944819032518276511289` applied by multiply. Because Excel's transcendental chain is x87 **80-bit** (per MEMORY: `excel_numeric::x87`), the direct-vs-inverse ranking from Welinder's `double` analysis may **not** hold — re-derive per-precision. The 9.4e-17 vs 2.5e-17 representation-error gap is exactly the ±1-ULP class MEMORY forbids accepting as divergence.
- **General scaling-constant table** (from the post) is a starting hypothesis for reverse-engineering any OxFunc kernel with a constant: multiply-by-inverse likely for `e, log(2), log(10), sqrt(5), sqrt(pi), sqrt(2π)`; use-directly likely for `pi, EulerGamma, log10(2)`; tie for `sqrt(2), sqrt(3)`, powers of two. Verify each against an mpmath/GSL oracle at x87 precision.
- Confirmed framing: rounding is an **engine** property, not a format property — keep reverse-engineering the calc engine; cross-check against Gnumeric/OOcalc to separate generic-IEEE behavior from Excel-specific quirks.

### Criteria / aggregate functions (COUNTIF/SUMIF/…, DCOUNT/DSUM/…)
- Encode anchoring as a **per-family FunctionSpec value-fact**: IF-family text criteria = **full-cell match** `/^pat$/`; D-family text criteria = **prefix match** `/^pat/` (begins-with). Do not unify the code path.
- Model criteria as a parsed mini-language: `=` matches **blank cells** (not empty strings); strip leading `> < >= <= <> =` and apply as typed comparisons; coerce number/date/boolean-looking criteria to **typed equality**; support `*` and `?` wildcards in both families with correct anchoring.
- **Pin the number-parse/locale** so criteria never diverge (avoid the German-decimal-separator and float-round-trip-precision hazards).

### Value model & coercion
- Implement the **SUM string-coercion asymmetry**: referenced text → ignored (skip/0); inline literal string → coerce to number and include; non-parseable inline string → Excel error (not 0). Generalize the provenance rule to AVERAGE/COUNT/PRODUCT.
- Nail down Excel's **boolean coercion** explicitly per function/context (1/0 in arithmetic and direct args; distinct type / ignored inside ranges by SUM/AVERAGE).
- Carry **error values, empty/blank, and arrays** as first-class value kinds — coercion and aggregation branch on value kind.
- Ensure VLOOKUP/HLOOKUP/MATCH text matching is **case-insensitive + Excel collation ordering** for approximate-match sorting. (INDIRECT vs INDEX is OxFml/OxCalc scope.)

### TEXT() / number formatting (when in scope)
- Implement the section model (pos;neg;zero;text; empty section = blank; text passthrough when no text section); bracketed tokens (`[Blue]`, `[$-40b]` hex LCID for localized months, quoted literals); `*c` fill including the `*;` fill-vs-separator ambiguity; fixed-denominator fraction formats (`0/128`).
- **Deliberately reproduce Excel's fraction-format overflow bug** (its 12 TEXT failures) — bit-exact to live Excel, not analytically correct. Treat number-formatting as the highest-risk subsystem; build exhaustive coverage before touching it.

### Testing methodology (mission-level)
- Use `TEXT()` workbooks and criteria-function sheets with an **array formula that counts failing cases** as differential oracles; treat any COUNTIF-vs-DCOUNT anchoring difference as a bit-exact requirement, per MEMORY's never-accept-divergence rule.
- Use **mpmath/GSL high-precision oracles** to expose constant-representation errors (feeds the existing x87-emulation-in-Python tooling from MEMORY).
- Build an **`ssdiff`-style cell-by-cell diff with a highlight mode** to surface Excel-vs-OxFunc discrepancies at scale — the exact shape of the discrepancy catalog (`docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md`).
- For each closed divergence, leave a **class-level guard** (scan all functions for the whole bug class), not just a single fixed row.
- Remember: **round-trip/internal-consistency tests prove nothing about Excel-conformance** — only differential comparison against real Excel does. This validates OxFunc's Excel-is-the-oracle posture.
