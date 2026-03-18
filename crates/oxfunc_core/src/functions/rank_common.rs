use crate::coercion::CoercionError;
use crate::functions::adapters::{
    AggregatePreparedValue, PreparedArgValue, coerce_prepared_to_number, prepare_arg_values_only,
};
use crate::functions::aggregate_common::median_argument_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, WorksheetErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankOrder {
    Descending,
    Ascending,
}

pub fn collect_rank_values(args: &[AggregatePreparedValue]) -> Result<Vec<f64>, CoercionError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = median_argument_value(arg)? {
            values.push(value);
        }
    }
    Ok(values)
}

pub fn prepare_rank_number(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<Option<f64>, CoercionError> {
    let prepared = prepare_arg_values_only(arg, resolver)?;
    match prepared {
        PreparedArgValue::Eval(crate::value::EvalValue::Number(n)) => Ok(Some(n)),
        PreparedArgValue::Eval(crate::value::EvalValue::Error(code)) => {
            Err(CoercionError::WorksheetError(code))
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

pub fn prepare_rank_order(
    arg: Option<&CallArgValue>,
    resolver: &impl ReferenceResolver,
) -> Result<RankOrder, CoercionError> {
    let Some(arg) = arg else {
        return Ok(RankOrder::Descending);
    };
    let prepared = prepare_arg_values_only(arg, resolver)?;
    let value = coerce_prepared_to_number(&prepared)?.trunc();
    Ok(if value == 0.0 {
        RankOrder::Descending
    } else {
        RankOrder::Ascending
    })
}

pub fn rank_eq(number: f64, values: &[f64], order: RankOrder) -> Result<f64, WorksheetErrorCode> {
    if values.is_empty() {
        return Err(WorksheetErrorCode::NA);
    }
    let matches = values.iter().filter(|value| **value == number).count();
    if matches == 0 {
        return Err(WorksheetErrorCode::NA);
    }

    let rank = match order {
        RankOrder::Descending => values.iter().filter(|value| **value > number).count() + 1,
        RankOrder::Ascending => values.iter().filter(|value| **value < number).count() + 1,
    };
    Ok(rank as f64)
}

pub fn rank_avg(number: f64, values: &[f64], order: RankOrder) -> Result<f64, WorksheetErrorCode> {
    let first = rank_eq(number, values, order)?;
    let matches = values.iter().filter(|value| **value == number).count() as f64;
    Ok(first + (matches - 1.0) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_eq_descending_basic_lane() {
        let values = vec![30.0, 20.0, 20.0, 10.0];
        let got = rank_eq(20.0, &values, RankOrder::Descending).unwrap();
        assert_eq!(got, 2.0);
    }

    #[test]
    fn rank_avg_descending_duplicate_lane() {
        let values = vec![30.0, 20.0, 20.0, 10.0];
        let got = rank_avg(20.0, &values, RankOrder::Descending).unwrap();
        assert_eq!(got, 2.5);
    }

    #[test]
    fn rank_eq_ascending_basic_lane() {
        let values = vec![30.0, 20.0, 20.0, 10.0];
        let got = rank_eq(20.0, &values, RankOrder::Ascending).unwrap();
        assert_eq!(got, 2.0);
    }
}
