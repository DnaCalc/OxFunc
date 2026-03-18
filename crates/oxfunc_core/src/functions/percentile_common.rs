use crate::coercion::CoercionError;
use crate::functions::adapters::{
    AggregatePreparedValue, PreparedArgValue, coerce_prepared_to_number,
};
use crate::value::{EvalValue, WorksheetErrorCode};

pub fn percentile_argument_value(
    item: &AggregatePreparedValue,
) -> Result<Option<f64>, CoercionError> {
    match &item.value {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(Some(*n)),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        PreparedArgValue::Eval(EvalValue::Text(_))
        | PreparedArgValue::Eval(EvalValue::Logical(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => Ok(None),
        PreparedArgValue::Eval(EvalValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
    }
}

pub fn collect_percentile_values(
    args: &[AggregatePreparedValue],
) -> Result<Vec<f64>, CoercionError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = percentile_argument_value(arg)? {
            values.push(value);
        }
    }
    Ok(values)
}

pub fn percentile_inc_kernel(values: &mut [f64], k: f64) -> Result<f64, WorksheetErrorCode> {
    if values.is_empty() {
        return Err(WorksheetErrorCode::Num);
    }
    if !(0.0..=1.0).contains(&k) {
        return Err(WorksheetErrorCode::Num);
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if values.len() == 1 {
        return Ok(values[0]);
    }
    let rank = 1.0 + k * (values.len() as f64 - 1.0);
    interpolate_sorted(values, rank)
}

pub fn percentile_exc_kernel(values: &mut [f64], k: f64) -> Result<f64, WorksheetErrorCode> {
    if values.is_empty() {
        return Err(WorksheetErrorCode::Num);
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = k * (values.len() as f64 + 1.0);
    if rank < 1.0 || rank > values.len() as f64 {
        return Err(WorksheetErrorCode::Num);
    }
    interpolate_sorted(values, rank)
}

fn interpolate_sorted(values: &[f64], rank: f64) -> Result<f64, WorksheetErrorCode> {
    let lower_index = rank.floor() as usize;
    let upper_index = rank.ceil() as usize;
    if lower_index == 0
        || upper_index == 0
        || lower_index > values.len()
        || upper_index > values.len()
    {
        return Err(WorksheetErrorCode::Num);
    }
    if lower_index == upper_index {
        return Ok(values[lower_index - 1]);
    }
    let lower = values[lower_index - 1];
    let upper = values[upper_index - 1];
    Ok(lower + (rank - lower_index as f64) * (upper - lower))
}

pub fn quartile_k(prepared: &PreparedArgValue) -> Result<i64, CoercionError> {
    Ok(coerce_prepared_to_number(prepared)?.trunc() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_inc_interpolates_seed_lane() {
        let mut values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile_inc_kernel(&mut values, 0.3), Ok(2.2));
    }

    #[test]
    fn percentile_exc_interpolates_seed_lane() {
        let mut values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let got = percentile_exc_kernel(&mut values, 0.3).unwrap();
        assert!((got - 1.8).abs() < 1e-12);
    }
}
