# HO-FN-009 - Locale/format seam ownership realignment

## 1. Direction
- **From**: `OxFunc`
- **To**: `OxFml`
- **Filed date**: `2026-04-10`
- **Source workset**: `W082`
- **Related inbound handoff**: none

## 2. Purpose
State the exact intended OxFunc <-> OxFml/FEC decomposition for locale/format
support and make the required downstream support explicit before OxFunc lands
the ownership realignment. This packet is written for both OxFml implementers
and OxFml-side callers/users of the OxFunc surface.

## 3. Why This Is Changing
The current OxFunc runtime shape is only a bootstrap approximation of the right
architecture:
1. OxFunc locale-sensitive functions already consume a typed
   `LocaleFormatContext`,
2. but OxFunc currently also ships local runtime parser/formatter
   implementations plus convenience context constructors,
3. that leaves format-language ownership split across repos and hides the real
   dependency boundary,
4. the intended program shape is simpler and stricter:
   - OxFunc owns function semantics and the typed seam,
   - OxFml/FEC owns the actual parser/formatter implementation and supplies it
     to OxFunc.

## 4. Exact OxFunc Target State
When `W082` lands, OxFunc is expected to move to this shape:

1. OxFunc retains the typed seam contract:
   - `LocaleFormatContext`
   - `FormatProfile`
   - `WorkbookDateSystem`
   - parser/formatter trait interfaces used by locale-sensitive functions

2. OxFunc does not retain an OxFunc-owned production parser/formatter runtime
   path:
   - no production `current_excel_host_context()` bootstrap path,
   - no production `en_us_context()` bootstrap path,
   - no ordinary runtime fallback where OxFunc silently provides its own format
     parser/renderer if the caller does not.

3. OxFunc locale-sensitive functions continue to own function semantics only:
   - argument coercion and arity handling,
   - blank/logical/error behavior,
   - decisions about when parse/render capability is needed,
   - mapping seam failures into worksheet-visible outcomes where the function
     contract requires that.

4. The actual parse/render behavior is supplied by the caller through
   `LocaleFormatContext`.

## 5. Affected OxFunc Surfaces
The immediate affected local surfaces are:
1. `TEXT`
2. `DOLLAR`
3. `FIXED`
4. `VALUE`

Closely adjacent locale-profile / parse surfaces that must be checked for the
same final ownership shape:
1. `NUMBERVALUE`
2. `DATEVALUE`
3. `TIMEVALUE`

The required implementation review should also include any host bridge or test
bridge that currently constructs a locale-format context inside OxFunc.

## 6. What OxFml/FEC Must Provide
OxFml/FEC must provide the concrete locale/format capability bundle used by the
OxFunc surface call.

That means OxFml/FEC must supply:
1. a `FormatProfile` appropriate for the workbook/host context:
   - decimal separator
   - thousands separator
   - list separator
   - currency symbol
   - date/time separators
   - currency decimal policy
2. workbook date-system selection
3. a concrete parser implementation for locale-sensitive text-to-number/date
   interpretation
4. a concrete formatter implementation for Excel format-code rendering

This is not a request for OxFml to pre-format results for OxFunc.
The intended seam is:
1. OxFml/FEC constructs the capability bundle,
2. OxFunc evaluates the function using that supplied capability bundle,
3. OxFunc returns the normal worksheet value/result.

## 7. What OxFml-Side Callers Must Assume
For any OxFml-side caller using the OxFunc surface directly:
1. locale-sensitive functions must not be called without a real
   `LocaleFormatContext`,
2. there will be no OxFunc bootstrap fallback that quietly supplies one,
3. if the caller omits the capability bundle, that is a caller/seam failure,
   not a hidden request for OxFunc to improvise formatting behavior.

Practical implication for OxFml users:
1. `TEXT`, `DOLLAR`, `FIXED`, and `VALUE` should be expected to depend on the
   OxFml/FEC formatting capability,
2. any OxFml execution path that currently calls OxFunc without locale-format
   support must be updated before consuming the `W082` ref,
3. this same principle likely applies to adjacent locale-profile parse surfaces
   such as `NUMBERVALUE`, `DATEVALUE`, and `TIMEVALUE`.

## 8. What Is Explicitly Not Supported
This packet does not ask for:
1. OxFml to pre-render `TEXT` / `DOLLAR` / `FIXED` results outside OxFunc,
2. dual ownership where both repos keep their own production formatter engines,
3. a backward-compatible OxFunc local fallback retained "just in case",
4. a claim that the full Excel formatting language is already complete.

## 9. Expected Downstream Work
OxFml/FEC follow-up should cover:
1. supply the production locale/format capability bundle on OxFunc calls that
   require it,
2. align any existing adapter signatures or call construction around the
   caller-supplied seam,
3. update OxFml-side tests/corpus runners so locale-sensitive functions are
   exercised through the new capability path,
4. communicate to any OxFml-side callers that locale-sensitive OxFunc
   evaluation now has an explicit required capability dependency.

## 10. Ack Criteria
Please acknowledge this packet once:
1. OxFml agrees with the ownership split,
2. the OxFml/FEC owner for the concrete parser/formatter implementation is
   identified,
3. the OxFml-side call paths that need `LocaleFormatContext` support are known,
4. any user-facing or caller-facing migration note needed on the OxFml side has
   an owner.
