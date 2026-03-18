use crate::coercion::CoercionError;
use crate::functions::adapters::{
    AggregateArgOrigin, AggregatePreparedValue, PreparedArgValue, coerce_prepared_to_number,
};

fn is_direct_scalar(origin: AggregateArgOrigin) -> bool {
    matches!(origin, AggregateArgOrigin::DirectScalar)
}

pub fn dual_policy_numeric_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    match &item.value {
        PreparedArgValue::Eval(_) if is_direct_scalar(item.origin) => {
            coerce_prepared_to_number(&item.value).map(Some)
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Number(n)) => Ok(Some(*n)),
        PreparedArgValue::Eval(crate::value::EvalValue::Error(code)) => {
            Err(CoercionError::WorksheetError(*code))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_))
        | PreparedArgValue::Eval(crate::value::EvalValue::Logical(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(crate::value::EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
    }
}

pub fn sum_argument_value(item: &AggregatePreparedValue) -> Result<Option<f64>, CoercionError> {
    dual_policy_numeric_value(item)
}

pub fn average_argument_value(item: &AggregatePreparedValue) -> Result<Option<f64>, CoercionError> {
    dual_policy_numeric_value(item)
}

pub fn averagea_argument_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    match &item.value {
        PreparedArgValue::Eval(crate::value::EvalValue::Number(n)) => Ok(Some(*n)),
        PreparedArgValue::Eval(crate::value::EvalValue::Error(code)) => {
            Err(CoercionError::WorksheetError(*code))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_))
            if is_direct_scalar(item.origin) =>
        {
            coerce_prepared_to_number(&item.value).map(Some)
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Logical(b))
            if is_direct_scalar(item.origin) =>
        {
            Ok(Some(if *b { 1.0 } else { 0.0 }))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_)) => Ok(Some(0.0)),
        PreparedArgValue::Eval(crate::value::EvalValue::Logical(b)) => {
            Ok(Some(if *b { 1.0 } else { 0.0 }))
        }
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(crate::value::EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
    }
}

pub fn median_argument_value(item: &AggregatePreparedValue) -> Result<Option<f64>, CoercionError> {
    average_argument_value(item)
}

pub fn extrema_a_argument_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    match &item.value {
        PreparedArgValue::Eval(crate::value::EvalValue::Number(n)) => Ok(Some(*n)),
        PreparedArgValue::Eval(crate::value::EvalValue::Error(code)) => {
            Err(CoercionError::WorksheetError(*code))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_))
            if is_direct_scalar(item.origin) =>
        {
            coerce_prepared_to_number(&item.value).map(Some)
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Logical(b))
            if is_direct_scalar(item.origin) =>
        {
            Ok(Some(if *b { 1.0 } else { 0.0 }))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_)) => Ok(Some(0.0)),
        PreparedArgValue::Eval(crate::value::EvalValue::Logical(b)) => {
            Ok(Some(if *b { 1.0 } else { 0.0 }))
        }
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(crate::value::EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
    }
}

pub fn count_argument_included(item: &AggregatePreparedValue) -> Result<bool, CoercionError> {
    match &item.value {
        PreparedArgValue::Eval(crate::value::EvalValue::Number(_)) => Ok(true),
        PreparedArgValue::Eval(crate::value::EvalValue::Error(code)) => {
            Err(CoercionError::WorksheetError(*code))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_))
            if is_direct_scalar(item.origin) =>
        {
            coerce_prepared_to_number(&item.value)
                .map(|_| true)
                .or_else(|err| match err {
                    CoercionError::NonNumericText(_) => Ok(false),
                    other => Err(other),
                })
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Logical(_))
            if is_direct_scalar(item.origin) =>
        {
            Ok(true)
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_))
        | PreparedArgValue::Eval(crate::value::EvalValue::Logical(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => Ok(false),
        PreparedArgValue::Eval(crate::value::EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
    }
}

pub fn counta_argument_included(item: &AggregatePreparedValue) -> Result<bool, CoercionError> {
    match &item.value {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(false),
        PreparedArgValue::Eval(_) => Ok(true),
    }
}

pub fn and_argument_truth(item: &AggregatePreparedValue) -> Result<Option<bool>, CoercionError> {
    match &item.value {
        PreparedArgValue::Eval(crate::value::EvalValue::Logical(b)) => Ok(Some(*b)),
        PreparedArgValue::Eval(crate::value::EvalValue::Number(n)) => Ok(Some(*n != 0.0)),
        PreparedArgValue::Eval(crate::value::EvalValue::Error(code)) => {
            Err(CoercionError::WorksheetError(*code))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_))
            if is_direct_scalar(item.origin) =>
        {
            Err(CoercionError::NonNumericText("direct_text".to_string()))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Text(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(crate::value::EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(crate::value::EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
    }
}
