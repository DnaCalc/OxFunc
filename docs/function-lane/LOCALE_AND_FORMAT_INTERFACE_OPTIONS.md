# Locale And Format Interface Options

Status: `active`
Owner lane: `OxFunc`
Relationship: handoff/design note for OxFml and FEC/F3E

## 1. Purpose
Capture the design options for the locale/profile-sensitive parse and format substrate required by:
1. `VALUE`
2. `TEXT`
3. `DOLLAR`
4. `FIXED`
5. later date/time and textification-sensitive functions

This note exists because W13 confirmed that these functions cannot be closed honestly without an explicit locale/format seam.

## 2. Confirmed Pressure
Direct Excel probes already show:
1. `DOLLAR` and `FIXED` return locale-shaped text, including currency symbol and grouping conventions
2. `TEXT` depends on Excel's format-code language and renderer
3. `VALUE` depends on locale/profile-sensitive parsing for numeric, percent, currency, date, and time-like text

So OxFunc should not guess at locale/format behavior with ad hoc Rust helpers.

## 3. Non-Negotiable Requirements
Any acceptable design must:
1. make locale/profile dependence explicit at the function boundary
2. keep format-code language ownership out of per-function kernels
3. let OxFunc request parsing/formatting behavior declaratively
4. preserve traceability so empirical Excel differences can be recorded and reproduced
5. support both:
   - value parsing (`VALUE`)
   - value rendering (`TEXT`, `DOLLAR`, `FIXED`)

## 4. Option A - Fat Locale/Format Service Object
Provide OxFunc with one rich service object containing everything locale/format-sensitive.

Candidate shape:

```text
LocaleFormatServices {
  locale_profile: LocaleProfile
  date_system: WorkbookDateSystem
  parse_value_text(...)
  format_number_with_code(...)
  format_currency(...)
  format_fixed(...)
}
```

Pros:
1. simple for OxFunc callers
2. one obvious injection point for locale-sensitive functions
3. easy to extend when later functions need more services

Cons:
1. tends to become a grab-bag
2. risks weak ownership boundaries between OxFml/FEC and OxFunc
3. makes it easier for functions to depend on more than they declare

## 5. Option B - Split Parse And Render Facilities
Provide separate facilities for:
1. locale-sensitive text-to-value parsing
2. format-code compilation/rendering
3. workbook date-system and related profile data

Candidate shape:

```text
LocaleValueParser
FormatCodeEngine
FormatProfile
WorkbookDateSystem
```

Possible OxFunc-facing contract:

```text
struct LocaleFormatContext {
  profile: FormatProfile,
  date_system: WorkbookDateSystem,
  parser: &dyn LocaleValueParser,
  formatter: &dyn FormatCodeEngine,
}
```

Pros:
1. clean ownership split
2. better fit for declarative `fec_dependency_profile` and facility tags
3. lets `VALUE` depend on parsing without silently gaining rendering
4. lets `TEXT`/`DOLLAR`/`FIXED` depend on rendering without silently gaining parsing
5. scales well when later functions need only one side

Cons:
1. more interface design work up front
2. slightly more plumbing at the boundary

## 6. Option C - Opaque Host Callback Layer
Keep OxFunc almost entirely ignorant of locale/format rules and expose opaque host callbacks:

```text
parse_text_as_value(profile, text) -> ParsedValue
render_value_with_format(profile, value, code) -> RenderedText
render_currency(profile, value, decimals) -> RenderedText
render_fixed(profile, value, decimals, commas) -> RenderedText
```

Pros:
1. minimal immediate work inside OxFunc
2. easy to keep the heavy format language in OxFml/FEC land

Cons:
1. weakest formalization story
2. hard to align Lean with opaque callbacks
3. easy to lose semantic clarity and artifact traceability

## 7. Recommended Direction
Option B is the best current fit.

Reason:
1. it keeps parsing and rendering separate
2. it matches the declared-facility style already used in OxFunc
3. it gives OxFml/FEC a clear place to own:
   - locale token policy
   - date-system selection
   - format-code language and rendering
4. it keeps OxFunc functions small and honest:
   - `VALUE` asks for parser help
   - `TEXT` asks for format-code rendering
   - `DOLLAR` and `FIXED` ask for specialized rendering over the same profile

## 8. Suggested Boundary Vocabulary
Minimum candidate vocabulary:
1. `LocaleProfile`
2. `WorkbookDateSystem`
3. `LocaleValueParser`
4. `FormatCodeEngine`
5. `FormatProfile`
6. `RenderedText`
7. `ParsedValue`

Candidate OxFunc-facing trait split:

```text
trait LocaleValueParser {
  parse_value_text(profile: &LocaleProfile, date_system: WorkbookDateSystem, text: &str) -> Result<ParsedValue, ParseFailure>;
}

trait FormatCodeEngine {
  render_with_code(profile: &FormatProfile, value: FormatValue, code: &str) -> Result<RenderedText, FormatFailure>;
  render_currency(profile: &FormatProfile, value: f64, decimals: i32) -> Result<RenderedText, FormatFailure>;
  render_fixed(profile: &FormatProfile, value: f64, decimals: i32, no_commas: bool) -> Result<RenderedText, FormatFailure>;
}
```

## 9. What OxFunc Should See
OxFunc does not need the entire formatting universe directly.

It needs:
1. stable declared capability tags
2. explicit injected parse/render services when a function needs them
3. reproducible profile identifiers in evidence records

That keeps function contracts readable and prevents the formatting language from being smeared across function kernels.

## 10. Adjacent Seam: Blank Scalar References
W13 also exposed a nearby issue for the non-locale subset:
1. functions like `TYPE(A2)`, `N(A2)`, and `T(A2)` depend on a blank single-cell reference being representable as a prepared empty-cell result
2. current OxFunc has `EmptyCell` at the call-arg/prepared layer, but not as a general resolved scalar eval result

Suggested rule:
1. OxFml/FEC should be able to deliver a dereferenced single-cell blank as an explicit prepared empty-cell outcome
2. OxFunc should not be forced to infer blankness indirectly from synthetic array wrappers

## 11. Adjacent Seam: Array-Lift Ownership
W13 also pressures where array-lift should live.

Suggested rule:
1. array-lift policy should remain declarative and visible in function metadata / substrate rules
2. FEC/OxFml may execute the lift mechanically
3. OxFunc should not duplicate array-lift orchestration ad hoc in every scalar function kernel

This is especially relevant for:
1. `SIN`
2. `ASIN`
3. `N`
4. `T`
5. `ROW`
6. `COLUMN`
