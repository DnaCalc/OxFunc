use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, expand_aggregate_arg};
use crate::functions::adapters::{AggregateArgOrigin, AggregateArrayProvenance};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const COUNTBLANK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COUNTBLANK",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CountBlankEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
}

fn prepared_counts_as_blank(value: &PreparedArgValue) -> Result<bool, CoercionError> {
    match value {
        PreparedArgValue::EmptyCell => Ok(true),
        PreparedArgValue::Eval(EvalValue::Text(t)) => Ok(t.utf16_code_units().is_empty()),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        PreparedArgValue::MissingArg => Ok(false),
        _ => Ok(false),
    }
}

pub fn eval_countblank_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CountBlankEvalError> {
    let argc = args.len();
    if !COUNTBLANK_META.arity.accepts(argc) {
        return Err(CountBlankEvalError::ArityMismatch {
            expected_min: COUNTBLANK_META.arity.min,
            expected_max: COUNTBLANK_META.arity.max,
            actual: argc,
        });
    }

    let mut count = 0.0;
    for arg in args {
        for item in expand_aggregate_arg(arg, resolver).map_err(CountBlankEvalError::Preparation)? {
            if matches!(
                item.origin,
                AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::OpaqueArrayValue)
                    | AggregateArgOrigin::ArrayLike(AggregateArrayProvenance::DirectArrayLiteral)
            ) {
                return Err(CountBlankEvalError::Preparation(
                    CoercionError::UnsupportedValueKind("countblank_array_substitute"),
                ));
            }
            if prepared_counts_as_blank(&item.value).map_err(CountBlankEvalError::Preparation)? {
                count += 1.0;
            }
        }
    }

    Ok(EvalValue::Number(count))
}

pub fn map_countblank_error_to_ws(e: &CountBlankEvalError) -> WorksheetErrorCode {
    match e {
        CountBlankEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CountBlankEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        CountBlankEvalError::Preparation(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    #[test]
    fn countblank_counts_empty_cells_and_empty_strings() {
        let got = eval_countblank_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "D1:D3".to_string(),
            })],
            &MockResolver {
                resolved: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::EmptyCell,
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(Vec::new())),
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn countblank_propagates_errors() {
        let got = eval_countblank_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "D1".to_string(),
            })],
            &MockResolver {
                resolved: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![ArrayCellValue::Error(WorksheetErrorCode::NA)]])
                        .unwrap(),
                )),
            },
        );
        assert_eq!(
            got,
            Err(CountBlankEvalError::Preparation(
                CoercionError::WorksheetError(WorksheetErrorCode::NA,)
            ))
        );
    }

    #[test]
    fn countblank_rejects_array_valued_substitutes() {
        let got = eval_countblank_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_utf16_code_units(Vec::new()))],
                    vec![ArrayCellValue::Number(1.0)],
                ])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Err(CountBlankEvalError::Preparation(
                CoercionError::UnsupportedValueKind("countblank_array_substitute")
            ))
        );
    }

    #[test]
    fn countblank_meta_preserves_reference_visibility_for_true_ranges() {
        assert_eq!(
            COUNTBLANK_META.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        );
    }
}
