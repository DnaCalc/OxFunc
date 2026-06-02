use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

pub const AVERAGE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.AVERAGE",
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
pub enum AverageEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_average_aggregate(
    args: &[AggregatePreparedValue],
) -> Result<FunctionValue, AverageEvalError> {
    let mut acc = 0.0;
    let mut count = 0usize;
    for arg in args {
        if let Some(value) = average_argument_value(arg).map_err(AverageEvalError::Coercion)? {
            acc += value;
            count += 1;
        }
    }

    if count == 0 {
        return Ok(FunctionValue::Error(WorksheetErrorCode::Div0));
    }

    Ok(FunctionValue::Number(acc / count as f64))
}

pub fn eval_average_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, AverageEvalError> {
    let argc = args.len();
    if !AVERAGE_META.arity.accepts(argc) {
        return Err(AverageEvalError::ArityMismatch {
            expected_min: AVERAGE_META.arity.min,
            expected_max: AVERAGE_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(AverageEvalError::Coercion)?);
    }
    eval_average_aggregate(&prepared)
}

pub fn map_average_error_to_ws(e: &AverageEvalError) -> WorksheetErrorCode {
    match e {
        AverageEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        AverageEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        AverageEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, FunctionArray, FunctionArrayCell, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved_value: Option<FunctionValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_average_on_numbers() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Number(2.0)),
            FunctionArg::Eval(FunctionValue::Number(5.0)),
        ];
        let got = eval_average_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(8.0 / 3.0)));
    }

    #[test]
    fn eval_average_counts_direct_numeric_text_and_logical() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            FunctionArg::Eval(FunctionValue::Logical(true)),
        ];
        let got = eval_average_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(1.5)));
    }

    #[test]
    fn eval_average_ignores_reference_derived_text_and_logical() {
        let args = vec![FunctionArg::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let got = eval_average_surface(
            &args,
            &MockResolver {
                resolved_value: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![FunctionArrayCell::Number(6.0)],
                        vec![FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        ))],
                        vec![FunctionArrayCell::Logical(true)],
                    ])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(6.0)));
    }

    #[test]
    fn eval_average_returns_div0_when_no_numeric_values_survive() {
        let args = vec![FunctionArg::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A2".to_string(),
        ))];
        let got = eval_average_surface(
            &args,
            &MockResolver {
                resolved_value: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        FunctionArrayCell::EmptyCell,
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Error(WorksheetErrorCode::Div0)));
    }

    #[test]
    fn eval_average_direct_array_uses_range_like_policy() {
        let got = eval_average_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    )),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap(),
            ))],
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(FunctionValue::Error(WorksheetErrorCode::Div0)));
    }
}
