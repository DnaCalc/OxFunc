use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, prepare_arg_values_only};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

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

fn prepared_to_eval(arg: PreparedValue) -> FunctionValue {
    match arg {
        PreparedValue::Eval(v) => v,
        PreparedValue::MissingArg => FunctionValue::Error(WorksheetErrorCode::Value),
        PreparedValue::EmptyCell => FunctionValue::Number(0.0),
    }
}

pub fn eval_ifna_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, IfNaEvalError> {
    if !IFNA_META.arity.accepts(args.len()) {
        return Err(IfNaEvalError::ArityMismatch {
            expected: IFNA_META.arity.min,
            actual: args.len(),
        });
    }

    let primary =
        prepare_arg_values_only(&args[0], resolver).map_err(IfNaEvalError::PrimaryPreparation)?;
    match primary {
        PreparedValue::Eval(FunctionValue::Error(WorksheetErrorCode::NA)) => {
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn ifna_catches_only_na() {
        assert_eq!(
            eval_ifna_surface(
                &[
                    FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::NA)),
                    FunctionArg::Eval(FunctionValue::Number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(FunctionValue::Number(7.0))
        );
        assert_eq!(
            eval_ifna_surface(
                &[
                    FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::Div0)),
                    FunctionArg::Eval(FunctionValue::Number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(FunctionValue::Error(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn ifna_returns_primary_when_not_na() {
        assert_eq!(
            eval_ifna_surface(
                &[
                    FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                        "x".encode_utf16().collect(),
                    ))),
                    FunctionArg::Eval(FunctionValue::Number(7.0)),
                ],
                &NoResolver,
            ),
            Ok(FunctionValue::Text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect(),
            )))
        );
    }
}
