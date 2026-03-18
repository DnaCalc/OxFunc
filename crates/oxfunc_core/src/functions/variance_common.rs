use crate::coercion::CoercionError;
use crate::functions::adapters::AggregatePreparedValue;
use crate::functions::aggregate_common::{average_argument_value, averagea_argument_value};
use crate::value::WorksheetErrorCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarianceInclusionPolicy {
    AverageLike,
    AverageALike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarianceDivisor {
    Population,
    Sample,
}

pub fn collect_variance_values(
    args: &[AggregatePreparedValue],
    policy: VarianceInclusionPolicy,
) -> Result<Vec<f64>, CoercionError> {
    let mut values = Vec::new();
    for arg in args {
        let value = match policy {
            VarianceInclusionPolicy::AverageLike => average_argument_value(arg)?,
            VarianceInclusionPolicy::AverageALike => averagea_argument_value(arg)?,
        };
        if let Some(value) = value {
            values.push(value);
        }
    }
    Ok(values)
}

pub fn variance_from_values(
    values: &[f64],
    divisor: VarianceDivisor,
) -> Result<f64, WorksheetErrorCode> {
    if values.is_empty() {
        return Err(WorksheetErrorCode::Div0);
    }
    if matches!(divisor, VarianceDivisor::Sample) && values.len() < 2 {
        return Err(WorksheetErrorCode::Div0);
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let sumsq = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>();
    let denom = match divisor {
        VarianceDivisor::Population => values.len() as f64,
        VarianceDivisor::Sample => (values.len() - 1) as f64,
    };
    Ok(sumsq / denom)
}

pub fn stdev_from_values(
    values: &[f64],
    divisor: VarianceDivisor,
) -> Result<f64, WorksheetErrorCode> {
    variance_from_values(values, divisor).map(f64::sqrt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variance_population_and_sample_split_basic_lane() {
        let values = vec![2.0, 3.0, 4.0];
        let pop = variance_from_values(&values, VarianceDivisor::Population).unwrap();
        let sample = variance_from_values(&values, VarianceDivisor::Sample).unwrap();
        assert!((pop - (2.0 / 3.0)).abs() < 1e-12);
        assert!((sample - 1.0).abs() < 1e-12);
    }

    #[test]
    fn sample_variance_requires_two_values() {
        let err = variance_from_values(&[1.0], VarianceDivisor::Sample).unwrap_err();
        assert_eq!(err, WorksheetErrorCode::Div0);
    }
}
