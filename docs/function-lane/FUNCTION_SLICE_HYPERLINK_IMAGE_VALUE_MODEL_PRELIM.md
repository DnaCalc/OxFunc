# Function Slice Note (Prelim) - HYPERLINK() / IMAGE() Value and Publication Model

## 1. Purpose
Capture the current-baseline OxFunc-side reading of what crosses the value boundary for `HYPERLINK` and `IMAGE`.

## 2. Evidence Surface
1. `tools/w23-probe/run-w23-hyperlink-image-value-model-baseline.ps1`
2. `.tmp/w23-hyperlink-image-value-model-results.csv`
3. `tools/w23-probe/run-w23-host-provider-classification.ps1`
4. `.tmp/w23-host-provider-classification-results.csv`
5. Microsoft Support `IMAGE` example URL:
   - `https://support.microsoft.com/en-us/office/image-function-7e112975-5e52-4f2a-b9da-1d913d51f5d5`

## 3. Current-Baseline Findings
1. `HYPERLINK("https://example.com","Go")` crosses the value boundary as ordinary text:
   - the formula cell has text/value `Go`,
   - `TYPE(...) = 2`,
   - `CELL("contents", ...) = "Go"`,
   - `T(...) = "Go"`,
   - `N(...) = 0`.
2. A referencing cell `=A1` receives the same plain text value, but does not preserve the hyperlink-style underline/publication treatment seen on the formula cell.
3. On the current baseline, `HYPERLINK` therefore looks like:
   - ordinary scalar text value in OxFunc,
   - plus host-side publication metadata/click behavior attached to the formula cell.
4. `IMAGE(...)` does not currently look like an ordinary scalar value on the baseline:
   - provider/binding failure lanes project worksheet-visible provider-style errors such as `#CONNECT!`,
   - a successful Microsoft support-example URL produces a non-ordinary payload where `TYPE(...) = 128`,
   - `CELL("contents", ...)` returns an opaque sentinel rather than a user text/number value,
   - and a referencing cell preserves the same non-ordinary payload shape.
5. This pressures an extended/rich host-managed value or publication-object model for `IMAGE`, not a plain scalar OxFunc value.

## 4. Current OxFunc Design Reading
1. `HYPERLINK` should be modeled as:
   - ordinary text value owned by OxFunc,
   - plus a presentation/style hint (`style=hyperlink`) carried alongside that text value,
   - with OxFml or the host-facing publication layer responsible for applying the hint to the formula cell.
2. the current OxFunc runtime shape for that is:
   - plain value path: `eval_hyperlink_surface(...) -> EvalValue::Text(...)`
   - extended publication-aware path: `eval_hyperlink_surface_extended(...) -> ValueWithPresentation(value=text, style=hyperlink)`
3. `IMAGE` return carriage is now pinned more carefully:
   - semantic return carrier: `ExtendedValue::RichValue(_webimage)` with typed request-normalized metadata
   - published worksheet value: provider-supplied fallback text on success, or classified provider-style worksheet errors such as `#CONNECT!` / `#BLOCKED!`
   - file/web access itself remains host/provider-owned through upstream helpers
   - the published worksheet fallback is not the semantic return carrier
   - OxFml now exercises a real `IMAGE(...)` lane through evaluator, host, and adapter paths, including `TypedContextQueryFamily::Image`
   - the remaining gap is no longer carrier definition; it is first-freeze cleanup on the host-query family name and any last returned-value-field questions

## 5. Status
1. runtime_status: `evidenced`
2. seam_status: `return_carrier_locked_and_exercised`
3. closure_reading:
   - `HYPERLINK` value side and first-pass presentation-hint carrier are now understood,
   - `IMAGE` runtime and return carrier are now real on the OxFunc side, and OxFml now has exercised evaluator/host/adapter coverage for the admitted lane.
