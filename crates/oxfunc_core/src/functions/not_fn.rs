use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const NOT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NOT",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum NotEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn eval_not_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NotEvalError> {
    if !NOT_META.arity.accepts(args.len()) {
        return Err(NotEvalError::ArityMismatch {
            expected_min: NOT_META.arity.min,
            expected_max: NOT_META.arity.max,
            actual: args.len(),
        });
    }
    let value = coerce_prepared_to_number(&args[0]).map_err(NotEvalError::Coercion)?;
    Ok(EvalValue::Logical(value == 0.0))
}

pub fn eval_not_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NotEvalError> {
    run_values_only_prepared(args, resolver, eval_not_prepared, NotEvalError::Coercion)
}

pub fn map_not_error_to_ws(e: &NotEvalError) -> WorksheetErrorCode {
    match e {
        NotEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        NotEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        NotEvalError::Coercion(_) => WorksheetErrorCode::Value,
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
    fn eval_not_basic_lanes() {
        assert_eq!(
            eval_not_surface(
                &[CallArgValue::Eval(EvalValue::Logical(true))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(EvalValue::Logical(false))
        );
        assert_eq!(
            eval_not_surface(
                &[CallArgValue::Eval(EvalValue::Number(0.0))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_not_surface(
                &[CallArgValue::Eval(EvalValue::Number(2.0))],
                &MockResolver {
                    resolved_value: None
                }
            ),
            Ok(EvalValue::Logical(false))
        );
    }

    #[test]
    fn eval_not_direct_text_is_value_error() {
        let got = eval_not_surface(
            &[CallArgValue::Eval(EvalValue::Text(
                ExcelText::from_utf16_code_units("x".encode_utf16().collect()),
            ))],
            &MockResolver {
                resolved_value: None,
            },
        );
        assert!(matches!(
            got,
            Err(NotEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
    }

    #[test]
    fn eval_not_reference_uses_resolved_scalar() {
        let got = eval_not_surface(
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            })],
            &MockResolver {
                resolved_value: Some(EvalValue::Logical(true)),
            },
        );
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }
}
