use crate::coercion::{CoercionError, coerce_calc_scalar_to_number, parse_excel_logical_text};
use crate::functions::adapters::{AggregateArgOrigin, AggregatePreparedItem};
use crate::value::CoreValue;

fn is_direct_scalar(origin: AggregateArgOrigin) -> bool {
    matches!(origin, AggregateArgOrigin::DirectScalar)
}

pub(crate) fn dual_policy_numeric_value(
    item: &AggregatePreparedItem,
) -> Result<Option<f64>, CoercionError> {
    match item.0.core() {
        _ if is_direct_scalar(item.1) => coerce_calc_scalar_to_number(&item.0).map(Some),
        CoreValue::Number(n) => Ok(Some(*n)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
            Ok(None)
        }
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub(crate) fn sum_argument_value(
    item: &AggregatePreparedItem,
) -> Result<Option<f64>, CoercionError> {
    dual_policy_numeric_value(item)
}

pub(crate) fn average_argument_value(
    item: &AggregatePreparedItem,
) -> Result<Option<f64>, CoercionError> {
    dual_policy_numeric_value(item)
}

pub(crate) fn averagea_argument_value(
    item: &AggregatePreparedItem,
) -> Result<Option<f64>, CoercionError> {
    match item.0.core() {
        CoreValue::Number(n) => Ok(Some(*n)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) if is_direct_scalar(item.1) => {
            coerce_calc_scalar_to_number(&item.0).map(Some)
        }
        CoreValue::Logical(b) if is_direct_scalar(item.1) => Ok(Some(if *b { 1.0 } else { 0.0 })),
        CoreValue::Text(_) => Ok(Some(0.0)),
        CoreValue::Logical(b) => Ok(Some(if *b { 1.0 } else { 0.0 })),
        CoreValue::Missing | CoreValue::Empty => Ok(None),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub(crate) fn median_argument_value(
    item: &AggregatePreparedItem,
) -> Result<Option<f64>, CoercionError> {
    average_argument_value(item)
}

pub(crate) fn extrema_a_argument_value(
    item: &AggregatePreparedItem,
) -> Result<Option<f64>, CoercionError> {
    match item.0.core() {
        CoreValue::Number(n) => Ok(Some(*n)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) if is_direct_scalar(item.1) => {
            coerce_calc_scalar_to_number(&item.0).map(Some)
        }
        CoreValue::Logical(b) if is_direct_scalar(item.1) => Ok(Some(if *b { 1.0 } else { 0.0 })),
        CoreValue::Text(_) => Ok(Some(0.0)),
        CoreValue::Logical(b) => Ok(Some(if *b { 1.0 } else { 0.0 })),
        CoreValue::Missing | CoreValue::Empty => Ok(None),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub(crate) fn count_argument_included(item: &AggregatePreparedItem) -> Result<bool, CoercionError> {
    match item.0.core() {
        CoreValue::Number(_) => Ok(true),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) if is_direct_scalar(item.1) => coerce_calc_scalar_to_number(&item.0)
            .map(|_| true)
            .or_else(|err| match err {
                CoercionError::NonNumericText(_) => Ok(false),
                other => Err(other),
            }),
        CoreValue::Logical(_) if is_direct_scalar(item.1) => Ok(true),
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
            Ok(false)
        }
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub(crate) fn counta_argument_included(
    item: &AggregatePreparedItem,
) -> Result<bool, CoercionError> {
    match item.0.core() {
        CoreValue::Missing | CoreValue::Empty => Ok(false),
        _ => Ok(true),
    }
}

/// Per-item truth rule shared by the `AND` / `OR` / `XOR` folds: `Ok(Some(b))` is a seen logical
/// value, `Ok(None)` an ignored item, `Err` a surfacing error.
///
/// A DIRECT text item (a literal or any text-valued expression result, `AggregateArgOrigin::
/// DirectScalar`) is coerced only when it spells `TRUE`/`FALSE` — ASCII-case-insensitively, no
/// whitespace tolerance ([`parse_excel_logical_text`]) — and is otherwise IGNORED like
/// reference text, numeric text included; the fold's own no-value rule then publishes `#VALUE!`
/// when nothing at all was seen. Live Excel 16.0 build 20326 (COM probes 2026-09-15, beads
/// `oxf-xvt5.14` / `oxf-xvt5.15`, catalog G1-02): `=OR("TRUE")` -> `TRUE`, `=AND("FALSE")` ->
/// `FALSE`, `=OR(FALSE,"true")` -> `TRUE`, `=OR("FALSE","FALSE")` -> `FALSE` (a coerced spelling
/// is a seen value); `=OR(FALSE,"x")` -> `FALSE`, `=OR("x",TRUE)` -> `TRUE`, `=AND(TRUE,"0")` ->
/// `TRUE`, `=OR(FALSE,"1")` / `"1.5"` / `" 1 "` / `"1e0"` / `"$1"` / `""` / `" TRUE"` -> `FALSE`;
/// `=OR("x")`, `=OR("1")`, `=OR("")`, `=OR(" TRUE")` -> `#VALUE!` only because no value was seen;
/// `=OR("x",1/0)` -> `#DIV/0!` (ignored text never masks an error). Computed text is direct
/// (`=OR(FALSE,"TR"&"UE")` / `LOWER("TRUE")` / `E1&""` -> `TRUE`), while the same spelling in a
/// cell or an array constant stays ignored (`=OR(E1)` with text `TRUE` in `E1` -> `#VALUE!`,
/// `=OR(FALSE,{"TRUE"})` -> `FALSE`) — hence the origin split below. Before `oxf-xvt5.15` this
/// arm raised `NonNumericText` (`#VALUE!`) for every direct text.
pub(crate) fn and_argument_truth(
    item: &AggregatePreparedItem,
) -> Result<Option<bool>, CoercionError> {
    match item.0.core() {
        CoreValue::Logical(b) => Ok(Some(*b)),
        CoreValue::Number(n) => Ok(Some(*n != 0.0)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(text) if is_direct_scalar(item.1) => {
            Ok(parse_excel_logical_text(&text.to_string_lossy()))
        }
        CoreValue::Text(_) | CoreValue::Missing | CoreValue::Empty => Ok(None),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}
