use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::median_argument_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const MEDIAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MEDIAN",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MedianEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_median_aggregate(args: &[AggregatePreparedValue]) -> Result<CalcValue, MedianEvalError> {
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = median_argument_value(arg).map_err(MedianEvalError::Coercion)? {
            values.push(value);
        }
    }
    if values.is_empty() {
        return Ok(CalcValue::error(WorksheetErrorCode::Num));
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = values.len();
    let mid = len / 2;
    let result = if len % 2 == 1 {
        values[mid]
    } else {
        (values[mid - 1] + values[mid]) / 2.0
    };
    Ok(CalcValue::number(result))
}

pub fn eval_median_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, MedianEvalError> {
    let argc = args.len();
    if !MEDIAN_META.arity.accepts(argc) {
        return Err(MedianEvalError::ArityMismatch {
            expected_min: MEDIAN_META.arity.min,
            expected_max: MEDIAN_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(MedianEvalError::Coercion)?);
    }
    eval_median_aggregate(&prepared)
}

pub fn map_median_error_to_ws(e: &MedianEvalError) -> WorksheetErrorCode {
    match e {
        MedianEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MedianEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MedianEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CalcArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved_value: Option<CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_median_accumulates_direct_numbers() {
        let args = vec![
            (CalcValue::number(2.0)),
            (CalcValue::number(3.0)),
            (CalcValue::number(4.0)),
        ];
        let got = eval_median_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(3.0)));
    }

    #[test]
    fn eval_median_even_count_averages_middle_pair() {
        let args = vec![(CalcValue::number(2.0)), (CalcValue::number(3.0))];
        let got = eval_median_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(2.5)));
    }

    #[test]
    fn eval_median_counts_direct_numeric_text_and_logical() {
        let args = vec![
            (CalcValue::logical(true)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_median_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(1.5)));
    }

    #[test]
    fn eval_median_ignored_reference_values_yield_num_when_empty() {
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A2".to_string(),
        ))];
        let got = eval_median_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        CalcValue::logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(CalcValue::error(WorksheetErrorCode::Num)));
    }
}
