use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const IFERROR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IFERROR",
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
pub enum IfErrorEvalError {
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

pub fn eval_iferror_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IfErrorEvalError> {
    if !IFERROR_META.arity.accepts(args.len()) {
        return Err(IfErrorEvalError::ArityMismatch {
            expected: IFERROR_META.arity.min,
            actual: args.len(),
        });
    }

    let primary = prepare_arg_values_only(&args[0], resolver)
        .map_err(IfErrorEvalError::PrimaryPreparation)?;
    match primary {
        PreparedArgValue::Eval(EvalValue::Error(_)) => {
            let fallback = prepare_arg_values_only(&args[1], resolver)
                .map_err(IfErrorEvalError::FallbackPreparation)?;
            Ok(prepared_to_eval(fallback))
        }
        other => Ok(prepared_to_eval(other)),
    }
}

pub fn map_iferror_error_to_ws(e: &IfErrorEvalError) -> WorksheetErrorCode {
    match e {
        IfErrorEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IfErrorEvalError::PrimaryPreparation(CoercionError::WorksheetError(code)) => *code,
        IfErrorEvalError::FallbackPreparation(CoercionError::WorksheetError(code)) => *code,
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
    fn eval_iferror_returns_primary_when_not_error() {
        let got = eval_iferror_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(4.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_iferror_returns_fallback_for_error_primary() {
        let got = eval_iferror_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "alt".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "alt".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_iferror_does_not_touch_fallback_when_primary_is_not_error() {
        let got = eval_iferror_surface(
            &[
                CallArgValue::EmptyCell,
                CallArgValue::Reference(ReferenceLike {
                    kind: crate::value::ReferenceKind::A1,
                    target: "Z99".to_string(),
                }),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_iferror_blank_fallback_becomes_zero_and_missing_fallback_is_value_error() {
        let blank_fallback = eval_iferror_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA)),
                CallArgValue::EmptyCell,
            ],
            &NoResolver,
        );
        assert_eq!(blank_fallback, Ok(EvalValue::Number(0.0)));

        let missing_fallback = eval_iferror_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA)),
                CallArgValue::MissingArg,
            ],
            &NoResolver,
        );
        assert_eq!(
            missing_fallback,
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        );
    }
}
