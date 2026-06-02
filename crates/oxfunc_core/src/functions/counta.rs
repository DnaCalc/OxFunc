use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    AggregateArrayProvenance, expand_aggregate_arg, expand_sparse_reference_values_with_provenance,
    sparse_reference_values_for_aggregate_arg,
};
use crate::functions::aggregate_common::counta_argument_included;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

pub const COUNTA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNTA",
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
pub enum CountaEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
}

pub fn eval_counta_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, CountaEvalError> {
    let argc = args.len();
    if !COUNTA_META.arity.accepts(argc) {
        return Err(CountaEvalError::ArityMismatch {
            expected_min: COUNTA_META.arity.min,
            expected_max: COUNTA_META.arity.max,
            actual: argc,
        });
    }

    let mut count = 0.0;
    for arg in args {
        let prepared = if let Some(values) =
            sparse_reference_values_for_aggregate_arg(arg, resolver)
                .map_err(CountaEvalError::Preparation)?
        {
            expand_sparse_reference_values_with_provenance(
                values,
                AggregateArrayProvenance::ReferenceDerived,
            )
        } else {
            expand_aggregate_arg(arg, resolver).map_err(CountaEvalError::Preparation)?
        };
        for item in prepared {
            if counta_argument_included(&item).map_err(CountaEvalError::Preparation)? {
                count += 1.0;
            }
        }
    }

    Ok(FunctionValue::Number(count))
}

pub fn map_counta_error_to_ws(e: &CountaEvalError) -> WorksheetErrorCode {
    match e {
        CountaEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CountaEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        CountaEvalError::Preparation(_) => WorksheetErrorCode::Value,
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
    fn eval_counta_counts_non_empty_values() {
        let args = vec![
            FunctionArg::Eval(FunctionValue::Number(1.0)),
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                Vec::new(),
            ))),
            FunctionArg::MissingArg,
            FunctionArg::EmptyCell,
        ];
        let got = eval_counta_surface(&args, &MockResolver { resolved: None });
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn eval_counta_counts_reference_derived_error_and_empty_string_but_not_empty_cells() {
        let args = vec![FunctionArg::Reference(ReferenceLike::new(
            ReferenceKind::Area,
            "A1:A3".to_string(),
        ))];
        let got = eval_counta_surface(
            &args,
            &MockResolver {
                resolved: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(Vec::new())),
                        FunctionArrayCell::Error(WorksheetErrorCode::NA),
                        FunctionArrayCell::EmptyCell,
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn eval_counta_admits_opaque_reference_value_through_generic_resolver() {
        let args = vec![FunctionArg::Eval(FunctionValue::Reference(
            ReferenceLike::new(ReferenceKind::Area, "NameBackedRange".to_string()),
        ))];
        let got = eval_counta_surface(
            &args,
            &MockResolver {
                resolved: Some(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(Vec::new())),
                        FunctionArrayCell::Error(WorksheetErrorCode::NA),
                        FunctionArrayCell::EmptyCell,
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn eval_counta_direct_array_counts_empty_string_and_error() {
        let got = eval_counta_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(ExcelText::from_utf16_code_units(Vec::new())),
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    FunctionArrayCell::EmptyCell,
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn eval_counta_consumes_sparse_reference_values_without_dense_resolution() {
        let resolver = SparseResolver {
            values: ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(1000, 1),
                vec![
                    ResolvedReferenceCell::new(
                        1,
                        1,
                        FunctionArrayCell::Text(ExcelText::from_utf16_code_units(Vec::new())),
                    ),
                    ResolvedReferenceCell::new(
                        2,
                        1,
                        FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    ),
                ],
                Some("reader:counta-sparse".to_string()),
            ),
            dense_calls: Cell::new(0),
        };

        let got = eval_counta_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A1:A1000",
            ))],
            &resolver,
        );

        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
        assert_eq!(resolver.dense_calls.get(), 0);
    }
}
