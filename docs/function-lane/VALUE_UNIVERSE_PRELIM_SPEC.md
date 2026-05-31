# Value Universe Preliminary Spec (W3)

Status: `active`
Workset: `W3`

## 1. Purpose
Define a boundary-scoped value universe for OxFunc/F3E so function contracts, coercion rules, and formal/runtime artifacts use one shared tag vocabulary.

## 2. Boundary Sets
The universe is decomposed into boundary-specific sets:
1. `CellContentValue`
2. `RawFunctionReturn`
3. `PublishedFormulaResult`
4. `CallArgValue`
5. `ReferenceLike`
6. `ExtendedValue`
7. `RichValueData`

Interpretation rule:
1. these sets are boundary views over one common tag algebra,
2. not every tag is admitted at every boundary,
3. the current Rust `EvalValue` type corresponds most closely to `PublishedFormulaResult`, not to the broadest raw interop/UDF return universe.

## 2A. Two-Tier CalcValue Direction
W098 promotes this boundary-scoped algebra into the system value carrier:

`CalcValue { core: CoreValue, rich: Option<Rc<RichValue>> }`

This is deliberately a two-tier model rather than one flat enum containing every current and future value shape.

1. `CoreValue` is the traditional Excel-compatible calculation substrate. It captures the limited value gamut shared by the mature public Excel surfaces: C API/XLOPER12-style values, COM automation `Range.Value`/`Value2` payloads, VBA UDF interaction, and ordinary worksheet formula exchange.
2. `RichValue` is the extension layer. It carries modern Excel value features and DNA Calc-specific extensions whose meaning exceeds the traditional core substrate.
3. Every `CalcValue` has a `core` projection. For ordinary scalar/array/reference values, the core is the whole value. For rich values, the core is the compatibility projection used by legacy-style calculation, coercion, publication fallback, display fallback, and degradation.
4. Not every `CoreValue` has a rich companion. Rich payloads are used only when a value has additional semantic identity or payload that cannot be faithfully represented by the core projection alone.
5. This mirrors Excel's own interface reality: Excel exposes a small traditional gamut through C API/COM/VBA surfaces while newer rich-cell features travel through richer object models and metadata-bearing carriers. DNA Calc keeps that separation explicit instead of flattening all value forms into one extended legacy-facing enum.

The durable naming direction is:
1. `CalcValue` — the system value carrier passed through OxFunc/OxFml/OxCalc and stored as a node value where admitted.
2. `CoreValue` — the required traditional calculation projection. Rust-facing names are
   `CoreValue::Empty` for the `empty_cell` tag and `CoreValue::Missing` for the `missing_arg`
   tag.
3. `RichValue` — the optional modern/extended semantic payload, including callable handles.
   W098 maps the legacy `lambda_value` tag to `RichValue::Callable`, not to a core value
   variant.

## 3. Tag Algebra
Canonical tag list for W3 baseline:
1. `number`
2. `text`
3. `logical`
4. `error`
5. `array`
6. `reference_like`
7. `missing_arg`
8. `empty_cell`
9. `lambda_value`
10. `rich_value`
11. `extended_wrapper`
12. `null_like` (reserved disputed category; not admitted in baseline boundaries)

Machine-readable table:
1. `VALUE_UNIVERSE_TAG_TABLE.csv`

## 4. Baseline Boundary Admission Policy
1. `CellContentValue` admits:
   - `number`, `text`, `logical`, `error`, `empty_cell`
2. `RawFunctionReturn` admits:
   - `number`, `text`, `logical`, `error`, `array`, `rich_value`, `reference_like`, `lambda_value`, `empty_cell`
   - and explicitly does **not** admit `missing_arg`, `null_like`
3. `PublishedFormulaResult` admits:
   - `number`, `text`, `logical`, `error`, `array`, `rich_value`, `reference_like`, `lambda_value`
   - and explicitly does **not** admit `missing_arg`, `empty_cell`, `null_like`
4. `CallArgValue` admits:
   - all `PublishedFormulaResult` tags plus `missing_arg` and `empty_cell`
5. `ReferenceLike` boundary admits:
   - `reference_like` only
6. `ExtendedValue` boundary admits:
   - `extended_wrapper` plus pass-through of core evaluable tags
7. `RichValueData` admits:
   - scalar data (`number`, `text`, `logical`, `error`, `empty_cell`)
   - `rich_array`
   - nested `rich_value`

## 5. Disputed Categories
### 5.1 Missing
1. represented as `missing_arg`,
2. Rust-facing name: `CoreValue::Missing`,
3. treated as call-boundary specific, not published-result specific,
4. not admitted as a node literal, ordinary array element, persisted value, or published
   formula result.

### 5.2 Empty
1. represented as `empty_cell`,
2. Rust-facing name: `CoreValue::Empty`,
3. treated as cell/call-boundary representable,
4. admitted in `RawFunctionReturn` for interop/UDF raw-return characterization,
5. not admitted in `PublishedFormulaResult`.

### 5.3 Null
1. represented only as reserved `null_like` tag in baseline algebra,
2. not admitted in any baseline boundary until direct evidence exists.
3. `#NULL!` is modeled as `error`, not `null_like`.

## 6. Error Taxonomy and Versioning
Error values remain scalar `error` tags with code-level metadata.

Code-level registry split (provisional):
1. legacy transferable family (`#NULL!`, `#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#N/A`)
2. extended worksheet-era family (for example `#SPILL!`, `#CALC!`, `#FIELD!`, `#BLOCKED!`, `#CONNECT!`)

Versioning rule:
1. code family membership is version-scoped by Excel build/channel and compatibility mode.

## 7. Text Subtype Baseline (W7 Feed)
For boundary-faithful modeling in this baseline scope:
1. text is treated as UTF-16 code-unit sequence at worksheet/interop boundaries.
2. observed cap for materialized text is `32767` UTF-16 code units.
3. interop ingress (`Range.Value2`) truncates over-cap strings to cap without set-time exception in tested rows.
4. truncation of surrogate-pair streams can yield dangling high-surrogate tail states (observed in emoji overflow scenario).
5. formula-generated overflow behavior is distinct from interop ingress:
   - formula path above cap produced `#VALUE!` in tested `REPT` rows.

Evidence binding:
1. `W7-STR-BL-20260305` (`STR8-019..STR8-046`).

## 8. Arrays, Lambda, and References
1. arrays are first-class `RawFunctionReturn` and `PublishedFormulaResult` tags; materialization policy is downstream (W4/W5/W6).
2. the current `array` tag models ordinary worksheet arrays (`EvalArray`) whose cells are scalar worksheet-like atoms.
3. spec `rich array` is different: it is a rich-value-data container whose elements may themselves be rich value data, including nested rich values.
4. callable/lambda values are intermediate-eval tags, not baseline cell content tags; under
   W098 they are represented as `RichValue::Callable` with a `#CALC!` core projection.
5. 3D references are modeled as a `reference_like` subtype (`reference_kind=three_d`) and require resolver-policy handling in W4.

## 9. Rich Value Alignment
The value universe now uses the SpreadsheetML rich-value vocabulary explicitly:
1. `rich value`
   - represented as:
     - `rich value type`
     - `rich value fallback`
     - key/value pairs
2. `rich value type`
   - represented as:
     - type name
     - required keys
     - key-flag declarations
3. `rich value data`
   - represented as the recursive domain:
     - scalar atom
     - `rich array`
     - nested `rich value`
4. `rich array`
   - represented as a shaped 2D container over `rich value data`
5. `extended_wrapper`
   - remains as a backward-compatible bucket for non-rich extensions such as format hints and enriched error-surface metadata.

Current design reading:
1. `IMAGE` pressures `rich_value`, not just `extended_wrapper`.
2. `HYPERLINK` still looks like ordinary text plus publication/style behavior, not a rich value.
3. dynamic-array functions currently operate over ordinary worksheet arrays, but the model now has an explicit place for future rich-array-bearing cells and nested rich-data values.

## 9A. Presentation Hint Alignment
For worksheet-facing cases where Excel applies formatting or styling without changing the underlying scalar value, the model now uses a uniform presentation wrapper:
1. `ValueWithPresentation`
   - carries an ordinary `EvalValue`
   - plus a `PresentationHint`
2. `PresentationHint`
   - `number_format`
   - `style`
3. baseline examples:
   - `TODAY()` / `NOW()`:
     - numeric serial value
     - plus `number_format` hint
   - `HYPERLINK(...)`:
     - ordinary text value
     - plus `style=hyperlink` hint

Design consequence:
1. number-format hints and style hints remain distinct semantic subfields,
2. but they share one wrapper so OxFml and the host-facing publication layer can consume them uniformly.
3. the current first-pass runtime hook for these cases is `eval_surface_extended_call(...)` in `crates/oxfunc_core/src/functions/surface_dispatch.rs`.

## 10. Lean/Rust Mirror Rule
W3 baseline requires shared tag vocabulary in:
1. Rust: `crates/oxfunc_core/src/value.rs`
2. Lean: `formal/lean/OxFunc/ValueUniverse.lean`

Invariant objectives for baseline:
1. `PublishedFormulaResult` excludes `missing_arg`, `empty_cell`, `null_like`.
2. `RawFunctionReturn` admits `empty_cell` but excludes `missing_arg` and `null_like`.
3. text-cap primitive encodes `<= 32767` UTF-16 code units.

## 11. Integration Hooks
1. W4 consumes tag/boundary policy for coercion and `Ref -> PublishedFormulaResult` seam typing.
2. W5 consumes numeric/error/array policy for `ABS`.
3. W6 consumes text/reference/error policy for `XMATCH`.
4. W9 consumes raw-return versus publication normalization policy for XLL/UDF seam characterization.

## 12. Raw Return vs Published Result Rule
Observed XLL/UDF probe evidence on `2026-03-12` (`W9-XLL-NIL-20260312`) now pins the following baseline rule:
1. raw scalar `xltypeNil` can be returned from an XLL function,
2. but it does not survive as an outer-observable scalar value in ordinary nested formula evaluation,
3. instead it normalizes to numeric-zero semantics before outer argument binding and final worksheet publication,
4. raw `xltypeNil` inside arrays is different:
   - it can survive as `empty_cell`-like element state inside an intermediate array value,
   - but it also collapses to numeric-zero semantics when scalarized or published into worksheet cells.

Design consequence for the current model:
1. `RawFunctionReturn` is broader than `PublishedFormulaResult`.
2. The publication/scalarization map is semantically important and must stay explicit in doctrine.
3. Built-in function completion should continue to target `PublishedFormulaResult` semantics even when OxFunc also models broader raw interop/UDF return shapes.

## 13. Open Points
1. explicit handling for modern dynamic-array error transfer across UDF/XLL boundaries,
2. empirical evidence for any first-class `null_like` behavior,
3. final decision on whether internal OxFunc runtime should store text as UTF-16 code-unit vectors end-to-end or only at boundary adapters,
4. whether ordinary worksheet arrays (`array`) and spec rich arrays should share one runtime carrier or stay split,
5. whether the W3 Lean/Rust mirrors should grow a first-class `publish_result` normalization model rather than staying boundary-table only.
