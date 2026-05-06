use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const IFNA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IFNA",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IfNaEvalError {
    ArityMismatch { expected: usize, actual: usize },
    PrimaryPreparation(CoercionError),
    FallbackPreparation(CoercionError),
}

fn prepared_to_eval(arg: PreparedArgValue) -> EvalValue {
    match arg {
        PreparedArgValue::Eval(v) => v,
        PreparedArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::Value),
        PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

pub fn eval_ifna_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, IfNaEvalError> {
    if !IFNA_META.arity.accepts(args.len()) {
        return Err(IfNaEvalError::ArityMismatch {
            expected: IFNA_META.arity.min,
            actual: args.len(),
        });
    }

    let primary =
        prepare_arg_values_only(&args[0], resolver).map_err(IfNaEvalError::PrimaryPreparation)?;
    match primary {
        PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA)) => {
            let fallback = prepare_arg_values_only(&args[1], resolver)
                .map_err(IfNaEvalError::FallbackPreparation)?;
            Ok(prepared_to_eval(fallback))
        }
        other => Ok(prepared_to_eval(other)),
    }
}

pub fn map_ifna_error_to_ws(e: &IfNaEvalError) -> WorksheetErrorCode {
    match e {
        IfNaEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IfNaEvalError::PrimaryPreparation(CoercionError::WorksheetError(code)) => *code,
        IfNaEvalError::FallbackPreparation(CoercionError::WorksheetError(code)) => *code,
        _ => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    #[test]
    fn ifna_catches_only_na() {
        assert_eq!(
            eval_ifna_surface(
                &[
                    CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA)),
                    CallArgValue::Eval(EvalValue::Number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Number(7.0))
        );
        assert_eq!(
            eval_ifna_surface(
                &[
                    CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                    CallArgValue::Eval(EvalValue::Number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn ifna_returns_primary_when_not_na() {
        assert_eq!(
            eval_ifna_surface(
                &[
                    CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    ))),
                    CallArgValue::Eval(EvalValue::Number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))
        );
    }
}
