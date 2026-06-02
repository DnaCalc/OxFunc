use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    AggregateArrayProvenance, expand_aggregate_arg, expand_sparse_reference_values_with_provenance,
    sparse_reference_values_for_aggregate_arg,
};
use crate::functions::aggregate_common::count_argument_included;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

pub const COUNT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNT",
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
pub enum CountEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn eval_count_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, CountEvalError> {
    let argc = args.len();
    if !COUNT_META.arity.accepts(argc) {
        return Err(CountEvalError::ArityMismatch {
            expected_min: COUNT_META.arity.min,
            expected_max: COUNT_META.arity.max,
            actual: argc,
        });
    }

    let mut count = 0.0;
    for arg in args {
        let prepared = if let Some(values) =
            sparse_reference_values_for_aggregate_arg(arg, resolver)
                .map_err(CountEvalError::Coercion)?
        {
            expand_sparse_reference_values_with_provenance(
                values,
                AggregateArrayProvenance::ReferenceDerived,
            )
        } else {
            expand_aggregate_arg(arg, resolver).map_err(CountEvalError::Coercion)?
        };
        for item in prepared {
            if count_argument_included(&item).map_err(CountEvalError::Coercion)? {
                count += 1.0;
            }
        }
    }

    Ok(FunctionValue::Number(count))
}

pub fn map_count_error_to_ws(e: &CountEvalError) -> WorksheetErrorCode {
    match e {
        CountEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CountEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CountEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{
        ReferenceResolutionError, ReferenceSystemCapabilities, ResolvedReferenceCell,
        ResolvedReferenceExtent, ResolvedReferenceValues,
    };
    use crate::value::{ExcelText, FunctionArray, FunctionArrayCell, ReferenceKind, ReferenceLike};
    use std::cell::Cell;

    struct MockResolver {
        resolved: Option<FunctionValue>,
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
            self.resolved.clone().ok_or(
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
    fn eval_count_counts_direct_numeric_text_and_logical() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Logical(true)),
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "bad".encode_utf16().collect(),
            ))),
            FunctionArg::EmptyCell,
        ];
        let got = eval_count_surface(&args, &MockResolver { resolved: None });
        assert_eq!(got, Ok(FunctionValue::Number(3.0)));
    }

    #[test]
    fn eval_count_ignores_reference_text_and_logical() {
        let got = eval_count_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A3".to_string(),
            ))],
            &MockResolver {
                resolved: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "3".encode_utf16().collect(),
                        )),
                        FunctionArrayCell::Logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(1.0)));
    }

    #[test]
    fn eval_count_admits_opaque_reference_value_through_generic_resolver() {
        let got = eval_count_surface(
            &[FunctionArg::Eval(FunctionValue::Reference(
                ReferenceLike::new(ReferenceKind::Area, "NameBackedRange".to_string()),
            ))],
            &MockResolver {
                resolved: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Number(2.0),
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "3".encode_utf16().collect(),
                        )),
                        FunctionArrayCell::Logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(1.0)));
    }

    #[test]
    fn eval_count_propagates_worksheet_errors() {
        let got = eval_count_surface(
            &[FunctionArg::Eval(FunctionValue::Error(
                WorksheetErrorCode::Div0,
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Err(CountEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::Div0
            )))
        );
    }

    #[test]
    fn eval_count_direct_array_uses_range_like_policy() {
        let got = eval_count_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                        "2".encode_utf16().collect(),
                    )),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(FunctionValue::Number(0.0)));
    }

    #[test]
    fn eval_count_consumes_sparse_reference_values_without_dense_resolution() {
        let resolver = SparseResolver {
            values: ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(1000, 1),
                vec![
                    ResolvedReferenceCell::new(1, 1, FunctionArrayCell::Number(2.0)),
                    ResolvedReferenceCell::new(
                        2,
                        1,
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(
                            "3".encode_utf16().collect(),
                        )),
                    ),
                    ResolvedReferenceCell::new(3, 1, FunctionArrayCell::Logical(true)),
                ],
                Some("reader:count-sparse".to_string()),
            ),
            dense_calls: Cell::new(0),
        };

        let got = eval_count_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A1000",
            ))],
            &resolver,
        );

        assert_eq!(got, Ok(FunctionValue::Number(1.0)));
        assert_eq!(resolver.dense_calls.get(), 0);
    }
}
