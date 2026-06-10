use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    AggregateArgOrigin, AggregatePreparedItem, expand_aggregate_arg,
};
use crate::resolver::ReferenceSystemProvider;
use crate::semantic_kernel::{
    NumericalReductionPolicy, SemanticKernelRuntimeError, reduce_numeric_sum,
};
use crate::value::CalcValue;
use crate::value::{CoreValue, WorksheetErrorCode};

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
    args: &[AggregatePreparedItem],
) -> Result<CalcValue, SumEvalError> {
    let mut values = Vec::with_capacity(args.len());
    for item in args {
        let value = match item.1 {
            AggregateArgOrigin::DirectScalar => {
                accumulate_direct_scalar(&item.0).map_err(SumEvalError::Coercion)?
            }
            AggregateArgOrigin::ArrayLike(_) => {
                accumulate_range_like(&item.0).map_err(SumEvalError::Coercion)?
            }
        };
        values.push(value);
    }
    reduce_numeric_sum(NumericalReductionPolicy::SequentialLeftFold, values)
        .map(CalcValue::number)
        .map_err(SumEvalError::SemanticKernel)
}

pub fn eval_sum_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SumEvalError> {
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
        ReferenceIdentityKey, ReferenceResolutionError, ReferenceSystemCapabilities,
        ResolvedReferenceCell, ResolvedReferenceExtent, ResolvedReferenceValues,
        reference_identity_key,
    };
    use crate::value::{CalcArray, ExcelText, ReferenceKind, ReferenceLike};
    use std::cell::Cell;
    use std::collections::BTreeMap;

    struct MockResolver {
        resolved_value: Option<CalcValue>,
        by_reference: BTreeMap<ReferenceIdentityKey, CalcValue>,
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
            if let Some(value) = self.by_reference.get(&reference_identity_key(reference)) {
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
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
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
            (CalcValue::number(1.0)),
            (CalcValue::number(2.0)),
            (CalcValue::number(3.0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(6.0)));
    }

    #[test]
    fn eval_sum_coerces_direct_logical_and_numeric_text() {
        let args = vec![
            (CalcValue::logical(true)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(3.0)));
    }

    #[test]
    fn eval_sum_rejects_direct_non_numeric_text() {
        let args = vec![
            (CalcValue::number(1.0)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_reference: BTreeMap::new(),
            },
        );
        assert!(matches!(got, Err(SumEvalError::Coercion(_))));
    }

    #[test]
    fn eval_sum_treats_missing_and_empty_direct_args_as_zero() {
        let args = vec![
            CalcValue::missing(),
            CalcValue::empty(),
            (CalcValue::number(4.0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(4.0)));
    }

    #[test]
    fn eval_sum_propagates_direct_scalar_error() {
        let args = vec![
            (CalcValue::number(1.0)),
            (CalcValue::error(WorksheetErrorCode::Div0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_reference: BTreeMap::new(),
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
        let reference = ReferenceLike::new(ReferenceKind::MultiArea, "(Alpha!A1:A2,Alpha!B2)");
        let mut by_reference = BTreeMap::new();
        by_reference.insert(
            reference_identity_key(&reference),
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(7.0),
                    CalcValue::number(11.0),
                    CalcValue::number(13.0),
                ]])
                .unwrap(),
            ),
        );
        let got = eval_sum_surface(
            &[CalcValue::reference(reference)],
            &MockResolver {
                resolved_value: None,
                by_reference,
            },
        );
        assert_eq!(got, Ok(CalcValue::number(31.0)));
    }

    #[test]
    fn eval_sum_preserves_provider_materialized_multi_area_error_cells() {
        let reference = ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,C1)");
        let mut by_reference = BTreeMap::new();
        by_reference.insert(
            reference_identity_key(&reference),
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(7.0),
                    CalcValue::error(WorksheetErrorCode::Div0),
                    CalcValue::number(13.0),
                ]])
                .unwrap(),
            ),
        );
        let got = eval_sum_surface(
            &[CalcValue::reference(reference)],
            &MockResolver {
                resolved_value: None,
                by_reference,
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
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(5.0)],
                        vec![CalcValue::text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        ))],
                        vec![CalcValue::logical(true)],
                    ])
                    .unwrap(),
                )),
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(5.0)));
    }

    #[test]
    fn eval_sum_admits_opaque_reference_value_through_generic_resolver() {
        let args = vec![
            (CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "NameBackedRange".to_string(),
            ))),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(5.0),
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "8".encode_utf16().collect(),
                        )),
                        CalcValue::logical(true),
                    ]])
                    .unwrap(),
                )),
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(5.0)));
    }

    #[test]
    fn eval_sum_combines_direct_scalar_and_reference_derived_policies() {
        let args = vec![
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:A3".to_string())),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(5.0)],
                        vec![CalcValue::text(ExcelText::from_utf16_code_units(
                            "8".encode_utf16().collect(),
                        ))],
                        vec![CalcValue::logical(true)],
                    ])
                    .unwrap(),
                )),
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(7.0)));
    }

    #[test]
    fn eval_sum_direct_array_literal_uses_array_scan_policy() {
        let array = CalcArray::from_rows(vec![vec![
            CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            CalcValue::logical(true),
        ]])
        .unwrap();
        let prepared = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::DirectArrayLiteral,
        );

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_sum_exercises_sequential_left_fold_reduction_policy() {
        let prepared = vec![
            (CalcValue::number(1.0e16), AggregateArgOrigin::DirectScalar),
            (CalcValue::number(1.0), AggregateArgOrigin::DirectScalar),
            (CalcValue::number(-1.0e16), AggregateArgOrigin::DirectScalar),
        ];

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_sum_opaque_array_fallback_uses_array_scan_policy() {
        let array = CalcArray::from_rows(vec![vec![
            CalcValue::text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            )),
            CalcValue::logical(true),
        ]])
        .unwrap();
        let prepared = expand_aggregate_array_with_provenance(
            &array,
            AggregateArrayProvenance::OpaqueArrayValue,
        );

        let got = eval_sum_prepared_aggregate(&prepared);
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_sum_direct_arrays_use_range_like_policy() {
        let args = vec![
            (CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        )),
                    ],
                    vec![CalcValue::logical(true), CalcValue::number(4.0)],
                ])
                .unwrap(),
            )),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(5.0)));
    }

    #[test]
    fn eval_sum_propagates_errors_from_range_like_inputs() {
        let args = vec![
            (CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::error(WorksheetErrorCode::Div0),
                ]])
                .unwrap(),
            )),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: None,
                by_reference: BTreeMap::new(),
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
        let args = vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::empty()],
                        vec![CalcValue::number(3.0)],
                        vec![CalcValue::empty()],
                    ])
                    .unwrap(),
                )),
                by_reference: BTreeMap::new(),
            },
        );
        assert_eq!(got, Ok(CalcValue::number(3.0)));
    }

    #[test]
    fn eval_sum_consumes_sparse_reference_values_without_dense_resolution() {
        let resolver = SparseResolver {
            values: ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(1_048_576, 1),
                vec![
                    ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                    ResolvedReferenceCell::new(1_048_576, 1, CalcValue::number(3.0)),
                ],
                Some("reader:whole-column".to_string()),
            ),
            dense_calls: Cell::new(0),
        };

        let got = eval_sum_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A:A",
            ))],
            &resolver,
        );

        assert_eq!(got, Ok(CalcValue::number(5.0)));
        assert_eq!(resolver.dense_calls.get(), 0);
    }
}
