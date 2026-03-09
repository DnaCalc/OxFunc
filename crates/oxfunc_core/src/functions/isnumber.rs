use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const ISNUMBER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISNUMBER",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsnumberEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Preparation(crate::coercion::CoercionError),
}

pub fn eval_isnumber_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, IsnumberEvalError> {
    if !ISNUMBER_META.arity.accepts(args.len()) {
        return Err(IsnumberEvalError::ArityMismatch {
            expected: ISNUMBER_META.arity.min,
            actual: args.len(),
        });
    }

    let is_number = matches!(args[0], PreparedArgValue::Eval(EvalValue::Number(_)));
    Ok(EvalValue::Logical(is_number))
}

pub fn eval_isnumber_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IsnumberEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_isnumber_adapter_prepared,
        IsnumberEvalError::Preparation,
    )
}

pub fn map_isnumber_error_to_ws(e: &IsnumberEvalError) -> WorksheetErrorCode {
    match e {
        IsnumberEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IsnumberEvalError::Preparation(crate::coercion::CoercionError::WorksheetError(code)) => {
            *code
        }
        IsnumberEvalError::Preparation(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        value: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.value
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    #[test]
    fn eval_isnumber_on_number_is_true() {
        let args = [CallArgValue::Eval(EvalValue::Number(1.0))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn eval_isnumber_on_text_is_false() {
        let args = [CallArgValue::Eval(EvalValue::Text(
            ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
        ))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_isnumber_on_error_is_false() {
        let args = [CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(EvalValue::Logical(false)));
    }

    #[test]
    fn eval_isnumber_reference_path_uses_preparation() {
        let args = [CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        })];
        let got = eval_isnumber_surface(
            &args,
            &MockResolver {
                value: Some(EvalValue::Number(3.0)),
            },
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }
}
