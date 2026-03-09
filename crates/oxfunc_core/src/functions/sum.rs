use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_args_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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

    // W10 keeps SUM in a conservative values-only seed mode.
    // Direct-vs-range dual coercion policy remains an explicit follow-up because
    // provenance is erased under `values_only_pre_adapter`.
    let prepared = prepare_args_values_only(args, resolver).map_err(SumEvalError::Coercion)?;
    let mut acc = 0.0;
    for arg in &prepared {
        let n = coerce_prepared_to_number(arg).map_err(SumEvalError::Coercion)?;
        acc += n;
    }
    Ok(EvalValue::Number(acc))
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

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
    fn eval_sum_coerces_logical_and_numeric_text() {
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
    fn eval_sum_rejects_non_numeric_text() {
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
    fn eval_sum_dereferences_reference_arg() {
        let args = vec![
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_sum_surface(
            &args,
            &MockResolver {
                resolved_value: Some(EvalValue::Number(5.0)),
            },
        );
        assert_eq!(got, Ok(EvalValue::Number(7.0)));
    }
}
