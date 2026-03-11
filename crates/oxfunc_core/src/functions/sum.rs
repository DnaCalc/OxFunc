use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    AggregateArgOrigin, AggregatePreparedValue, PreparedArgValue, coerce_prepared_to_number,
    expand_aggregate_arg,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const SUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUM",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SumEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn accumulate_direct_scalar(arg: &PreparedArgValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
        other => coerce_prepared_to_number(other),
    }
}

fn accumulate_range_like(arg: &PreparedArgValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(*n),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        PreparedArgValue::Eval(EvalValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
        PreparedArgValue::Eval(EvalValue::Text(_))
        | PreparedArgValue::Eval(EvalValue::Logical(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => Ok(0.0),
        PreparedArgValue::Eval(EvalValue::Array(_)) => Err(CoercionError::UnsupportedValueKind("array")),
    }
}

pub fn eval_sum_prepared_aggregate(
    args: &[AggregatePreparedValue],
) -> Result<EvalValue, SumEvalError> {
    let mut acc = 0.0;
    for item in args {
        acc += match item.origin {
            AggregateArgOrigin::DirectScalar =>
                accumulate_direct_scalar(&item.value).map_err(SumEvalError::Coercion)?,
            AggregateArgOrigin::ArrayLike(_) =>
                accumulate_range_like(&item.value).map_err(SumEvalError::Coercion)?,
        };
    }
    Ok(EvalValue::Number(acc))
}

pub fn eval_sum_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SumEvalError> {
    let argc = args.len();
    if !SUM_META.arity.accepts(argc) {
        return Err(SumEvalError::ArityMismatch {
            expected_min: SUM_META.arity.min,
            expected_max: SUM_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(SumEvalError::Coercion)?);
    }
    eval_sum_prepared_aggregate(&prepared)
}

pub fn map_sum_error_to_ws(e: &SumEvalError) -> WorksheetErrorCode {
    match e {
        SumEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SumEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SumEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::adapters::{
        AggregateArrayProvenance, expand_aggregate_array_with_provenance,
    };
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved_value: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved_value
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    #[test]
    fn eval_sum_on_numbers() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(6.0)));
    }

    #[test]
    fn eval_sum_coerces_direct_logical_and_numeric_text() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Logical(true)),
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn eval_sum_rejects_direct_non_numeric_text() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert!(matches!(got, Err(SumEvalError::Coercion(_))));
    }

    #[test]
    fn eval_sum_treats_missing_and_empty_direct_args_as_zero() {
        let args = vec![
            CallArgValue::MissingArg,
            CallArgValue::EmptyCell,
            CallArgValue::Eval(EvalValue::Number(4.0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(4.0)));
    }

    #[test]
    fn eval_sum_propagates_direct_scalar_error() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(
            got,
            Err(SumEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }

    #[test]
    fn eval_sum_ignores_reference_derived_text_and_logical() {
        let args = vec![CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A3".to_string(),
        })];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(5.0)],
                        vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        ))],
                        vec![ArrayCellValue::Logical(true)],
                    ])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn eval_sum_combines_direct_scalar_and_reference_derived_policies() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:A3".to_string(),
            }),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(5.0)],
                        vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "8".encode_utf16().collect(),
                        ))],
                        vec![ArrayCellValue::Logical(true)],
                    ])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(7.0)));
    }

    #[test]
    fn eval_sum_direct_array_literal_uses_array_scan_policy() {
        let array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            ArrayCellValue::Logical(true),
        ]])
        .unwrap();
        let prepared = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::DirectArrayLiteral,
        );

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_sum_opaque_array_fallback_uses_array_scan_policy() {
        let array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            ArrayCellValue::Logical(true),
        ]])
        .unwrap();
        let prepared = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::OpaqueArrayValue,
        );

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_sum_direct_arrays_use_range_like_policy() {
        let args = vec![CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    )),
                ],
                vec![ArrayCellValue::Logical(true), ArrayCellValue::Number(4.0)],
            ])
            .unwrap(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn eval_sum_propagates_errors_from_range_like_inputs() {
        let args = vec![CallArgValue::Eval(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Error(WorksheetErrorCode::Div0),
            ]])
            .unwrap(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(
            got,
            Err(SumEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }

    #[test]
    fn eval_sum_reference_derived_empty_cells_are_ignored() {
        let args = vec![CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A3".to_string(),
        })];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::EmptyCell],
                        vec![ArrayCellValue::Number(3.0)],
                        vec![ArrayCellValue::EmptyCell],
                    ])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }
}
