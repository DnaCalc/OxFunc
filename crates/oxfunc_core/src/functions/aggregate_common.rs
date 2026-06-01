use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::functions::adapters::{AggregateArgOrigin, AggregatePreparedValue};
use crate::value::CoreValue;

fn is_direct_scalar(origin: AggregateArgOrigin) -> bool {
    matches!(origin, AggregateArgOrigin::DirectScalar)
}

pub(crate) fn dual_policy_numeric_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    match item.value().core() {
        _ if is_direct_scalar(item.origin()) => {
            coerce_calc_scalar_to_number(item.value()).map(Some)
        }
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
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    dual_policy_numeric_value(item)
}

pub(crate) fn average_argument_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    dual_policy_numeric_value(item)
}

pub(crate) fn averagea_argument_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    match item.value().core() {
        CoreValue::Number(n) => Ok(Some(*n)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) if is_direct_scalar(item.origin()) => {
            coerce_calc_scalar_to_number(item.value()).map(Some)
        }
        CoreValue::Logical(b) if is_direct_scalar(item.origin()) => {
            Ok(Some(if *b { 1.0 } else { 0.0 }))
        }
        CoreValue::Text(_) => Ok(Some(0.0)),
        CoreValue::Logical(b) => Ok(Some(if *b { 1.0 } else { 0.0 })),
        CoreValue::Missing | CoreValue::Empty => Ok(None),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub(crate) fn median_argument_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    average_argument_value(item)
}

pub(crate) fn extrema_a_argument_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    match item.value().core() {
        CoreValue::Number(n) => Ok(Some(*n)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) if is_direct_scalar(item.origin()) => {
            coerce_calc_scalar_to_number(item.value()).map(Some)
        }
        CoreValue::Logical(b) if is_direct_scalar(item.origin()) => {
            Ok(Some(if *b { 1.0 } else { 0.0 }))
        }
        CoreValue::Text(_) => Ok(Some(0.0)),
        CoreValue::Logical(b) => Ok(Some(if *b { 1.0 } else { 0.0 })),
        CoreValue::Missing | CoreValue::Empty => Ok(None),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub(crate) fn count_argument_included(
    item: &AggregatePreparedValue,
) -> Result<bool, CoercionError> {
    match item.value().core() {
        CoreValue::Number(_) => Ok(true),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) if is_direct_scalar(item.origin()) => {
            coerce_calc_scalar_to_number(item.value())
                .map(|_| true)
                .or_else(|err| match err {
                    CoercionError::NonNumericText(_) => Ok(false),
                    other => Err(other),
                })
        }
        CoreValue::Logical(_) if is_direct_scalar(item.origin()) => Ok(true),
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
            Ok(false)
        }
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}

pub(crate) fn counta_argument_included(
    item: &AggregatePreparedValue,
) -> Result<bool, CoercionError> {
    match item.value().core() {
        CoreValue::Missing | CoreValue::Empty => Ok(false),
        _ => Ok(true),
    }
}

pub(crate) fn and_argument_truth(
    item: &AggregatePreparedValue,
) -> Result<Option<bool>, CoercionError> {
    match item.value().core() {
        CoreValue::Logical(b) => Ok(Some(*b)),
        CoreValue::Number(n) => Ok(Some(*n != 0.0)),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(_) if is_direct_scalar(item.origin()) => {
            Err(CoercionError::NonNumericText("direct_text".to_string()))
        }
        CoreValue::Text(_) | CoreValue::Missing | CoreValue::Empty => Ok(None),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
    }
}
