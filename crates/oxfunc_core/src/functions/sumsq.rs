use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregatePreparedValue, expand_aggregate_arg};
use crate::functions::aggregate_common::dual_policy_numeric_value;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const SUMSQ_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUMSQ",
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
pub enum SumsqEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_sumsq_aggregate(args: &[AggregatePreparedValue]) -> Result<EvalValue, SumsqEvalError> {
    let mut acc = 0.0;
    for arg in args {
        if let Some(value) = dual_policy_numeric_value(arg).map_err(SumsqEvalError::Coercion)? {
            acc += value * value;
        }
    }
    Ok(EvalValue::Number(acc))
}

pub fn eval_sumsq_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, SumsqEvalError> {
    let argc = args.len();
    if !SUMSQ_META.arity.accepts(argc) {
        return Err(SumsqEvalError::ArityMismatch {
            expected_min: SUMSQ_META.arity.min,
            expected_max: SUMSQ_META.arity.max,
            actual: argc,
        });
    }

    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(SumsqEvalError::Coercion)?);
    }
    eval_sumsq_aggregate(&prepared)
}

pub fn map_sumsq_error_to_ws(e: &SumsqEvalError) -> WorksheetErrorCode {
    match e {
        SumsqEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SumsqEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SumsqEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn eval_sumsq_accumulates_direct_numbers() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(4.0)),
        ];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(29.0)));
    }

    #[test]
    fn eval_sumsq_counts_direct_numeric_text_and_logical() {
        let args = vec![
            CallArgValue::Eval(EvalValue::Logical(true)),
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                "2".encode_utf16().collect(),
            ))),
        ];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn eval_sumsq_rejects_direct_non_numeric_text() {
        let args = vec![CallArgValue::Eval(EvalValue::Text(
            ExcelText::from_utf16_code_units("x".encode_utf16().collect()),
        ))];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: None,
            },
        );
        assert!(matches!(got, Err(SumsqEvalError::Coercion(_))));
    }

    #[test]
    fn eval_sumsq_ignores_reference_derived_text_and_logical() {
        let args = vec![CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A2".to_string(),
        })];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                        ArrayCellValue::Logical(true),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_sumsq_propagates_reference_derived_errors() {
        let args = vec![CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: "A1:A2".to_string(),
        })];
        let got = eval_sumsq_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ]])
                    .unwrap(),
                )),
            },
        );
        assert_eq!(
            got,
            Err(SumsqEvalError::Coercion(CoercionError::WorksheetError(
                WorksheetErrorCode::NA
            )))
        );
    }
}
