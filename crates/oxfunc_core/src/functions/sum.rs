use crate::coercion::{coerce_calc_scalar_to_number, CoercionError};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    expand_aggregate_arg, expand_sparse_reference_values_with_provenance,
    sparse_reference_values_for_aggregate_arg, AggregateArgOrigin, AggregatePreparedValue,
};
use crate::resolver::ReferenceResolver;
use crate::semantic_kernel::{
    reduce_numeric_sum, NumericalReductionPolicy, SemanticKernelRuntimeError,
};
use crate::value::{CalcValue, CallArgValue, CoreValue, EvalValue, WorksheetErrorCode};

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
) -> Result<EvalValue, SumEvalError> {
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
        .map(EvalValue::Number)
        .map_err(SumEvalError::SemanticKernel)
}

pub fn eval_sum_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
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
        expand_aggregate_array_with_provenance, AggregateArrayProvenance,
    };
    use crate::resolver::{
        RefResolutionError, ResolvedReferenceCell, ResolvedReferenceExtent,
        ResolvedReferenceValues, ResolverCapabilities,
    };
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};
    use std::cell::Cell;
    use std::collections::BTreeMap;

    struct MockResolver {
        resolved_value: Option<EvalValue>,
        by_target: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            if let Some(value) = self.by_target.get(&reference.target) {
                return Ok(value.clone());
            }
            self.resolved_value
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    struct SparseResolver {
        values: ResolvedReferenceValues,
        dense_calls: Cell<usize>,
    }

    impl ReferenceResolver for SparseResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.dense_calls.set(self.dense_calls.get() + 1);
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }

        fn resolve_reference_values(
            &self,
            _reference: &ReferenceLike,
        ) -> Result<Option<ResolvedReferenceValues>, RefResolutionError> {
            Ok(Some(self.values.clone()))
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
                by_target: BTreeMap::new(),
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
                by_target: BTreeMap::new(),
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
                by_target: BTreeMap::new(),
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
                by_target: BTreeMap::new(),
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
    fn eval_sum_materializes_multi_area_reference_through_resolver() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "Alpha!A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(7.0)],
                    vec![ArrayCellValue::Number(11.0)],
                ])
                .unwrap(),
            ),
        );
        by_target.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));
        let got = eval_sum_surface(
            &[CallArgValue::Reference(ReferenceLike::new(
                ReferenceKind::MultiArea,
                "(Alpha!A1:A2,Alpha!B2)",
            ))],
            &MockResolver {
                resolved_value: None,
                by_target,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(31.0)));
    }

    #[test]
    fn eval_sum_preserves_multi_area_member_error_cells() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(7.0)],
                    vec![ArrayCellValue::Error(WorksheetErrorCode::Div0)],
                ])
                .unwrap(),
            ),
        );
        by_target.insert("C1".to_string(), EvalValue::Number(13.0));
        let got = eval_sum_surface(
            &[CallArgValue::Reference(ReferenceLike::new(
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
        let args = vec![CallArgValue::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
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
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn eval_sum_admits_opaque_reference_value_through_generic_resolver() {
        let args = vec![CallArgValue::Eval(EvalValue::Reference(
            ReferenceLike::new(ReferenceKind::Area, "NameBackedRange".to_string()),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(5.0),
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "8".encode_utf16().collect(),
                        )),
                        ArrayCellValue::Logical(true),
                    ]])
                    .unwrap(),
                )),
                by_target: BTreeMap::new(),
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
            CallArgValue::Reference(ReferenceLike::new(ReferenceKind::Area, "A1:A3".to_string())),
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
                by_target: BTreeMap::new(),
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
    fn eval_sum_exercises_sequential_left_fold_reduction_policy() {
        let prepared = vec![
            AggregatePreparedValue::direct_scalar(CalcValue::number(1.0e16)),
            AggregatePreparedValue::direct_scalar(CalcValue::number(1.0)),
            AggregatePreparedValue::direct_scalar(CalcValue::number(-1.0e16)),
        ];

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
                by_target: BTreeMap::new(),
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
        let args = vec![CallArgValue::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
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
                by_target: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn eval_sum_consumes_sparse_reference_values_without_dense_resolution() {
        let resolver = SparseResolver {
            values: ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(1_048_576, 1),
                vec![
                    ResolvedReferenceCell::new(1, 1, ArrayCellValue::Number(2.0)),
                    ResolvedReferenceCell::new(1_048_576, 1, ArrayCellValue::Number(3.0)),
                ],
                Some("reader:whole-column".to_string()),
            ),
            dense_calls: Cell::new(0),
        };

        let got = eval_sum_surface(
            &[CallArgValue::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A:A",
            ))],
            &resolver,
        );

        assert_eq!(got, Ok(EvalValue::Number(5.0)));
        assert_eq!(resolver.dense_calls.get(), 0);
    }
}
