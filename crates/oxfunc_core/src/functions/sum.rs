use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    AggregateArgOrigin, AggregatePreparedValue, expand_aggregate_arg,
    expand_sparse_reference_values_with_provenance, sparse_reference_values_for_aggregate_arg,
};
use crate::resolver::ReferenceSystemProvider;
use crate::semantic_kernel::{
    NumericalReductionPolicy, SemanticKernelRuntimeError, reduce_numeric_sum,
};
use crate::value::{CalcValue, CoreValue, FunctionArg, FunctionValue, WorksheetErrorCode};

pub const SUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUM",
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
pub enum SumEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    SemanticKernel(SemanticKernelRuntimeError),
}

fn accumulate_direct_scalar(arg: &CalcValue) -> Result<f64, CoercionError> {
    match arg.core() {
        CoreValue::Missing | CoreValue::Empty => Ok(0.0),
        _ => coerce_calc_scalar_to_number(arg),
    }
}

fn accumulate_range_like(arg: &CalcValue) -> Result<f64, CoercionError> {
    match arg.core() {
        CoreValue::Number(n) => Ok(*n),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference_like")),
        CoreValue::Text(_) | CoreValue::Logical(_) | CoreValue::Missing | CoreValue::Empty => {
            Ok(0.0)
        }
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
    }
}

pub(crate) fn eval_sum_prepared_aggregate(
    args: &[AggregatePreparedValue],
) -> Result<FunctionValue, SumEvalError> {
    let mut values = Vec::with_capacity(args.len());
    for item in args {
        let value = match item.origin() {
            AggregateArgOrigin::DirectScalar => {
                accumulate_direct_scalar(item.value()).map_err(SumEvalError::Coercion)?
            }
            AggregateArgOrigin::ArrayLike(_) => {
                accumulate_range_like(item.value()).map_err(SumEvalError::Coercion)?
            }
        };
        values.push(value);
    }
    reduce_numeric_sum(NumericalReductionPolicy::SequentialLeftFold, values)
        .map(FunctionValue::Number)
        .map_err(SumEvalError::SemanticKernel)
}

pub fn eval_sum_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, SumEvalError> {
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
        if let Some(values) = sparse_reference_values_for_aggregate_arg(arg, resolver)
            .map_err(SumEvalError::Coercion)?
        {
            prepared.extend(expand_sparse_reference_values_with_provenance(
                values,
                crate::functions::adapters::AggregateArrayProvenance::ReferenceDerived,
            ));
            continue;
        }
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(SumEvalError::Coercion)?);
    }
    eval_sum_prepared_aggregate(&prepared)
}

pub fn map_sum_error_to_ws(e: &SumEvalError) -> WorksheetErrorCode {
    match e {
        SumEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SumEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SumEvalError::Coercion(_) => WorksheetErrorCode::Value,
        SumEvalError::SemanticKernel(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::adapters::{
        AggregateArrayProvenance, expand_aggregate_array_with_provenance,
    };
    use crate::resolver::{
        ReferenceResolutionError, ReferenceSystemCapabilities, ResolvedReferenceCell,
        ResolvedReferenceExtent, ResolvedReferenceValues,
    };
    use crate::value::{ExcelText, FunctionArray, FunctionArrayCell, ReferenceKind, ReferenceLike};
    use std::cell::Cell;
    use std::collections::BTreeMap;

    struct MockResolver {
        resolved_value: Option<FunctionValue>,
        by_target: BTreeMap<String, FunctionValue>,
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
            if let Some(value) = self.by_target.get(reference.target()) {
                return Ok(value.clone());
            }
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    struct SparseResolver {
        values: ResolvedReferenceValues,
        dense_calls: Cell<usize>,
    }

    impl ReferenceSystemProvider for SparseResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.dense_calls.set(self.dense_calls.get() + 1);
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }

        fn enumerate_values(
            &self,
            _request: &crate::resolver::ReferenceEnumerationRequest,
        ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
            Ok(Some(self.values.clone()))
        }
    }

    #[test]
    fn eval_sum_on_numbers() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Number(2.0)),
            FunctionArg::Eval(FunctionValue::Number(3.0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(6.0)));
    }

    #[test]
    fn eval_sum_coerces_direct_logical_and_numeric_text() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Logical(true)),
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(3.0)));
    }

    #[test]
    fn eval_sum_rejects_direct_non_numeric_text() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_target: BTreeMap::new(),
            },
        );
        assert!(matches!(got, Err(SumEvalError::Coercion(_))));
    }

    #[test]
    fn eval_sum_treats_missing_and_empty_direct_args_as_zero() {
        let args = vec![
            FunctionArg::MissingArg,
            FunctionArg::EmptyCell,
            FunctionArg::Eval(FunctionValue::Number(4.0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(4.0)));
    }

    #[test]
    fn eval_sum_propagates_direct_scalar_error() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::Div0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_target: BTreeMap::new(),
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
    fn eval_sum_accepts_provider_materialized_multi_area_reference() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(Alpha!A1:A2,Alpha!B2)".to_string(),
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(7.0),
                    FunctionArrayCell::Number(11.0),
                    FunctionArrayCell::Number(13.0),
                ]])
                .unwrap(),
            ),
        );
        let got = eval_sum_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(Alpha!A1:A2,Alpha!B2)",
            ))],
            &MockResolver {
                resolved_value: None,
                by_target,
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(31.0)));
    }

    #[test]
    fn eval_sum_preserves_provider_materialized_multi_area_error_cells() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(A1:A2,C1)".to_string(),
            FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(7.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::Div0),
                    FunctionArrayCell::Number(13.0),
                ]])
                .unwrap(),
            ),
        );
        let got = eval_sum_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(A1:A2,C1)",
            ))],
            &MockResolver {
                resolved_value: None,
                by_target,
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
        let args = vec![FunctionArg::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![FunctionArrayCell::Number(5.0)],
                        vec![FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        ))],
                        vec![FunctionArrayCell::Logical(true)],
                    ])
                    .unwrap(),
                )),
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(5.0)));
    }

    #[test]
    fn eval_sum_admits_opaque_reference_value_through_generic_resolver() {
        let args = vec![FunctionArg::Eval(FunctionValue::Reference(
            ReferenceLike::new(ReferenceKind::Area, "NameBackedRange".to_string()),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Number(5.0),
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "8".encode_utf16().collect(),
                        )),
                        FunctionArrayCell::Logical(true),
                    ]])
                    .unwrap(),
                )),
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(5.0)));
    }

    #[test]
    fn eval_sum_combines_direct_scalar_and_reference_derived_policies() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            FunctionArg::Reference(ReferenceLike::new(ReferenceKind::Area, "A1:A3".to_string())),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![FunctionArrayCell::Number(5.0)],
                        vec![FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "8".encode_utf16().collect(),
                        ))],
                        vec![FunctionArrayCell::Logical(true)],
                    ])
                    .unwrap(),
                )),
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(7.0)));
    }

    #[test]
    fn eval_sum_direct_array_literal_uses_array_scan_policy() {
        let array = FunctionArray::from_rows(vec![vec![
            FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            FunctionArrayCell::Logical(true),
        ]])
        .unwrap();
        let prepared = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::DirectArrayLiteral,
        );

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(FunctionValue::Number(0.0)));
    }

    #[test]
    fn eval_sum_exercises_sequential_left_fold_reduction_policy() {
        let prepared = vec![
            AggregatePreparedValue::direct_scalar(CalcValue::number(1.0e16)),
            AggregatePreparedValue::direct_scalar(CalcValue::number(1.0)),
            AggregatePreparedValue::direct_scalar(CalcValue::number(-1.0e16)),
        ];

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(FunctionValue::Number(0.0)));
    }

    #[test]
    fn eval_sum_opaque_array_fallback_uses_array_scan_policy() {
        let array = FunctionArray::from_rows(vec![vec![
            FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            FunctionArrayCell::Logical(true),
        ]])
        .unwrap();
        let prepared = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::OpaqueArrayValue,
        );

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(FunctionValue::Number(0.0)));
    }

    #[test]
    fn eval_sum_direct_arrays_use_range_like_policy() {
        let args = vec![FunctionArg::Eval(FunctionValue::Array(
            FunctionArray::from_rows(vec![
                vec![
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    )),
                ],
                vec![
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Number(4.0),
                ],
            ])
            .unwrap(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(5.0)));
    }

    #[test]
    fn eval_sum_propagates_errors_from_range_like_inputs() {
        let args = vec![FunctionArg::Eval(FunctionValue::Array(
            FunctionArray::from_rows(vec![vec![
                FunctionArrayCell::Number(1.0),
                FunctionArrayCell::Error(WorksheetErrorCode::Div0),
            ]])
            .unwrap(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_target: BTreeMap::new(),
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
        let args = vec![FunctionArg::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![
                        vec![FunctionArrayCell::EmptyCell],
                        vec![FunctionArrayCell::Number(3.0)],
                        vec![FunctionArrayCell::EmptyCell],
                    ])
                    .unwrap(),
                )),
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(3.0)));
    }

    #[test]
    fn eval_sum_consumes_sparse_reference_values_without_dense_resolution() {
        let resolver = SparseResolver {
            values: ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(1_048_576, 1),
                vec![
                    ResolvedReferenceCell::new(1, 1, FunctionArrayCell::Number(2.0)),
                    ResolvedReferenceCell::new(1_048_576, 1, FunctionArrayCell::Number(3.0)),
                ],
                Some("reader:whole-column".to_string()),
            ),
            dense_calls: Cell::new(0),
        };

        let got = eval_sum_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A:A",
            ))],
            &resolver,
        );

        assert_eq!(got, Ok(FunctionValue::Number(5.0)));
        assert_eq!(resolver.dense_calls.get(), 0);
    }
}
